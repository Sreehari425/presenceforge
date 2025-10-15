use serde_json::{json, Value};
use std::collections::VecDeque;
use std::io::Write;
use std::process;
use std::time::{Duration, Instant};

use crate::activity::Activity;
use crate::debug_println;
use crate::error::{DiscordIpcError, Result};
use crate::ipc::{
    constants, Command, HandshakePayload, IpcConnection, IpcMessage, Opcode, PipeConfig,
};
use crate::nonce::generate_nonce;

/// Discord IPC Client
pub struct DiscordIpcClient {
    client_id: String,
    connection: IpcConnection,
    pending_messages: VecDeque<PendingMessage>,
}

impl DiscordIpcClient {
    /// Create a new Discord IPC client (uses auto-discovery)
    pub fn new<S: Into<String>>(client_id: S) -> Result<Self> {
        Self::new_with_config(client_id, None)
    }

    /// Create a new Discord IPC client with pipe configuration
    ///
    /// # Arguments
    ///
    /// * `client_id` - The Discord application client ID
    /// * `config` - Optional pipe configuration. If `None`, auto-discovery is used.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use presenceforge::{DiscordIpcClient, PipeConfig};
    ///
    /// // Auto-discovery (default)
    /// let client = DiscordIpcClient::new_with_config("client_id", None)?;
    ///
    /// // Connect to custom path
    /// let client = DiscordIpcClient::new_with_config(
    ///     "client_id",
    ///     Some(PipeConfig::CustomPath("/tmp/discord-ipc-0".to_string()))
    /// )?;
    /// # Ok::<(), presenceforge::DiscordIpcError>(())
    /// ```
    pub fn new_with_config<S: Into<String>>(
        client_id: S,
        config: Option<PipeConfig>,
    ) -> Result<Self> {
        let client_id = client_id.into();
        let connection = IpcConnection::new_with_config(config)?;

        Ok(Self {
            client_id,
            connection,
            pending_messages: VecDeque::new(),
        })
    }

    /// Create a new Discord IPC client with a connection timeout (uses auto-discovery)
    ///
    /// # Arguments
    ///
    /// * `client_id` - The Discord application client ID
    /// * `timeout_ms` - Connection timeout in milliseconds
    ///
    /// # Returns
    ///
    /// A new Discord IPC client
    ///
    /// # Errors
    ///
    /// Returns a `DiscordIpcError::ConnectionTimeout` if the connection times out
    pub fn new_with_timeout<S: Into<String>>(client_id: S, timeout_ms: u64) -> Result<Self> {
        Self::new_with_config_and_timeout(client_id, None, timeout_ms)
    }

    /// Create a new Discord IPC client with pipe configuration and timeout
    ///
    /// # Arguments
    ///
    /// * `client_id` - The Discord application client ID
    /// * `config` - Optional pipe configuration. If `None`, auto-discovery is used.
    /// * `timeout_ms` - Connection timeout in milliseconds
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use presenceforge::{DiscordIpcClient, PipeConfig};
    ///
    /// // Auto-discovery with timeout
    /// let client = DiscordIpcClient::new_with_config_and_timeout("client_id", None, 5000)?;
    ///
    /// // Custom pipe path with timeout
    /// let client = DiscordIpcClient::new_with_config_and_timeout(
    ///     "client_id",
    ///     Some(PipeConfig::CustomPath("/tmp/discord-ipc-0".to_string())),
    ///     5000
    /// )?;
    /// # Ok::<(), presenceforge::DiscordIpcError>(())
    /// ```
    pub fn new_with_config_and_timeout<S: Into<String>>(
        client_id: S,
        config: Option<PipeConfig>,
        timeout_ms: u64,
    ) -> Result<Self> {
        let client_id = client_id.into();
        let connection = IpcConnection::new_with_config_and_timeout(config, timeout_ms)?;

        Ok(Self {
            client_id,
            connection,
            pending_messages: VecDeque::new(),
        })
    }

    /// Perform handshake with Discord
    ///
    /// # Returns
    ///
    /// The Discord handshake response as a JSON Value
    ///
    /// # Errors
    ///
    /// Returns a `DiscordIpcError::HandshakeFailed` if the handshake fails
    pub fn connect(&mut self) -> Result<Value> {
        self.pending_messages.clear();

        let handshake = HandshakePayload {
            v: constants::IPC_VERSION,
            client_id: self.client_id.clone(),
        };

        let payload =
            serde_json::to_value(handshake).map_err(DiscordIpcError::SerializationFailed)?;

        self.connection.send(Opcode::Handshake, &payload)?;

        let (opcode, response) = self.connection.recv()?;
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

    /// Set Discord Rich Presence activity
    ///
    /// # Arguments
    ///
    /// * `activity` - The activity to set
    ///
    /// # Errors
    ///
    /// Returns a `DiscordIpcError` if serialization fails or if Discord returns an error
    pub fn set_activity(&mut self, activity: &Activity) -> Result {
        // Validate the activity first
        if let Err(reason) = activity.validate() {
            return Err(DiscordIpcError::InvalidActivity(reason));
        }

        // Generate a cryptographically secure unique nonce for this request
        let nonce = generate_nonce("set-activity");

        let message = IpcMessage {
            cmd: Command::SetActivity,
            args: json!({
                "pid": process::id(),
                "activity": activity
            }),
            nonce: nonce.clone(),
        };
        let payload = serde_json::to_value(message)?;
        // debug_println!("[IPC_MESSAGE] : {:?} ", payload);
        // std::io::stdout().flush().unwrap();
        self.connection.send(Opcode::Frame, &payload)?;

        // Receive the response to check for errors
        let (opcode, response) = self.recv_for_nonce(&nonce)?;

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

    /// Clear Discord Rich Presence activity
    ///
    /// # Returns
    ///
    /// The response from Discord as a JSON Value
    ///
    /// # Errors
    ///
    /// Returns a `DiscordIpcError` if communication fails or if Discord returns an error
    pub fn clear_activity(&mut self) -> Result<Value> {
        // Generate a cryptographically secure unique nonce
        let nonce = generate_nonce("clear-activity");

        let message = IpcMessage {
            cmd: Command::SetActivity,
            args: json!({
                "pid": process::id(),
                "activity": Value::Null
            }),
            nonce: nonce.clone(),
        };

        let payload = serde_json::to_value(message)?;
        self.connection.send(Opcode::Frame, &payload)?;

        let (opcode, response) = self.recv_for_nonce(&nonce)?;
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

    /// Send a raw IPC message
    pub fn send_message(&mut self, opcode: Opcode, payload: &Value) -> Result {
        self.connection.send(opcode, payload)
    }

    /// Receive a raw IPC message
    pub fn recv_message(&mut self) -> Result<(Opcode, Value)> {
        self.next_message()
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

    /// Close the connection
    pub fn close(&mut self) {
        self.connection.close();
        self.pending_messages.clear();
    }
}

impl Drop for DiscordIpcClient {
    fn drop(&mut self) {
        self.close();
    }
}

impl DiscordIpcClient {
    fn next_message(&mut self) -> Result<(Opcode, Value)> {
        if let Some(message) = self.pending_messages.pop_front() {
            let PendingMessage {
                opcode, payload, ..
            } = message;
            return Ok((opcode, payload));
        }

        self.connection.recv()
    }

    fn recv_for_nonce(&mut self, expected_nonce: &str) -> Result<(Opcode, Value)> {
        if let Some(message) = self.take_pending_by_nonce(expected_nonce) {
            return Ok(message);
        }

        loop {
            let (opcode, response) = self.connection.recv()?;
            if Self::value_has_nonce(&response, expected_nonce) {
                return Ok((opcode, response));
            }

            self.pending_messages
                .push_back(PendingMessage::new(opcode, response));
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_has_nonce_detects_match() {
        let payload = serde_json::json!({
            "nonce": "abc",
            "data": {}
        });

        assert!(DiscordIpcClient::value_has_nonce(&payload, "abc"));
        assert!(!DiscordIpcClient::value_has_nonce(&payload, "def"));
    }

    #[test]
    fn value_has_nonce_handles_missing_field() {
        let payload = serde_json::json!({ "data": 1 });
        assert!(!DiscordIpcClient::value_has_nonce(&payload, "anything"));
    }

    #[test]
    fn pending_message_records_creation_time() {
        let message = PendingMessage::new(Opcode::Frame, serde_json::json!({"nonce": "1"}));
        let elapsed = Instant::now().saturating_duration_since(message.received_at);
        assert!(elapsed.as_secs() < 1);
    }
}
