//! Async Discord IPC Client implementation

use serde_json::{json, Value};
use std::process;

use super::traits::ipc_utils::{read_u32_le, write_u32_le};
use super::traits::{read_exact, write_all, AsyncRead, AsyncWrite};
use crate::activity::Activity;
use crate::debug_println;
use crate::error::{DiscordIpcError, Result};
use crate::ipc::{constants, Command, HandshakePayload, IpcMessage, Opcode};

/// Async implementation of Discord IPC client
pub struct AsyncDiscordIpcClient<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    connection: T,
    client_id: String,
}

impl<T> AsyncDiscordIpcClient<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// Creates a new async Discord IPC client
    ///
    /// This constructor doesn't establish a connection yet.
    /// Call `connect()` to establish a connection.
    pub fn new(client_id: impl Into<String>, connection: T) -> Self {
        Self {
            connection,
            client_id: client_id.into(),
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
        let handshake = HandshakePayload {
            v: constants::IPC_VERSION,
            client_id: self.client_id.clone(),
        };

        let payload =
            serde_json::to_value(handshake).map_err(DiscordIpcError::SerializationFailed)?;

        self.send_message(Opcode::Handshake, &payload).await?;

        let (opcode, response) = self.recv_message().await?;
        debug_println!("Handshake response: {}", response);

        // Check for error in the response
        if let Some(err) = response.get("error") {
            if let (Some(code), Some(message)) = (
                err.get("code").and_then(|c| c.as_i64()),
                err.get("message").and_then(|m| m.as_str()),
            ) {
                return Err(DiscordIpcError::discord_error(code as i32, message));
            } else {
                return Err(DiscordIpcError::HandshakeFailed(format!(
                    "Invalid error format: {}",
                    err
                )));
            }
        }

        // Verify opcode is correct for handshake response
        if !opcode.is_handshake_response() {
            return Err(DiscordIpcError::HandshakeFailed(format!(
                "Expected handshake response opcode, got {:?}",
                opcode
            )));
        }

        Ok(response)
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
        if let Err(reason) = activity.validate() {
            return Err(DiscordIpcError::InvalidActivity(reason));
        }

        // Generate a unique nonce for this request using a timestamp
        let nonce = format!(
            "set-activity-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        let message = IpcMessage {
            cmd: Command::SetActivity,
            args: json!({
                "pid": process::id(),
                "activity": activity
            }),
            nonce: nonce.clone(),
        };

        let payload = serde_json::to_value(message)?;
        self.send_message(Opcode::Frame, &payload).await?;

        // Receive the response to check for errors
        let (opcode, response) = self.recv_message().await?;

        // Check if we got the correct response type
        if !opcode.is_frame_response() {
            return Err(DiscordIpcError::InvalidResponse(format!(
                "Expected frame response, got {:?}",
                opcode
            )));
        }

        // Check for error in the response
        if let Some(err) = response.get("error") {
            if let (Some(code), Some(message)) = (
                err.get("code").and_then(|c| c.as_i64()),
                err.get("message").and_then(|m| m.as_str()),
            ) {
                return Err(DiscordIpcError::discord_error(code as i32, message));
            } else {
                return Err(DiscordIpcError::InvalidResponse(format!(
                    "Invalid error format in response: {}",
                    err
                )));
            }
        }

        // Verify nonce matches to ensure we got the right response
        if let Some(resp_nonce) = response.get("nonce").and_then(|n| n.as_str()) {
            if resp_nonce != nonce {
                return Err(DiscordIpcError::InvalidResponse(format!(
                    "Nonce mismatch: expected {}, got {}",
                    nonce, resp_nonce
                )));
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
        // Generate a unique nonce
        let nonce = format!(
            "clear-activity-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        let message = IpcMessage {
            cmd: Command::SetActivity,
            args: json!({
                "pid": process::id(),
                "activity": Value::Null
            }),
            nonce: nonce.clone(),
        };

        let payload = serde_json::to_value(message)?;
        self.send_message(Opcode::Frame, &payload).await?;

        let (opcode, response) = self.recv_message().await?;
        debug_println!("Clear Activity response: {}", response);

        // Check if we got the correct response type
        if !opcode.is_frame_response() {
            return Err(DiscordIpcError::InvalidResponse(format!(
                "Expected frame response, got {:?}",
                opcode
            )));
        }

        // Check for error in the response
        if let Some(err) = response.get("error") {
            if let (Some(code), Some(message)) = (
                err.get("code").and_then(|c| c.as_i64()),
                err.get("message").and_then(|m| m.as_str()),
            ) {
                return Err(DiscordIpcError::discord_error(code as i32, message));
            } else {
                return Err(DiscordIpcError::InvalidResponse(format!(
                    "Invalid error format in response: {}",
                    err
                )));
            }
        }

        // Verify nonce matches to ensure we got the right response
        if let Some(resp_nonce) = response.get("nonce").and_then(|n| n.as_str()) {
            if resp_nonce != nonce {
                return Err(DiscordIpcError::InvalidResponse(format!(
                    "Nonce mismatch: expected {}, got {}",
                    nonce, resp_nonce
                )));
            }
        }

        Ok(response)
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

        // Write header
        write_u32_le(&mut self.connection, opcode.into()).await?;
        write_u32_le(&mut self.connection, raw.len() as u32).await?;

        // Write payload
        write_all(&mut self.connection, &raw).await?;

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
        // Read header
        let opcode_raw = read_u32_le(&mut self.connection).await?;
        let length = read_u32_le(&mut self.connection).await?;

        let opcode = Opcode::from(opcode_raw);

        // Read payload
        let mut data = vec![0u8; length as usize];
        read_exact(&mut self.connection, &mut data)
            .await
            .map_err(|_| DiscordIpcError::SocketClosed)?;

        let value: Value = serde_json::from_slice(&data)?;

        Ok((opcode, value))
    }
}
