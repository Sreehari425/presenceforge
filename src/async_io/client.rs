//! Async Discord IPC Client implementation

#![allow(clippy::collapsible_if)]

use bytes::{BufMut, BytesMut};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::process;
use std::time::{Duration, Instant};

use super::traits::ipc_utils::read_u32_le;
use super::traits::{read_exact, write_all, AsyncRead, AsyncWrite};
use crate::activity::Activity;
use crate::debug_println;
use crate::error::{DiscordIpcError, HandshakeFailureKind, InvalidResponseKind, Result};
use crate::ipc::{
    constants, Command, EventData, HandshakePayload, IpcMessage, IpcResponse, Opcode, ReadyEvent,
};
use crate::nonce::generate_nonce;

/// Async implementation of Discord IPC client
pub struct AsyncDiscordIpcClient<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    connection: T,
    client_id: String,
    read_buf: BytesMut,
    write_buf: BytesMut,
    pending_messages: VecDeque<PendingMessage>,
    connected: bool,
}

impl<T> AsyncDiscordIpcClient<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// Initial capacity for read and write buffers (4KB)
    const INITIAL_BUFFER_CAPACITY: usize = 4096;

    /// Creates a new async Discord IPC client
    ///
    /// This constructor doesn't establish a connection yet.
    /// Call `connect()` to establish a connection.
    pub fn new(client_id: impl Into<String>, connection: T) -> Self {
        Self {
            connection,
            client_id: client_id.into(),
            read_buf: BytesMut::with_capacity(Self::INITIAL_BUFFER_CAPACITY),
            write_buf: BytesMut::with_capacity(Self::INITIAL_BUFFER_CAPACITY),
            pending_messages: VecDeque::new(),
            connected: false,
        }
    }

    /// Performs handshake with Discord
    ///
    /// # Returns
    ///
    /// A `Result` containing the Discord handshake response
    ///
    /// # Errors
    ///
    /// Returns `DiscordIpcError::HandshakeFailed` if the handshake fails
    pub async fn connect(&mut self) -> Result<Value> {
        self.pending_messages.clear();
        self.connected = false;

        let handshake = HandshakePayload {
            v: constants::IPC_VERSION,
            client_id: self.client_id.clone(),
        };

        let payload =
            serde_json::to_value(handshake).map_err(DiscordIpcError::SerializationFailed)?;

        self.send_message(Opcode::Handshake, &payload).await?;

        let (opcode, response) = self.recv_from_connection().await?;
        debug_println!("Handshake response: {}", response);

        // Check for error in the response
        if let Some(err) = response.get("error") {
            if let (Some(code), Some(message)) = (
                err.get("code").and_then(|c| c.as_i64()),
                err.get("message").and_then(|m| m.as_str()),
            ) {
                return Err(DiscordIpcError::discord_error(code as i32, message));
            } else {
                return Err(DiscordIpcError::handshake_failed(
                    HandshakeFailureKind::InvalidErrorPayload,
                    format!("Invalid error format: {}", err),
                ));
            }
        }

        // Verify opcode is correct for handshake response
        if !opcode.is_handshake_response() {
            return Err(DiscordIpcError::handshake_failed(
                HandshakeFailureKind::UnexpectedOpcode,
                format!("Expected handshake response opcode, got {:?}", opcode),
            ));
        }

        self.connected = true;
        Ok(response)
    }

    /// Perform handshake and return the typed READY payload when available.
    pub async fn connect_with_ready(&mut self) -> Result<Option<ReadyEvent>> {
        let response = self.connect().await?;
        Self::ready_event_from_payload(&response)
    }

    /// Parse a raw IPC payload into a READY event if this payload is a READY dispatch.
    pub fn ready_event_from_payload(payload: &Value) -> Result<Option<ReadyEvent>> {
        let response: IpcResponse = serde_json::from_value(payload.clone())
            .map_err(DiscordIpcError::DeserializationFailed)?;

        match response.parse_event()? {
            Some(EventData::Ready(ready)) => Ok(Some(ready)),
            _ => Ok(None),
        }
    }

    /// Sets Discord Rich Presence activity
    ///
    /// # Arguments
    ///
    /// * `activity` - The activity to set
    ///
    /// # Errors
    ///
    /// Returns a `DiscordIpcError` if serialization fails or if Discord returns an error
    pub async fn set_activity(&mut self, activity: &Activity) -> Result<()> {
        // Validate the activity first
        activity.validate()?;

        // Generate a cryptographically secure unique nonce for this request
        let nonce = generate_nonce("set-activity");

        let message = IpcMessage {
            cmd: Command::SetActivity,
            args: json!({
                "pid": process::id(),
                "activity": activity
            }),
            nonce: nonce.clone(),
            evt: None,
        };

        let payload = serde_json::to_value(message)?;
        self.send_message(Opcode::Frame, &payload).await?;

        // Receive the response to check for errors
        let (opcode, response) = self.recv_for_nonce(&nonce).await?;

        // Check if we got the correct response type
        if !opcode.is_frame_response() {
            return Err(DiscordIpcError::invalid_response(
                InvalidResponseKind::UnexpectedOpcode,
                format!("Expected frame response, got {:?}", opcode),
            ));
        }

        // Check for error in the response
        if let Some(err) = response.get("error") {
            if let (Some(code), Some(message)) = (
                err.get("code").and_then(|c| c.as_i64()),
                err.get("message").and_then(|m| m.as_str()),
            ) {
                return Err(DiscordIpcError::discord_error(code as i32, message));
            } else {
                return Err(DiscordIpcError::invalid_response(
                    InvalidResponseKind::InvalidErrorPayload,
                    format!("Invalid error format in response: {}", err),
                ));
            }
        }

        // Verify nonce matches to ensure we got the right response
        if let Some(resp_nonce) = response.get("nonce").and_then(|n| n.as_str()) {
            if resp_nonce != nonce {
                return Err(DiscordIpcError::invalid_response(
                    InvalidResponseKind::NonceMismatch,
                    format!("Nonce mismatch: expected {}, got {}", nonce, resp_nonce),
                ));
            }
        }

        Ok(())
    }

    /// Clears Discord Rich Presence activity
    ///
    /// # Returns
    ///
    /// A `Result` containing the Discord response
    ///
    /// # Errors
    ///
    /// Returns a `DiscordIpcError` if communication fails or if Discord returns an error
    pub async fn clear_activity(&mut self) -> Result<Value> {
        // Generate a cryptographically secure unique nonce
        let nonce = generate_nonce("clear-activity");

        let message = IpcMessage {
            cmd: Command::SetActivity,
            args: json!({
                "pid": process::id(),
                "activity": Value::Null
            }),
            nonce: nonce.clone(),
            evt: None,
        };

        let payload = serde_json::to_value(message)?;
        self.send_message(Opcode::Frame, &payload).await?;

        let (opcode, response) = self.recv_for_nonce(&nonce).await?;
        debug_println!("Clear Activity response: {}", response);

        // Check if we got the correct response type
        if !opcode.is_frame_response() {
            return Err(DiscordIpcError::invalid_response(
                InvalidResponseKind::UnexpectedOpcode,
                format!("Expected frame response, got {:?}", opcode),
            ));
        }

        // Check for error in the response
        if let Some(err) = response.get("error") {
            if let (Some(code), Some(message)) = (
                err.get("code").and_then(|c| c.as_i64()),
                err.get("message").and_then(|m| m.as_str()),
            ) {
                return Err(DiscordIpcError::discord_error(code as i32, message));
            } else {
                return Err(DiscordIpcError::invalid_response(
                    InvalidResponseKind::InvalidErrorPayload,
                    format!("Invalid error format in response: {}", err),
                ));
            }
        }

        // Verify nonce matches to ensure we got the right response
        if let Some(resp_nonce) = response.get("nonce").and_then(|n| n.as_str()) {
            if resp_nonce != nonce {
                return Err(DiscordIpcError::invalid_response(
                    InvalidResponseKind::NonceMismatch,
                    format!("Nonce mismatch: expected {}, got {}", nonce, resp_nonce),
                ));
            }
        }

        Ok(response)
    }

    /// Subscribe to a Discord IPC event
    pub async fn subscribe<S: Into<String>>(&mut self, event: S, args: Value) -> Result<()> {
        let event = event.into();
        let nonce = generate_nonce("subscribe");

        let message = IpcMessage {
            cmd: Command::Subscribe,
            args,
            nonce: nonce.clone(),
            evt: Some(event),
        };

        let payload = serde_json::to_value(message)?;
        self.send_message(Opcode::Frame, &payload).await?;

        let (_, response) = self.recv_for_nonce(&nonce).await?;

        if let Some(err) = response.get("error") {
            let code = err.get("code").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
            let message = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(DiscordIpcError::discord_error(code, message));
        }

        Ok(())
    }

    /// Unsubscribe from a Discord IPC event
    pub async fn unsubscribe<S: Into<String>>(&mut self, event: S, args: Value) -> Result<()> {
        let event = event.into();
        let nonce = generate_nonce("unsubscribe");

        let message = IpcMessage {
            cmd: Command::Unsubscribe,
            args,
            nonce: nonce.clone(),
            evt: Some(event),
        };

        let payload = serde_json::to_value(message)?;
        self.send_message(Opcode::Frame, &payload).await?;

        let (_, response) = self.recv_for_nonce(&nonce).await?;

        if let Some(err) = response.get("error") {
            let code = err.get("code").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
            let message = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(DiscordIpcError::discord_error(code, message));
        }

        Ok(())
    }

    /// Waits for the next event asynchronously
    pub async fn next_event(&mut self) -> Result<EventData> {
        if let Some(event) = self.take_pending_event()? {
            return Ok(event);
        }

        loop {
            let (opcode, payload) = self.recv_from_connection().await?;

            if !opcode.is_frame_response() {
                self.pending_messages
                    .push_back(PendingMessage::new(opcode, payload));
                continue;
            }

            let response: IpcResponse = serde_json::from_value(payload.clone())
                .map_err(DiscordIpcError::DeserializationFailed)?;

            if let Some(event) = response.parse_event()? {
                return Ok(event);
            }

            self.pending_messages
                .push_back(PendingMessage::new(opcode, payload));
        }
    }

    /// Sends a raw IPC message
    ///
    /// # Arguments
    ///
    /// * `opcode` - The opcode to send
    /// * `payload` - The JSON payload to send
    ///
    /// # Errors
    ///
    /// Returns a `DiscordIpcError` if serialization or communication fails
    pub async fn send_message(&mut self, opcode: Opcode, payload: &Value) -> Result<()> {
        let raw = serde_json::to_vec(payload)?;

        // Clear and prepare write buffer
        self.write_buf.clear();
        self.write_buf.reserve(8 + raw.len());

        // Write header and payload to buffer
        self.write_buf.put_u32_le(opcode.into());
        self.write_buf.put_u32_le(raw.len() as u32);
        self.write_buf.extend_from_slice(&raw);

        // Write entire buffer at once
        write_all(&mut self.connection, &self.write_buf).await?;

        Ok(())
    }

    /// Receives a raw IPC message
    ///
    /// # Returns
    ///
    /// A tuple containing the opcode and JSON payload
    ///
    /// # Errors
    ///
    /// Returns a `DiscordIpcError` if deserialization or communication fails
    pub async fn recv_message(&mut self) -> Result<(Opcode, Value)> {
        self.next_message().await
    }

    /// Returns `true` once a handshake has been successfully completed.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Remove pending responses older than the provided `max_age` and return how many were dropped.
    pub fn cleanup_pending(&mut self, max_age: Duration) -> usize {
        if max_age.is_zero() {
            let dropped = self.pending_messages.len();
            self.pending_messages.clear();
            return dropped;
        }

        let now = Instant::now();
        let original_len = self.pending_messages.len();
        self.pending_messages
            .retain(|message| now.saturating_duration_since(message.received_at) <= max_age);
        original_len - self.pending_messages.len()
    }

    async fn next_message(&mut self) -> Result<(Opcode, Value)> {
        if let Some(message) = self.pending_messages.pop_front() {
            let PendingMessage {
                opcode, payload, ..
            } = message;
            return Ok((opcode, payload));
        }

        self.recv_from_connection().await
    }

    async fn recv_for_nonce(&mut self, expected_nonce: &str) -> Result<(Opcode, Value)> {
        if let Some(message) = self.take_pending_by_nonce(expected_nonce) {
            return Ok(message);
        }

        loop {
            let (opcode, response) = self.recv_from_connection().await?;
            if Self::value_has_nonce(&response, expected_nonce) {
                return Ok((opcode, response));
            }

            self.pending_messages
                .push_back(PendingMessage::new(opcode, response));
        }
    }

    async fn recv_from_connection(&mut self) -> Result<(Opcode, Value)> {
        // Read header using utility function
        let opcode_raw = read_u32_le(&mut self.connection).await?;
        let length = read_u32_le(&mut self.connection).await?;

        // Validate payload size to prevent excessive memory allocation
        if length > crate::ipc::protocol::constants::MAX_PAYLOAD_SIZE {
            return Err(DiscordIpcError::invalid_response(
                InvalidResponseKind::PayloadTooLarge,
                format!(
                    "Payload size {} exceeds maximum allowed size of {} bytes",
                    length,
                    crate::ipc::protocol::constants::MAX_PAYLOAD_SIZE
                ),
            ));
        }

        let opcode = Opcode::try_from(opcode_raw)?;

        // Reuse read buffer for payload
        self.read_buf.clear();
        self.read_buf.resize(length as usize, 0);

        read_exact(&mut self.connection, &mut self.read_buf[..])
            .await
            .map_err(|_| DiscordIpcError::SocketClosed)?;

        let value: Value = serde_json::from_slice(&self.read_buf)?;

        Ok((opcode, value))
    }

    fn take_pending_by_nonce(&mut self, expected_nonce: &str) -> Option<(Opcode, Value)> {
        let position = self
            .pending_messages
            .iter()
            .position(|message| Self::value_has_nonce(&message.payload, expected_nonce));

        position.and_then(|index| {
            self.pending_messages.remove(index).map(|message| {
                let PendingMessage {
                    opcode, payload, ..
                } = message;
                (opcode, payload)
            })
        })
    }

    fn value_has_nonce(value: &Value, expected_nonce: &str) -> bool {
        value
            .get("nonce")
            .and_then(|n| n.as_str())
            .map(|actual| actual == expected_nonce)
            .unwrap_or(false)
    }

    fn take_pending_event(&mut self) -> Result<Option<EventData>> {
        let position = self
            .pending_messages
            .iter()
            .position(|message| Self::value_is_event(&message.payload));

        if let Some(index) = position {
            if let Some(message) = self.pending_messages.remove(index) {
                let response: IpcResponse = serde_json::from_value(message.payload)
                    .map_err(DiscordIpcError::DeserializationFailed)?;
                return response.parse_event();
            }
        }

        Ok(None)
    }

    fn value_is_event(value: &Value) -> bool {
        value
            .get("evt")
            .and_then(|evt| evt.as_str())
            .is_some()
    }
}

#[derive(Debug)]
struct PendingMessage {
    opcode: Opcode,
    payload: Value,
    received_at: Instant,
}

impl PendingMessage {
    fn new(opcode: Opcode, payload: Value) -> Self {
        Self {
            opcode,
            payload,
            received_at: Instant::now(),
        }
    }
}
