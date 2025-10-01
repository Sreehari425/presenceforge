use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde_json::Value;

use crate::error::{DiscordIpcError, Result};
use crate::ipc::protocol::{Opcode, constants};

/// Low-level Discord IPC connection handler
pub struct IpcConnection {
    stream: UnixStream,
}

impl IpcConnection {
    /// Create a new IPC connection
    pub fn new() -> Result<Self> {
        let stream = Self::connect_to_discord()?;
        Ok(Self { stream })
    }

    /// Connect to Discord IPC socket
    fn connect_to_discord() -> Result<UnixStream> {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));

        for i in 0..constants::MAX_IPC_SOCKETS {
            let socket_path = format!("{}/{}{}", runtime_dir, constants::IPC_SOCKET_PREFIX, i);
            
            if let Ok(stream) = UnixStream::connect(&socket_path) {
                return Ok(stream);
            }
        }
        
        Err(DiscordIpcError::ConnectionFailed(
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No Discord IPC socket found"
            )
        ))
    }

    /// Send data with opcode
    pub fn send(&mut self, opcode: Opcode, payload: &Value) -> Result<()> {
        let raw = serde_json::to_vec(payload)?;
        let mut buffer = Vec::with_capacity(8 + raw.len());
        
        buffer.write_u32::<LittleEndian>(opcode.into())?;
        buffer.write_u32::<LittleEndian>(raw.len() as u32)?;
        buffer.extend_from_slice(&raw);
        
        self.stream.write_all(&buffer)?;
        Ok(())
    }

    /// Receive data and return opcode and payload
    pub fn recv(&mut self) -> Result<(Opcode, Value)> {
        let mut header = [0u8; 8];
        self.stream.read_exact(&mut header)
            .map_err(|_| DiscordIpcError::SocketClosed)?;
        
        let mut header_reader = &header[..];
        let opcode_raw = header_reader.read_u32::<LittleEndian>()?;
        let length = header_reader.read_u32::<LittleEndian>()?;
        
        let opcode = Opcode::from(opcode_raw);
        
        let mut data = vec![0u8; length as usize];
        self.stream.read_exact(&mut data)
            .map_err(|_| DiscordIpcError::SocketClosed)?;
        
        let value: Value = serde_json::from_slice(&data)?;
        Ok((opcode, value))
    }

    /// Close the connection
    pub fn close(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}