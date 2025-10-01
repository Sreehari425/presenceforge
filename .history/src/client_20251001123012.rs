use std::process;
use serde_json::{json, Value};

use crate::error::Result;
use crate::ipc::{IpcConnection, Opcode, HandshakePayload, IpcMessage, Command, constants};
use crate::activity::Activity;

/// Discord IPC Client
pub struct DiscordIpcClient {
    client_id: String,
    connection: IpcConnection,
}

impl DiscordIpcClient {
    /// Create a new Discord IPC client
    pub fn new<S: Into<String>>(client_id: S) -> Result<Self> {
        let client_id = client_id.into();
        let connection = IpcConnection::new()?;
        
        Ok(Self {
            client_id,
            connection,
        })
    }

    /// Perform handshake with Discord
    pub fn connect(&mut self) -> Result<Value> {
        let handshake = HandshakePayload {
            v: constants::IPC_VERSION,
            client_id: self.client_id.clone(),
        };
        
        let payload = serde_json::to_value(handshake)?;
        self.connection.send(Opcode::Handshake, &payload)?;
        
        let (_opcode, response) = self.connection.recv()?;
        println!("Handshake response: {}", response);
        Ok(response)
    }

    /// Set Discord Rich Presence activity
    pub fn set_activity(&mut self, activity: &Activity) -> Result<Value> {
        let message = IpcMessage {
            cmd: Command::SetActivity,
            args: json!({
                "pid": process::id(),
                "activity": activity
            }),
            nonce: "set_activity_nonce".to_string(),
        };
        
        let payload = serde_json::to_value(message)?;
        self.connection.send(Opcode::Frame, &payload)?;
        
        let (_opcode, response) = self.connection.recv()?;
        println!("Set Activity response: {}", response);
        Ok(response)
    }

    /// Clear Discord Rich Presence activity
    pub fn clear_activity(&mut self) -> Result<Value> {
        let message = IpcMessage {
            cmd: Command::SetActivity,
            args: json!({
                "pid": process::id(),
                "activity": Value::Null
            }),
            nonce: "clear_activity_nonce".to_string(),
        };
        
        let payload = serde_json::to_value(message)?;
        self.connection.send(Opcode::Frame, &payload)?;
        
        let (_opcode, response) = self.connection.recv()?;
        println!("Clear Activity response: {}", response);
        Ok(response)
    }

    /// Send a raw IPC message
    pub fn send_message(&mut self, opcode: Opcode, payload: &Value) -> Result<()> {
        self.connection.send(opcode, payload)
    }

    /// Receive a raw IPC message
    pub fn recv_message(&mut self) -> Result<(Opcode, Value)> {
        self.connection.recv()
    }

    /// Close the connection
    pub fn close(&mut self) {
        self.connection.close();
    }
}

impl Drop for DiscordIpcClient {
    fn drop(&mut self) {
        self.close();
    }
}