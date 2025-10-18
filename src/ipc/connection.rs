use byteorder::{LittleEndian, ReadBytesExt};
use bytes::{BufMut, BytesMut};
use serde_json::Value;
use std::io::Read;

#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(windows)]
use std::io::{BufReader, BufWriter};

use crate::error::{DiscordIpcError, ProtocolContext, Result};
use crate::ipc::protocol::{Opcode, constants};

/// Configuration for selecting which Discord IPC pipe to connect to
#[derive(Debug, Clone, Default)]
pub enum PipeConfig {
    /// Automatically discover and connect to the first available pipe (default behavior)
    #[default]
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
    read_buf: BytesMut,
    write_buf: BytesMut,
}

#[cfg(windows)]
pub struct IpcConnection {
    reader: BufReader<std::fs::File>,
    writer: BufWriter<std::fs::File>,
    read_buf: BytesMut,
    write_buf: BytesMut,
}

impl IpcConnection {
    /// Initial capacity for read and write buffers (4KB)
    const INITIAL_BUFFER_CAPACITY: usize = 4096;

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

    // Returns the current users UID on unix based systems
    #[cfg(unix)]
    fn current_uid() -> u32 {
        unsafe { libc::getuid() }
    }
    /// Discovers potential base directories where IPC sockets may exist
    /// Check environment variables
    /// - `XDG_RUNTIME_DIR`
    /// - `TMPDIR`
    /// - `TMP`
    /// - `TEMP`
    /// - `XDG_RUNTIME_DIR/app/com.discordapp.Discord` -> flatpak specific
    /// - if XDG_RUNTIME_DIR is not set the function will grab the uid of the current user
    /// - `/run/user/{UID}`
    #[cfg(unix)]
    fn candidate_ipc_dir() -> Vec<String> {
        let env_keys = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP", "tmp"];
        let mut directories = Vec::new();
        for key in &env_keys {
            if let Ok(dir) = std::env::var(key) {
                directories.push(dir.clone());

                // Also check Flatpak Discord path if XDG_RUNTIME_DIR is set
                if key == &"XDG_RUNTIME_DIR" {
                    directories.push(format!("{}/app/com.discordapp.Discord", dir));
                }
            }
        }
        if directories.is_empty() {
            let uid = Self::current_uid();
            directories.push(format!("/run/user/{}", uid));
            // Also try Flatpak path as fallback
            directories.push(format!("/run/user/{}/app/com.discordapp.Discord", uid));
        }

        directories
    }
    #[cfg(unix)]
    fn discover_pipes_unix() -> Vec<DiscoveredPipe> {
        let mut pipes = Vec::new();

        // Try each directory with each socket number
        for dir in Self::candidate_ipc_dir() {
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
            Ok(Self {
                stream,
                read_buf: BytesMut::with_capacity(Self::INITIAL_BUFFER_CAPACITY),
                write_buf: BytesMut::with_capacity(Self::INITIAL_BUFFER_CAPACITY),
            })
        }

        #[cfg(windows)]
        {
            let (reader, writer) = Self::connect_to_discord_windows_with_config(&config)?;
            Ok(Self {
                reader,
                writer,
                read_buf: BytesMut::with_capacity(Self::INITIAL_BUFFER_CAPACITY),
                write_buf: BytesMut::with_capacity(Self::INITIAL_BUFFER_CAPACITY),
            })
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

        let mut last_error_message = None;

        // Keep trying to connect until we succeed or timeout
        while start.elapsed() < timeout {
            match Self::try_connect_with_config(&config) {
                Ok(connection) => return Ok(connection),
                Err(DiscordIpcError::NoValidSocket) => {
                    last_error_message = Some("No valid Discord socket found".to_string());
                    // Wait a bit before trying again
                    std::thread::sleep(Duration::from_millis(constants::DEFAULT_RETRY_INTERVAL_MS));
                    continue;
                }
                Err(DiscordIpcError::SocketDiscoveryFailed { ref source, .. }) => {
                    last_error_message = Some(format!("Socket discovery failed: {}", source));
                    // Wait a bit before trying again
                    std::thread::sleep(Duration::from_millis(constants::DEFAULT_RETRY_INTERVAL_MS));
                    continue;
                }
                Err(e) => {
                    // Non-recoverable error
                    return Err(e);
                }
            }
        }

        Err(DiscordIpcError::connection_timeout(
            timeout_ms,
            last_error_message,
        ))
    }

    /// Try to connect to Discord with configuration
    fn try_connect_with_config(config: &PipeConfig) -> Result<Self> {
        #[cfg(unix)]
        {
            let stream = Self::connect_to_discord_unix_with_config(config)?;
            Ok(Self {
                stream,
                read_buf: BytesMut::with_capacity(Self::INITIAL_BUFFER_CAPACITY),
                write_buf: BytesMut::with_capacity(Self::INITIAL_BUFFER_CAPACITY),
            })
        }

        #[cfg(windows)]
        {
            let (reader, writer) = Self::connect_to_discord_windows_with_config(config)?;
            Ok(Self {
                reader,
                writer,
                read_buf: BytesMut::with_capacity(Self::INITIAL_BUFFER_CAPACITY),
                write_buf: BytesMut::with_capacity(Self::INITIAL_BUFFER_CAPACITY),
            })
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
        // Try each directory with each socket number
        let mut last_error = None;
        let mut attempted_paths = Vec::new();

        for dir in Self::candidate_ipc_dir() {
            for i in 0..constants::MAX_IPC_SOCKETS {
                let socket_path = format!("{}/{}{}", dir, constants::IPC_SOCKET_PREFIX, i);
                attempted_paths.push(socket_path.clone());

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
            // Return the last error we encountered for diagnostic purposes with all attempted paths
            Err(DiscordIpcError::socket_discovery_failed(
                err,
                attempted_paths,
            ))
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
    fn connect_to_discord_windows_auto()
    -> Result<(BufReader<std::fs::File>, BufWriter<std::fs::File>)> {
        let mut last_error = None;
        let mut attempted_paths = Vec::new();

        for i in 0..constants::MAX_IPC_SOCKETS {
            let pipe_path = format!(r"\\?\pipe\discord-ipc-{}", i);
            attempted_paths.push(pipe_path.clone());

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
            // Return the last error we encountered with all attempted paths
            Err(DiscordIpcError::socket_discovery_failed(
                err,
                attempted_paths,
            ))
        } else {
            Err(DiscordIpcError::NoValidSocket)
        }
    }

    /// Send data with opcode
    pub fn send(&mut self, opcode: Opcode, payload: &Value) -> Result<()> {
        let raw = serde_json::to_vec(payload)?;
        // Clear and prepare write buffer
        self.write_buf.clear();
        self.write_buf.reserve(8 + raw.len());

        // Write header and payload to buffer
        self.write_buf.put_u32_le(opcode.into());
        self.write_buf.put_u32_le(raw.len() as u32);
        self.write_buf.extend_from_slice(&raw);

        #[cfg(unix)]
        {
            use std::io::Write;
            self.stream.write_all(&self.write_buf)?;
        }

        #[cfg(windows)]
        {
            use std::io::Write;
            self.writer.write_all(&self.write_buf)?;
            self.writer.flush()?;
        }

        Ok(())
    }

    /// Receive data and return opcode and payload
    pub fn recv(&mut self) -> Result<(Opcode, Value)> {
        // Read header into buffer
        self.read_buf.clear();
        self.read_buf.reserve(8);

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

        // Validate payload size to prevent excessive memory allocation
        if length > constants::MAX_PAYLOAD_SIZE {
            let context = ProtocolContext::with_payload(opcode_raw, length as usize);
            return Err(DiscordIpcError::protocol_violation(
                format!(
                    "Payload size {} exceeds maximum allowed size of {} bytes",
                    length,
                    constants::MAX_PAYLOAD_SIZE
                ),
                context,
            ));
        }

        let opcode = Opcode::try_from(opcode_raw)?;

        // Reuse read buffer for payload
        self.read_buf.clear();
        self.read_buf.resize(length as usize, 0);

        #[cfg(unix)]
        {
            self.stream
                .read_exact(&mut self.read_buf[..])
                .map_err(|_| DiscordIpcError::SocketClosed)?;
        }

        #[cfg(windows)]
        {
            self.reader
                .read_exact(&mut self.read_buf[..])
                .map_err(|_| DiscordIpcError::SocketClosed)?;
        }

        let value: Value = serde_json::from_slice(&self.read_buf)?;
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
