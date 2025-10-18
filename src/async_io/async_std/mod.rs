//! async-std specific implementations for async Discord IPC

use std::future::Future;
use std::io;
use std::pin::Pin;

#[cfg(unix)]
use async_std::io::ReadExt as _;
#[cfg(unix)]
use async_std::io::WriteExt as _;
#[cfg(unix)]
use async_std::os::unix::net::UnixStream;

#[cfg(windows)]
use std::fs::File;
#[cfg(windows)]
use std::io::{Read, Write};
#[cfg(windows)]
use std::sync::{Arc, Mutex};

use crate::async_io::traits::{AsyncRead, AsyncWrite};
use crate::debug_println;
use crate::error::{DiscordIpcError, Result};
use crate::ipc::{PipeConfig, constants};

/// A Discord IPC connection using async-std
pub(crate) enum AsyncStdConnection {
    #[cfg(unix)]
    Unix(UnixStream),

    #[cfg(windows)]
    Windows(Arc<Mutex<File>>),
}

impl AsyncStdConnection {
    /// Create a new async-std connection with pipe configuration
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

    /// Create a new connection with pipe configuration and timeout
    pub async fn new_with_config_and_timeout(
        config: Option<PipeConfig>,
        timeout_ms: u64,
    ) -> Result<Self> {
        use async_std::future::timeout;
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

        match timeout(timeout_duration, Self::new_with_config(config)).await {
            Ok(result) => result,
            Err(_) => Err(DiscordIpcError::ConnectionTimeout {
                timeout_ms,
                last_error: None,
            }),
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
                    "Permission denied when connecting to Discord IPC socket. Check file permissions.",
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

                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .custom_flags(FILE_FLAG_OVERLAPPED)
                    .open(path)
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
            // We use blocking operations wrapped in async context via the blocking crate
            // this can cause a perfomance loss but there was no other way i could think of
            // Todo : write a better solution for the below code

            let result = blocking::unblock(move || {
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
                    "Permission denied when connecting to Discord IPC pipe. Is Discord running with the right permissions?",
                )))
            } else {
                Err(DiscordIpcError::ConnectionFailed(err))
            }
        } else {
            Err(DiscordIpcError::NoValidSocket)
        }
    }
}

impl AsyncRead for AsyncStdConnection {
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

                    // Use blocking crate to handle synchronous I/O in async context
                    let result = blocking::unblock(move || {
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

impl AsyncWrite for AsyncStdConnection {
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

                    // Use blocking crate to handle synchronous I/O in async context
                    blocking::unblock(move || {
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

                    blocking::unblock(move || {
                        let mut file = pipe_clone.lock().unwrap();
                        file.flush()
                    })
                    .await
                }
            }
        })
    }
}

/// async-std specific implementation of AsyncDiscordIpcClient
pub mod client {
    use super::AsyncStdConnection;
    use crate::async_io::client::AsyncDiscordIpcClient;
    use crate::error::{DiscordIpcError, Result};
    use crate::ipc::PipeConfig;
    use serde_json::Value;
    use std::time::Duration;

    /// A reconnectable async-std-based Discord IPC client
    ///
    /// This wrapper stores the connection configuration and client ID,
    /// allowing you to reconnect after connection loss.
    pub struct AsyncStdDiscordIpcClient {
        inner: AsyncDiscordIpcClient<AsyncStdConnection>,
        client_id: String,
        pipe_config: Option<PipeConfig>,
        timeout_ms: Option<u64>,
    }

    impl AsyncStdDiscordIpcClient {
        /// Creates a new reconnectable async-std-based Discord IPC client
        async fn new_internal(
            client_id: impl Into<String>,
            pipe_config: Option<PipeConfig>,
            timeout_ms: Option<u64>,
        ) -> Result<Self> {
            let client_id = client_id.into();

            let connection = if let Some(timeout) = timeout_ms {
                AsyncStdConnection::new_with_config_and_timeout(pipe_config.clone(), timeout)
                    .await?
            } else {
                AsyncStdConnection::new_with_config(pipe_config.clone()).await?
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
                AsyncStdConnection::new_with_config_and_timeout(self.pipe_config.clone(), timeout)
                    .await?
            } else {
                AsyncStdConnection::new_with_config(self.pipe_config.clone()).await?
            };

            // Replace the inner client with a new one
            self.inner = AsyncDiscordIpcClient::new(self.client_id.clone(), connection);

            // Perform handshake
            self.inner.connect().await
        }

        /// Create a new async-std-based Discord IPC client (uses auto-discovery)
        pub async fn new(client_id: impl Into<String>) -> Result<Self> {
            Self::new_internal(client_id, None, None).await
        }

        /// Create a new async-std-based Discord IPC client with pipe configuration
        pub async fn new_with_config(
            client_id: impl Into<String>,
            config: Option<PipeConfig>,
        ) -> Result<Self> {
            Self::new_internal(client_id, config, None).await
        }

        /// Create a new async-std-based Discord IPC client with a connection timeout
        pub async fn new_with_timeout(
            client_id: impl Into<String>,
            timeout_ms: u64,
        ) -> Result<Self> {
            Self::new_internal(client_id, None, Some(timeout_ms)).await
        }

        /// Create a new async-std-based Discord IPC client with pipe configuration and timeout
        pub async fn new_with_config_and_timeout(
            client_id: impl Into<String>,
            config: Option<PipeConfig>,
            timeout_ms: u64,
        ) -> Result<Self> {
            Self::new_internal(client_id, config, Some(timeout_ms)).await
        }

        /// Performs handshake with Discord with a timeout
        pub async fn connect_with_timeout(&mut self, timeout_duration: Duration) -> Result<Value> {
            match async_std::future::timeout(timeout_duration, self.inner.connect()).await {
                Ok(result) => result,
                Err(_) => Err(DiscordIpcError::connection_timeout(
                    timeout_duration.as_millis() as u64,
                    None,
                )),
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

    /// Helper extension trait for async-std-specific timeout operations
    pub trait AsyncStdClientExt {
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

    impl AsyncStdClientExt for AsyncDiscordIpcClient<AsyncStdConnection> {
        async fn connect_with_timeout(&mut self, timeout_duration: Duration) -> Result<Value> {
            match async_std::future::timeout(timeout_duration, self.connect()).await {
                Ok(result) => result,
                Err(_) => Err(DiscordIpcError::connection_timeout(
                    timeout_duration.as_millis() as u64,
                    None,
                )),
            }
        }
    }
}

pub use client::*;
