use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde_json::Value;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use crate::error::{DiscordIpcError, Result};
use crate::ipc::protocol::{Opcode, constants};

/// Discord-IPC connection handler
pub struct IpcConnection {
    stream: UnixStream,
}

impl IpcConnection {
    /// Create a new IPC connection
    pub fn new() -> Result<Self> {
        let stream = Self::connect_to_discord()?;
        Ok(Self { stream })
    }

    /// Connect to Discord IPC socket - checks multiple possible directories
    fn connect_to_discord() -> Result<UnixStream> {
        // Try environment variables in order of preference
        let env_keys = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"];
        let mut directories = Vec::new();

        for env_key in &env_keys {
            if let Ok(dir) = std::env::var(env_key) {
                directories.push(dir);
            }
        }

        // Fallback to /run/user/{uid} if no env vars found

        if directories.is_empty() {
            directories.push(format!("/run/user/{}", unsafe { libc::getuid() }));
        }

        // Try each directory with each socket number
        for dir in &directories {
            for i in 0..constants::MAX_IPC_SOCKETS {
                let socket_path = format!("{}/{}{}", dir, constants::IPC_SOCKET_PREFIX, i);

                if let Ok(stream) = UnixStream::connect(&socket_path) {
                    return Ok(stream);
                }
            }
        }

        Err(DiscordIpcError::ConnectionFailed(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No Discord IPC socket found in any of the expected directories",
        )))
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
        self.stream
            .read_exact(&mut header)
            .map_err(|_| DiscordIpcError::SocketClosed)?;

        let mut header_reader = &header[..];
        let opcode_raw = header_reader.read_u32::<LittleEndian>()?;
        let length = header_reader.read_u32::<LittleEndian>()?;

        let opcode = Opcode::from(opcode_raw);

        let mut data = vec![0u8; length as usize];
        self.stream
            .read_exact(&mut data)
            .map_err(|_| DiscordIpcError::SocketClosed)?;

        let value: Value = serde_json::from_slice(&data)?;
        Ok((opcode, value))
    }

    /// Close the connection
    pub fn close(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}
