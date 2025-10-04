//! smol specific implementations for async Discord IPC

use std::future::Future;
use std::io;
use std::pin::Pin;

#[cfg(unix)]
use smol::io::{AsyncReadExt as _, AsyncWriteExt as _};
#[cfg(unix)]
use smol::net::unix::UnixStream;

#[cfg(windows)]
use std::fs::File;
#[cfg(windows)]
use std::io::{Read, Write};
#[cfg(windows)]
use std::sync::{Arc, Mutex};

use crate::async_io::traits::{AsyncRead, AsyncWrite};
use crate::debug_println;
use crate::error::{DiscordIpcError, Result};
use crate::ipc::{constants, PipeConfig};

/// A Discord IPC connection using smol
pub enum SmolConnection {
    #[cfg(unix)]
    Unix(UnixStream),

    #[cfg(windows)]
    Windows(Arc<Mutex<File>>),
}

impl SmolConnection {
    /// Create a new smol connection to Discord (uses auto-discovery)
    pub async fn new() -> Result<Self> {
        Self::new_with_config(None).await
    }

    /// Create a new smol connection with pipe configuration
    pub async fn new_with_config(config: Option<PipeConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();

        #[cfg(unix)]
        {
            Self::connect_unix_with_config(&config).await
        }

        #[cfg(windows)]
        {
            Self::connect_windows_with_config(&config).await
        }
    }

    /// Create a new connection with timeout (uses auto-discovery)
    pub async fn new_with_timeout(timeout_ms: u64) -> Result<Self> {
        Self::new_with_config_and_timeout(None, timeout_ms).await
    }

    /// Create a new connection with pipe configuration and timeout
    pub async fn new_with_config_and_timeout(
        config: Option<PipeConfig>,
        timeout_ms: u64,
    ) -> Result<Self> {
        use smol::Timer;
        use std::time::Duration;

        #[cfg(windows)]
        debug_println!(
            "Attempting to connect to Discord IPC with timeout {} ms (Windows)",
            timeout_ms
        );

        #[cfg(unix)]
        debug_println!(
            "Attempting to connect to Discord IPC with timeout {} ms (Unix)",
            timeout_ms
        );

        let timeout_duration = Duration::from_millis(timeout_ms);

        // Use smol's Timer for timeout
        match smol::future::or(
            async {
                Timer::after(timeout_duration).await;
                Err(DiscordIpcError::ConnectionTimeout {
                    timeout_ms,
                    last_error: None,
                })
            },
            Self::new_with_config(config),
        )
        .await
        {
            Ok(conn) => Ok(conn),
            Err(e) => Err(e),
        }
    }

    #[cfg(unix)]
    /// Connect to Discord IPC socket on Unix systems with configuration
    async fn connect_unix_with_config(config: &PipeConfig) -> Result<Self> {
        match config {
            PipeConfig::Auto => Self::connect_unix_auto().await,
            PipeConfig::CustomPath(path) => UnixStream::connect(path)
                .await
                .map(Self::Unix)
                .map_err(DiscordIpcError::ConnectionFailed),
        }
    }

    #[cfg(unix)]
    /// Connect to Discord IPC socket using auto-discovery
    async fn connect_unix_auto() -> Result<Self> {
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

                match UnixStream::connect(&socket_path).await {
                    Ok(stream) => {
                        return Ok(Self::Unix(stream));
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
            if err.kind() == io::ErrorKind::PermissionDenied {
                Err(DiscordIpcError::ConnectionFailed(io::Error::new(
                    io::ErrorKind::PermissionDenied,
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
    async fn connect_windows_with_config(config: &PipeConfig) -> Result<Self> {
        match config {
            PipeConfig::Auto => Self::connect_windows_auto().await,
            PipeConfig::CustomPath(path) => {
                use std::fs::OpenOptions;
                use std::os::windows::fs::OpenOptionsExt;
                const FILE_FLAG_OVERLAPPED: u32 = 0x40000000;

                let path_clone = path.clone();
                let file = smol::unblock(move || {
                    OpenOptions::new()
                        .read(true)
                        .write(true)
                        .custom_flags(FILE_FLAG_OVERLAPPED)
                        .open(&path_clone)
                })
                .await
                .map_err(DiscordIpcError::ConnectionFailed)?;

                Ok(Self::Windows(Arc::new(Mutex::new(file))))
            }
        }
    }

    #[cfg(windows)]
    /// Connect to Discord IPC named pipe using auto-discovery
    async fn connect_windows_auto() -> Result<Self> {
        use std::fs::OpenOptions;
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_OVERLAPPED: u32 = 0x40000000;

        let mut last_error = None;

        for i in 0..constants::MAX_IPC_SOCKETS {
            let pipe_path = format!(r"\\.\pipe\discord-ipc-{}", i);

            debug_println!("Attempting to connect to Windows named pipe: {}", pipe_path);

            // Clone pipe_path for the closure
            let pipe_path_clone = pipe_path.clone();

            // Open the named pipe with overlapped I/O support
            // We use blocking operations wrapped in async context via smol's unblock
            let result = smol::unblock(move || {
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .custom_flags(FILE_FLAG_OVERLAPPED)
                    .open(&pipe_path_clone)
            })
            .await;

            match result {
                Ok(file) => {
                    debug_println!("Successfully opened named pipe: {}", pipe_path);
                    return Ok(Self::Windows(Arc::new(Mutex::new(file))));
                }
                Err(err) => {
                    debug_println!("Failed to connect to named pipe {}: {}", pipe_path, err);
                    last_error = Some(err);
                    continue; // Try next pipe number
                }
            }
        }

        // If we got here, no valid pipe was found
        if let Some(err) = last_error {
            // Return the last error we encountered for diagnostic purposes
            if err.kind() == io::ErrorKind::PermissionDenied {
                Err(DiscordIpcError::ConnectionFailed(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Permission denied when connecting to Discord IPC pipe. Is Discord running with the right permissions?"
                )))
            } else {
                Err(DiscordIpcError::ConnectionFailed(err))
            }
        } else {
            Err(DiscordIpcError::NoValidSocket)
        }
    }
}

impl AsyncRead for SmolConnection {
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                #[cfg(unix)]
                Self::Unix(stream) => stream.read(buf).await,

                #[cfg(windows)]
                Self::Windows(pipe) => {
                    // Clone the Arc to pass into the blocking task
                    let pipe_clone = Arc::clone(pipe);
                    let buf_len = buf.len();

                    // Use smol's unblock to handle synchronous I/O in async context
                    let result = smol::unblock(move || {
                        let mut local_buf = vec![0u8; buf_len];
                        let mut file = match pipe_clone.lock().map_err(|e| {
                            io::Error::new(io::ErrorKind::Other, format!("Mutex poisoned: {}", e))
                        }) {
                            Ok(f) => f,
                            Err(e) => return Err(e),
                        };
                        match file.read(&mut local_buf) {
                            Ok(n) => Ok((n, local_buf)),
                            Err(e) => Err(e),
                        }
                    })
                    .await;

                    match result {
                        Ok((n, data)) => {
                            buf[..n].copy_from_slice(&data[..n]);
                            Ok(n)
                        }
                        Err(e) => Err(e),
                    }
                }
            }
        })
    }
}

impl AsyncWrite for SmolConnection {
    fn write<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                #[cfg(unix)]
                Self::Unix(stream) => stream.write(buf).await,

                #[cfg(windows)]
                Self::Windows(pipe) => {
                    // Clone the Arc to pass into the blocking task
                    let pipe_clone = Arc::clone(pipe);
                    let data = buf.to_vec();

                    // Use smol's unblock to handle synchronous I/O in async context
                    smol::unblock(move || {
                        let mut file = pipe_clone.lock().unwrap();
                        file.write(&data)
                    })
                    .await
                }
            }
        })
    }

    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                #[cfg(unix)]
                Self::Unix(stream) => stream.flush().await,

                #[cfg(windows)]
                Self::Windows(pipe) => {
                    // Clone the Arc to pass into the blocking task
                    let pipe_clone = Arc::clone(pipe);

                    smol::unblock(move || {
                        let mut file = pipe_clone.lock().unwrap();
                        file.flush()
                    })
                    .await
                }
            }
        })
    }
}

/// smol specific implementation of AsyncDiscordIpcClient
pub mod client {
    use super::SmolConnection;
    use crate::async_io::client::AsyncDiscordIpcClient;
    use crate::debug_println;
    use crate::error::{DiscordIpcError, Result};
    use crate::ipc::PipeConfig;
    use serde_json::Value;
    use std::time::Duration;

    /// A reconnectable smol-based Discord IPC client
    ///
    /// This wrapper stores the connection configuration and client ID,
    /// allowing you to reconnect after connection loss.
    pub struct SmolDiscordIpcClient {
        inner: AsyncDiscordIpcClient<SmolConnection>,
        client_id: String,
        pipe_config: Option<PipeConfig>,
        timeout_ms: Option<u64>,
    }

    impl SmolDiscordIpcClient {
        /// Creates a new reconnectable smol-based Discord IPC client
        async fn new_internal(
            client_id: impl Into<String>,
            pipe_config: Option<PipeConfig>,
            timeout_ms: Option<u64>,
        ) -> Result<Self> {
            let client_id = client_id.into();

            let connection = if let Some(timeout) = timeout_ms {
                SmolConnection::new_with_config_and_timeout(pipe_config.clone(), timeout).await?
            } else {
                SmolConnection::new_with_config(pipe_config.clone()).await?
            };

            Ok(Self {
                inner: AsyncDiscordIpcClient::new(client_id.clone(), connection),
                client_id,
                pipe_config,
                timeout_ms,
            })
        }

        /// Performs handshake with Discord
        pub async fn connect(&mut self) -> Result<Value> {
            self.inner.connect().await
        }

        /// Sets Discord Rich Presence activity
        pub async fn set_activity(&mut self, activity: &crate::activity::Activity) -> Result<()> {
            self.inner.set_activity(activity).await
        }

        /// Clears Discord Rich Presence activity
        pub async fn clear_activity(&mut self) -> Result<Value> {
            self.inner.clear_activity().await
        }

        /// Reconnect to Discord IPC
        ///
        /// This method closes the existing connection and establishes a new one,
        /// then performs the handshake again.
        pub async fn reconnect(&mut self) -> Result<Value> {
            // Create a new connection with the same configuration
            let connection = if let Some(timeout) = self.timeout_ms {
                SmolConnection::new_with_config_and_timeout(self.pipe_config.clone(), timeout)
                    .await?
            } else {
                SmolConnection::new_with_config(self.pipe_config.clone()).await?
            };

            // Replace the inner client with a new one
            self.inner = AsyncDiscordIpcClient::new(self.client_id.clone(), connection);

            // Perform handshake
            self.inner.connect().await
        }

        /// Create a new smol-based Discord IPC client (uses auto-discovery)
        pub async fn new(client_id: impl Into<String>) -> Result<Self> {
            Self::new_internal(client_id, None, None).await
        }

        /// Create a new smol-based Discord IPC client with pipe configuration
        pub async fn new_with_config(
            client_id: impl Into<String>,
            config: Option<PipeConfig>,
        ) -> Result<Self> {
            Self::new_internal(client_id, config, None).await
        }

        /// Create a new smol-based Discord IPC client with a connection timeout
        pub async fn new_with_timeout(
            client_id: impl Into<String>,
            timeout_ms: u64,
        ) -> Result<Self> {
            Self::new_internal(client_id, None, Some(timeout_ms)).await
        }

        /// Create a new smol-based Discord IPC client with pipe configuration and timeout
        pub async fn new_with_config_and_timeout(
            client_id: impl Into<String>,
            config: Option<PipeConfig>,
            timeout_ms: u64,
        ) -> Result<Self> {
            Self::new_internal(client_id, config, Some(timeout_ms)).await
        }

        /// Performs handshake with Discord with a timeout
        pub async fn connect_with_timeout(&mut self, timeout_duration: Duration) -> Result<Value> {
            use smol::future::or;
            use smol::Timer;

            match or(
                async move {
                    Timer::after(timeout_duration).await;
                    Err(DiscordIpcError::connection_timeout(
                        timeout_duration.as_millis() as u64,
                        None,
                    ))
                },
                self.inner.connect(),
            )
            .await
            {
                Ok(result) => Ok(result),
                Err(e) => Err(e),
            }
        }

        /// Send a raw IPC message
        pub async fn send_message(
            &mut self,
            opcode: crate::ipc::Opcode,
            payload: &Value,
        ) -> Result<()> {
            self.inner.send_message(opcode, payload).await
        }

        /// Receive a raw IPC message
        pub async fn recv_message(&mut self) -> Result<(crate::ipc::Opcode, Value)> {
            self.inner.recv_message().await
        }
    }

    /// Create a new smol-based Discord IPC client (backward compatible function)
    ///
    /// **Note:** This returns the lower-level `AsyncDiscordIpcClient` which does not support `reconnect()`.
    /// For reconnection support, use `SmolDiscordIpcClient::new()` instead.
    pub async fn new_discord_ipc_client(
        client_id: impl Into<String>,
    ) -> Result<AsyncDiscordIpcClient<SmolConnection>> {
        let client_id_str = client_id.into();
        debug_println!(
            "Creating Discord IPC client with client ID: {}",
            client_id_str
        );

        debug_println!("Attempting to establish connection to Discord...");
        let connection = match SmolConnection::new().await {
            Ok(conn) => {
                debug_println!("Connection established successfully");
                conn
            }
            Err(e) => {
                debug_println!("Failed to connect to Discord: {:?}", e);
                return Err(e);
            }
        };

        let client = AsyncDiscordIpcClient::new(client_id_str, connection);

        Ok(client)
    }

    /// Create a new smol-based Discord IPC client with a connection timeout (backward compatible)
    ///
    /// **Note:** This returns the lower-level `AsyncDiscordIpcClient` which does not support `reconnect()`.
    /// For reconnection support, use `SmolDiscordIpcClient::new_with_timeout()` instead.
    pub async fn new_discord_ipc_client_with_timeout(
        client_id: impl Into<String>,
        timeout_ms: u64,
    ) -> Result<AsyncDiscordIpcClient<SmolConnection>> {
        let client_id_str = client_id.into();
        debug_println!(
            "Creating Discord IPC client with timeout {}ms and client ID: {}",
            timeout_ms,
            client_id_str
        );

        debug_println!("Attempting to establish connection to Discord with timeout...");
        let connection = match SmolConnection::new_with_timeout(timeout_ms).await {
            Ok(conn) => {
                debug_println!("Connection established successfully within timeout");
                conn
            }
            Err(e) => {
                debug_println!("Failed to connect to Discord within timeout: {:?}", e);
                return Err(e);
            }
        };

        let client = AsyncDiscordIpcClient::new(client_id_str, connection);

        Ok(client)
    }

    /// Helper extension trait for smol-specific timeout operations
    pub trait SmolClientExt {
        /// Performs handshake with Discord with a timeout
        ///
        /// # Arguments
        ///
        /// * `timeout_duration` - The maximum time to wait for the connection
        ///
        /// # Returns
        ///
        /// A `Result` containing the Discord handshake response
        ///
        /// # Errors
        ///
        /// Returns `DiscordIpcError::ConnectionTimeout` if the operation times out
        /// Returns `DiscordIpcError::HandshakeFailed` if the handshake fails
        fn connect_with_timeout(
            &mut self,
            timeout_duration: Duration,
        ) -> impl std::future::Future<Output = Result<Value>> + Send;
    }

    impl SmolClientExt for AsyncDiscordIpcClient<SmolConnection> {
        fn connect_with_timeout(
            &mut self,
            timeout_duration: Duration,
        ) -> impl std::future::Future<Output = Result<Value>> + Send {
            use smol::future::or;
            use smol::Timer;

            async move {
                match or(
                    async move {
                        Timer::after(timeout_duration).await;
                        Err(DiscordIpcError::connection_timeout(
                            timeout_duration.as_millis() as u64,
                            None,
                        ))
                    },
                    self.connect(),
                )
                .await
                {
                    Ok(result) => Ok(result),
                    Err(e) => Err(e),
                }
            }
        }
    }
}
