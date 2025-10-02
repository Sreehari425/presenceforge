use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde_json::Value;
use std::io::Read;

#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(windows)]
use std::io::{BufReader, BufWriter};

use crate::error::{DiscordIpcError, Result};
use crate::ipc::protocol::{constants, Opcode};

#[cfg(unix)]
pub struct IpcConnection {
    stream: UnixStream,
}

#[cfg(windows)]
pub struct IpcConnection {
    reader: BufReader<std::fs::File>,
    writer: BufWriter<std::fs::File>,
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
            let (reader, writer) = Self::connect_to_discord_windows()?;
            Ok(Self { reader, writer })
        }
    }

    /// Create a new IPC connection with a timeout
    pub fn new_with_timeout(timeout_ms: u64) -> Result<Self> {
        use std::time::{Duration, Instant};

        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        // Keep trying to connect until we succeed or timeout
        while start.elapsed() < timeout {
            match Self::try_connect() {
                Ok(connection) => return Ok(connection),
                Err(DiscordIpcError::NoValidSocket) => {
                    // Wait a bit before trying again
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(DiscordIpcError::ConnectionTimeout(timeout_ms))
    }

    /// Try to connect to Discord
    fn try_connect() -> Result<Self> {
        #[cfg(unix)]
        {
            let stream = Self::connect_to_discord_unix()?;
            Ok(Self { stream })
        }

        #[cfg(windows)]
        {
            let (reader, writer) = Self::connect_to_discord_windows()?;
            Ok(Self { reader, writer })
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
        let mut last_error = None;

        for dir in &directories {
            for i in 0..constants::MAX_IPC_SOCKETS {
                let socket_path = format!("{}/{}{}", dir, constants::IPC_SOCKET_PREFIX, i);

                match UnixStream::connect(&socket_path) {
                    Ok(stream) => {
                        // Configure socket
                        if let Err(err) = stream.set_nonblocking(false) {
                            last_error = Some(err);
                            continue;
                        }

                        return Ok(stream);
                    }
                    Err(err) => {
                        last_error = Some(err);
                        continue;
                    }
                }
            }
        }

        // If we got here, no valid socket was found
        if let Some(err) = last_error {
            // Return the last error we encountered for diagnostic purposes
            if err.kind() == std::io::ErrorKind::PermissionDenied {
                Err(DiscordIpcError::ConnectionFailed(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Permission denied when connecting to Discord IPC socket. Check file permissions."
                )))
            } else {
                Err(DiscordIpcError::ConnectionFailed(err))
            }
        } else {
            Err(DiscordIpcError::NoValidSocket)
        }
    }

    #[cfg(windows)]
    /// Connect to Discord IPC named pipe on Windows
    fn connect_to_discord_windows() -> Result<(BufReader<std::fs::File>, BufWriter<std::fs::File>)>
    {
        let mut last_error = None;

        for i in 0..constants::MAX_IPC_SOCKETS {
            let pipe_path = format!(r"\\?\pipe\discord-ipc-{}", i);

            // Try to open the named pipe
            match OpenOptions::new().read(true).write(true).open(&pipe_path) {
                Ok(file) => {
                    // Clone the file handle for reader and writer
                    match file.try_clone() {
                        Ok(reader_file) => {
                            let writer_file = file;
                            return Ok((BufReader::new(reader_file), BufWriter::new(writer_file)));
                        }
                        Err(err) => {
                            last_error = Some(err);
                            continue;
                        }
                    }
                }
                Err(err) => {
                    last_error = Some(err);
                    continue; // Try next pipe number
                }
            }
        }

        // If we got here, no valid pipe was found
        if let Some(err) = last_error {
            // Return the last error we encountered for diagnostic purposes
            if err.kind() == std::io::ErrorKind::PermissionDenied {
                Err(DiscordIpcError::ConnectionFailed(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Permission denied when connecting to Discord IPC pipe. Is Discord running with the right permissions?"
                )))
            } else {
                Err(DiscordIpcError::ConnectionFailed(err))
            }
        } else {
            Err(DiscordIpcError::NoValidSocket)
        }
    }

    /// Send data with opcode
    pub fn send(&mut self, opcode: Opcode, payload: &Value) -> Result<()> {
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
            self.writer.write_all(&buffer)?;
            self.writer.flush()?;
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
            self.reader
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
            self.reader
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
