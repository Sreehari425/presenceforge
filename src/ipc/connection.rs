use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde_json::Value;
use std::io::Read;

#[cfg(unix)]
use std::{io::Write, os::unix::net::UnixStream};

#[cfg(windows)]
use std::fs::{File, OpenOptions};

use crate::error::{DiscordIpcError, Result};
use crate::ipc::protocol::{constants, Opcode};

#[cfg(unix)]
pub struct IpcConnection {
    stream: UnixStream,
}

#[cfg(windows)]
pub struct IpcConnection {
    file: File,
}

impl IpcConnection {
    /// Create a new IPC connection
    pub fn new() -> Result<Self> {
        #[cfg(unix)]
        {
            let stream = Self::connect_to_discord_unix()?;
            Ok(Self { stream })
        }

        #[cfg(windows)]
        {
            let file = Self::connect_to_discord_windows()?;
            Ok(Self { file })
        }
    }

    #[cfg(unix)]
    /// Connect to Discord IPC socket on Unix systems
    fn connect_to_discord_unix() -> Result<UnixStream> {
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

    #[cfg(windows)]
    /// Connect to Discord IPC named pipe on Windows
    fn connect_to_discord_windows() -> Result<File> {
        for i in 0..constants::MAX_IPC_SOCKETS {
            let pipe_path = format!(r"\\?\pipe\discord-ipc-{i}");

            if let Ok(file) = OpenOptions::new().read(true).write(true).open(&pipe_path) {
                return Ok(file);
            }
        }

        Err(DiscordIpcError::ConnectionFailed(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No Discord IPC named pipe found",
        )))
    }

    /// Send data with opcode
    pub fn send(&mut self, opcode: Opcode, payload: &Value) -> Result {
        let raw = serde_json::to_vec(payload)?;
        let mut buffer = Vec::with_capacity(8 + raw.len());

        buffer.write_u32::<LittleEndian>(opcode.into())?;
        buffer.write_u32::<LittleEndian>(raw.len() as u32)?;
        buffer.extend_from_slice(&raw);

        #[cfg(unix)]
        {
            use std::io::Write;
            self.stream.write_all(&buffer)?;
        }

        #[cfg(windows)]
        {
            use std::io::Write;
            self.file.write_all(&buffer)?;
        }

        Ok(())
    }

    /// Receive data and return opcode and payload
    pub fn recv(&mut self) -> Result<(Opcode, Value)> {
        let mut header = [0u8; 8];

        #[cfg(unix)]
        {
            self.stream
                .read_exact(&mut header)
                .map_err(|_| DiscordIpcError::SocketClosed)?;
        }

        #[cfg(windows)]
        {
            self.file
                .read_exact(&mut header)
                .map_err(|_| DiscordIpcError::SocketClosed)?;
        }

        let mut header_reader = &header[..];
        let opcode_raw = header_reader.read_u32::<LittleEndian>()?;
        let length = header_reader.read_u32::<LittleEndian>()?;

        let opcode = Opcode::from(opcode_raw);

        let mut data = vec![0u8; length as usize];

        #[cfg(unix)]
        {
            self.stream
                .read_exact(&mut data)
                .map_err(|_| DiscordIpcError::SocketClosed)?;
        }

        #[cfg(windows)]
        {
            self.file
                .read_exact(&mut data)
                .map_err(|_| DiscordIpcError::SocketClosed)?;
        }

        let value: Value = serde_json::from_slice(&data)?;
        Ok((opcode, value))
    }

    /// Close the connection
    pub fn close(&mut self) {
        #[cfg(unix)]
        {
            let _ = self.stream.shutdown(std::net::Shutdown::Both);
        }

        #[cfg(windows)]
        {
            // Windows named pipes don't need explicit shutdown
            // Files will be closed when dropped
        }
    }
}
