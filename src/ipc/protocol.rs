use crate::error::{DiscordIpcError, ProtocolContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Discord IPC Opcodes
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    Handshake = 0,
    Frame = 1,
    Close = 2,
    Ping = 3,
    Pong = 4,
}

impl Opcode {
    /// Check if this opcode is a response to a handshake
    /// In Discord IPC protocol, handshake responses actually use the Frame opcode (1)
    pub fn is_handshake_response(&self) -> bool {
        *self == Opcode::Frame
    }

    /// Check if this opcode is a response to a frame
    pub fn is_frame_response(&self) -> bool {
        *self == Opcode::Frame
    }
}

impl TryFrom<u32> for Opcode {
    type Error = DiscordIpcError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Opcode::Handshake),
            1 => Ok(Opcode::Frame),
            2 => Ok(Opcode::Close),
            3 => Ok(Opcode::Ping),
            4 => Ok(Opcode::Pong),
            _ => {
                let context = ProtocolContext {
                    expected_opcode: None,
                    received_opcode: Some(value),
                    payload_size: None,
                };
                Err(DiscordIpcError::protocol_violation(
                    format!("Invalid opcode value: {}", value),
                    context,
                ))
            }
        }
    }
}

impl From<Opcode> for u32 {
    fn from(opcode: Opcode) -> Self {
        opcode as u32
    }
}

/// Discord IPC Commands
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Command {
    SetActivity,
    Subscribe,
    Unsubscribe,
}

/// Discord IPC Message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    pub cmd: Command,
    pub args: Value,
    pub nonce: String,
}

/// Handshake payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakePayload {
    pub v: u32,
    pub client_id: String,
}

/// Response from Discord IPC
#[derive(Debug, Clone, Deserialize)]
pub struct IpcResponse {
    pub cmd: Option<String>,
    pub data: Option<Value>,
    pub evt: Option<String>,
    pub nonce: Option<String>,
}

/// Constants and configuration for Discord IPC protocol
pub mod constants {
    /// Discord IPC protocol version
    ///
    /// This is the version number sent during the handshake process.
    /// Discord currently uses version 1 for its IPC protocol.
    /// This should match the version Discord expects.
    pub const IPC_VERSION: u32 = 1;

    /// Maximum number of IPC socket/pipe instances to scan
    ///
    /// Discord creates numbered IPC sockets from 0 to 9 (discord-ipc-0 through discord-ipc-9).
    /// Each socket represents a potential Discord client instance.
    /// Value of 10 allows checking all possible Discord instances.
    ///
    /// # Background
    /// - Discord can run multiple instances (PTB, Canary, Stable)
    /// - Each instance may use a different socket number
    /// - The official Discord RPC client scans up to 10 sockets
    ///
    /// # Platform Notes
    /// - Unix: Named sockets in XDG_RUNTIME_DIR or /tmp
    /// - Windows: Named pipes (\\\\.\\pipe\\discord-ipc-N)
    pub const MAX_IPC_SOCKETS: u8 = 10;

    /// IPC socket name prefix used for socket discovery
    ///
    /// Discord IPC sockets follow the naming pattern: `discord-ipc-{N}`
    /// where N is a number from 0 to MAX_IPC_SOCKETS-1.
    pub const IPC_SOCKET_PREFIX: &str = "discord-ipc-";

    /// Default connection retry interval in milliseconds
    ///
    /// When auto-discovery fails to find an available socket,
    /// the connection attempt waits this amount of time before retrying.
    /// 100ms provides a good balance between responsiveness and CPU usage.
    pub const DEFAULT_RETRY_INTERVAL_MS: u64 = 100;

    /// Maximum size for IPC payload data (16 MB)
    ///
    /// Discord IPC messages should not exceed this size.
    /// This is a safety limit to prevent excessive memory allocation
    /// from malformed or malicious data.
    ///
    /// # Note
    /// Typical Discord Rich Presence payloads are less than 1 KB.
    /// This limit is intentionally generous for future compatibility.
    pub const MAX_PAYLOAD_SIZE: u32 = 16 * 1024 * 1024;

    /// Size of the IPC message header in bytes
    ///
    /// Discord IPC protocol uses an 8-byte header:
    /// - 4 bytes: Opcode (u32, little-endian)
    /// - 4 bytes: Payload length (u32, little-endian)
    pub const IPC_HEADER_SIZE: usize = 8;
}

/// Configuration for Discord IPC protocol behavior
///
/// Allows customization of protocol parameters for different Discord setups
/// or special use cases (testing, non-standard installations, etc.)
#[derive(Debug, Clone)]
pub struct IpcConfig {
    /// Maximum number of socket instances to scan during auto-discovery
    pub max_sockets: u8,

    /// Retry interval in milliseconds when connection fails
    pub retry_interval_ms: u64,

    /// Maximum allowed payload size in bytes
    pub max_payload_size: u32,

    /// IPC protocol version to use in handshake
    pub ipc_version: u32,
}

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            max_sockets: constants::MAX_IPC_SOCKETS,
            retry_interval_ms: constants::DEFAULT_RETRY_INTERVAL_MS,
            max_payload_size: constants::MAX_PAYLOAD_SIZE,
            ipc_version: constants::IPC_VERSION,
        }
    }
}

impl IpcConfig {
    /// Create a new IpcConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration optimized for faster connection attempts
    ///
    /// Useful when you know Discord is running and want quick connection.
    /// Reduces retry interval and socket scan range.
    pub fn fast_connect() -> Self {
        Self {
            max_sockets: 3,        // Only check first 3 sockets
            retry_interval_ms: 50, // Retry faster
            ..Default::default()
        }
    }

    /// Create a configuration for testing or non-standard Discord installations
    ///
    /// Allows scanning more socket instances and has longer retry intervals
    /// for slower systems or unusual configurations.
    pub fn extended() -> Self {
        Self {
            max_sockets: 10,
            retry_interval_ms: 200,
            ..Default::default()
        }
    }

    /// Set the maximum number of sockets to scan
    pub fn with_max_sockets(mut self, max_sockets: u8) -> Self {
        self.max_sockets = max_sockets;
        self
    }

    /// Set the retry interval in milliseconds
    pub fn with_retry_interval(mut self, retry_interval_ms: u64) -> Self {
        self.retry_interval_ms = retry_interval_ms;
        self
    }

    /// Set the maximum payload size in bytes
    pub fn with_max_payload_size(mut self, max_payload_size: u32) -> Self {
        self.max_payload_size = max_payload_size;
        self
    }

    /// Validate the configuration
    ///
    /// Returns true if all parameters are within acceptable ranges
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.max_sockets == 0 {
            return Err("max_sockets must be greater than 0");
        }
        if self.max_sockets > 100 {
            return Err("max_sockets exceeds reasonable limit (100)");
        }
        if self.retry_interval_ms == 0 {
            return Err("retry_interval_ms must be greater than 0");
        }
        if self.retry_interval_ms > 10_000 {
            return Err("retry_interval_ms exceeds reasonable limit (10 seconds)");
        }
        if self.max_payload_size < 1024 {
            return Err("max_payload_size too small (minimum 1 KB)");
        }
        if self.max_payload_size > 100 * 1024 * 1024 {
            return Err("max_payload_size too large (maximum 100 MB)");
        }
        Ok(())
    }
}
