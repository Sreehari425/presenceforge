use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Discord IPC Opcodes
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Handshake = 0,
    Frame = 1,
    Close = 2,
    Ping = 3,
    Pong = 4,
}

impl From<u32> for Opcode {
    fn from(value: u32) -> Self {
        match value {
            0 => Opcode::Handshake,
            1 => Opcode::Frame,
            2 => Opcode::Close,
            3 => Opcode::Ping,
            4 => Opcode::Pong,
            _ => panic!("Invalid opcode: {value}"),
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

/// Constants for Discord IPC
pub mod constants {
    /// IPC version
    pub const IPC_VERSION: u32 = 1;

    /// Maximum IPC socket attempts
    pub const MAX_IPC_SOCKETS: u8 = 10;

    /// IPC socket name prefix
    pub const IPC_SOCKET_PREFIX: &str = "discord-ipc-";
}
