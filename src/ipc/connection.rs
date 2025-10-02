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

/// Configuration for selecting which Discord IPC pipe to connect to
#[derive(Debug, Clone)]
pub enum PipeConfig {
    /// Automatically discover and connect to the first available pipe (default behavior)
    Auto,
    /// Connect to a custom pipe path
    ///
    /// # Examples
    ///
    /// Unix: `/run/user/1000/discord-ipc-0` or `/run/user/1000/app/com.discordapp.Discord/discord-ipc-0`
    ///
    /// Windows: `\\.\pipe\discord-ipc-0`
    CustomPath(String),
}

impl Default for PipeConfig {
    fn default() -> Self {
        PipeConfig::Auto
    }
}

/// Information about a discovered Discord IPC pipe
#[derive(Debug, Clone)]
pub struct DiscoveredPipe {
    /// The pipe number (0-9)
    pub pipe_number: u8,
    /// The full path to the pipe
    pub path: String,
}

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
    /// Discover all available Discord IPC pipes
    ///
    /// Returns a list of all Discord IPC pipes that are currently accessible
    ///
    /// # Example
    ///
    /// ```no_run
    /// use presenceforge::ipc::IpcConnection;
    ///
    /// let pipes = IpcConnection::discover_pipes();
    /// for pipe in pipes {
    ///     println!("Found pipe {}: {}", pipe.pipe_number, pipe.path);
    /// }
    /// ```
    pub fn discover_pipes() -> Vec<DiscoveredPipe> {
        #[cfg(unix)]
        {
            Self::discover_pipes_unix()
        }

        #[cfg(windows)]
        {
            Self::discover_pipes_windows()
        }
    }

    #[cfg(unix)]
    fn discover_pipes_unix() -> Vec<DiscoveredPipe> {
        let mut pipes = Vec::new();

        // Try environment variables in order of preference
        let env_keys = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"];
        let mut directories = Vec::new();

        for env_key in &env_keys {
            if let Ok(dir) = std::env::var(env_key) {
                directories.push(dir.clone());

                // Also check Flatpak Discord path if XDG_RUNTIME_DIR is set
                if env_key == &"XDG_RUNTIME_DIR" {
                    directories.push(format!("{}/app/com.discordapp.Discord", dir));
                }
            }
        }

        // Fallback to /run/user/{uid} if no env vars found
        if directories.is_empty() {
            let uid = unsafe { libc::getuid() };
            directories.push(format!("/run/user/{}", uid));
            // Also try Flatpak path as fallback
            directories.push(format!("/run/user/{}/app/com.discordapp.Discord", uid));
        }

        // Try each directory with each socket number
        for dir in &directories {
            for i in 0..constants::MAX_IPC_SOCKETS {
                let socket_path = format!("{}/{}{}", dir, constants::IPC_SOCKET_PREFIX, i);

                // Check if we can connect to this socket
                if let Ok(stream) = UnixStream::connect(&socket_path) {
                    drop(stream); // Close the test connection
                    pipes.push(DiscoveredPipe {
                        pipe_number: i,
                        path: socket_path,
                    });
                }
            }
        }

        pipes
    }

    #[cfg(windows)]
    fn discover_pipes_windows() -> Vec<DiscoveredPipe> {
        let mut pipes = Vec::new();

        for i in 0..constants::MAX_IPC_SOCKETS {
            let pipe_path = format!(r"\\?\pipe\discord-ipc-{}", i);

            // Try to open the named pipe to check if it exists
            if let Ok(file) = OpenOptions::new().read(true).write(true).open(&pipe_path) {
                drop(file); // Close the test connection
                pipes.push(DiscoveredPipe {
                    pipe_number: i,
                    path: pipe_path,
                });
            }
        }

        pipes
    }

    /// Create a new IPC connection with optional pipe configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Optional pipe configuration. If `None`, auto-discovery is used.
    pub fn new_with_config(config: Option<PipeConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();

        #[cfg(unix)]
        {
            let stream = Self::connect_to_discord_unix_with_config(&config)?;
            Ok(Self { stream })
        }

        #[cfg(windows)]
        {
            let (reader, writer) = Self::connect_to_discord_windows_with_config(&config)?;
            Ok(Self { reader, writer })
        }
    }

    /// Create a new IPC connection (uses auto-discovery)
    pub fn new() -> Result<Self> {
        Self::new_with_config(None)
    }

    /// Create a new IPC connection with a timeout
    pub fn new_with_timeout(timeout_ms: u64) -> Result<Self> {
        Self::new_with_config_and_timeout(None, timeout_ms)
    }

    /// Create a new IPC connection with optional pipe configuration and timeout
    ///
    /// # Arguments
    ///
    /// * `config` - Optional pipe configuration. If `None`, auto-discovery is used.
    /// * `timeout_ms` - Connection timeout in milliseconds
    pub fn new_with_config_and_timeout(
        config: Option<PipeConfig>,
        timeout_ms: u64,
    ) -> Result<Self> {
        use std::time::{Duration, Instant};

        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);
        let config = config.unwrap_or_default();

        // Keep trying to connect until we succeed or timeout
        while start.elapsed() < timeout {
            match Self::try_connect_with_config(&config) {
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

    /// Try to connect to Discord with configuration
    fn try_connect_with_config(config: &PipeConfig) -> Result<Self> {
        #[cfg(unix)]
        {
            let stream = Self::connect_to_discord_unix_with_config(config)?;
            Ok(Self { stream })
        }

        #[cfg(windows)]
        {
            let (reader, writer) = Self::connect_to_discord_windows_with_config(config)?;
            Ok(Self { reader, writer })
        }
    }

    #[cfg(unix)]
    /// Connect to Discord IPC socket on Unix systems with configuration
    fn connect_to_discord_unix_with_config(config: &PipeConfig) -> Result<UnixStream> {
        match config {
            PipeConfig::Auto => {
                // Auto-discovery: try all possible pipes
                Self::connect_to_discord_unix_auto()
            }
            PipeConfig::CustomPath(path) => {
                // Connect to custom path
                UnixStream::connect(path)
                    .and_then(|stream| {
                        stream.set_nonblocking(false)?;
                        Ok(stream)
                    })
                    .map_err(DiscordIpcError::ConnectionFailed)
            }
        }
    }

    #[cfg(unix)]
    /// Connect to Discord IPC socket using auto-discovery
    fn connect_to_discord_unix_auto() -> Result<UnixStream> {
        // Try environment variables in order of preference
        let env_keys = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"];
        let mut directories = Vec::new();

        for env_key in &env_keys {
            if let Ok(dir) = std::env::var(env_key) {
                directories.push(dir.clone());

                // Also check Flatpak Discord path if XDG_RUNTIME_DIR is set
                if env_key == &"XDG_RUNTIME_DIR" {
                    directories.push(format!("{}/app/com.discordapp.Discord", dir));
                }
            }
        }

        // Fallback to /run/user/{uid} if no env vars found
        if directories.is_empty() {
            let uid = unsafe { libc::getuid() };
            directories.push(format!("/run/user/{}", uid));
            // Also try Flatpak path as fallback
            directories.push(format!("/run/user/{}/app/com.discordapp.Discord", uid));
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
    /// Connect to Discord IPC named pipe on Windows with configuration
    fn connect_to_discord_windows_with_config(
        config: &PipeConfig,
    ) -> Result<(BufReader<std::fs::File>, BufWriter<std::fs::File>)> {
        match config {
            PipeConfig::Auto => {
                // Auto-discovery: try all possible pipes
                Self::connect_to_discord_windows_auto()
            }
            PipeConfig::CustomPath(path) => {
                // Connect to custom path
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(path)
                    .and_then(|file| {
                        let reader_file = file.try_clone()?;
                        Ok((BufReader::new(reader_file), BufWriter::new(file)))
                    })
                    .map_err(DiscordIpcError::ConnectionFailed)
            }
        }
    }

    #[cfg(windows)]
    /// Connect to Discord IPC named pipe on Windows using auto-discovery
    fn connect_to_discord_windows_auto(
    ) -> Result<(BufReader<std::fs::File>, BufWriter<std::fs::File>)> {
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
