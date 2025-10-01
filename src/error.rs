use std::io;
use thiserror::Error;

/// Errors that can occur during Discord IPC operations
#[derive(Error, Debug)]
pub enum DiscordIpcError {
    #[error("Failed to connect to Discord IPC socket")]
    ConnectionFailed(#[from] io::Error),

    #[error("Failed to serialize JSON payload")]
    SerializationFailed(#[from] serde_json::Error),

    #[error("Invalid response from Discord")]
    InvalidResponse,

    #[error("Handshake failed")]
    HandshakeFailed,

    #[error("Socket connection was closed")]
    SocketClosed,

    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u32),
}

/// Result type for Discord IPC operations
pub type Result<T> = std::result::Result<T, DiscordIpcError>;
