// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025-2026 Sreehari Anil and project contributors

//! smol specific implementations for async Discord IPC

use std::future::Future;
use std::io;
use std::pin::Pin;

#[cfg(unix)]
use smol::io::{AsyncReadExt as _, AsyncWriteExt as _};
#[cfg(unix)]
use smol::net::unix::UnixStream;

#[cfg(windows)]
use smol::fs::File;

use crate::async_io::traits::{AsyncRead, AsyncWrite};
use crate::debug_println;
use crate::error::{DiscordIpcError, Result};
use crate::ipc::{constants, PipeConfig};

/// A Discord IPC connection using smol
pub(crate) enum SmolConnection {
    #[cfg(unix)]
    Unix(UnixStream),

    #[cfg(windows)]
    Windows(File),
}

impl SmolConnection {
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
        let mut last_error = None;

        for socket_path in crate::ipc::discovery::get_socket_paths() {
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

                Ok(Self::Windows(File::from(file)))
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

        for pipe_path in crate::ipc::discovery::get_pipe_paths() {
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
                    return Ok(Self::Windows(File::from(file)));
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

impl AsyncRead for SmolConnection {
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                #[cfg(unix)]
                Self::Unix(stream) => {
                    use smol::io::AsyncReadExt;
                    stream.read(buf).await
                }

                #[cfg(windows)]
                Self::Windows(pipe) => {
                    use smol::io::AsyncReadExt;
                    pipe.read(buf).await
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
                Self::Unix(stream) => {
                    use smol::io::AsyncWriteExt;
                    stream.write(buf).await
                }

                #[cfg(windows)]
                Self::Windows(pipe) => {
                    use smol::io::AsyncWriteExt;
                    pipe.write(buf).await
                }
            }
        })
    }

    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                #[cfg(unix)]
                Self::Unix(stream) => {
                    use smol::io::AsyncWriteExt;
                    stream.flush().await
                }

                #[cfg(windows)]
                Self::Windows(pipe) => {
                    use smol::io::AsyncWriteExt;
                    pipe.flush().await
                }
            }
        })
    }
}

/// smol specific implementation of AsyncDiscordIpcClient
pub mod client {
    use super::SmolConnection;
    use crate::async_io::client::AsyncDiscordIpcClient;
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

        /// Perform handshake and return the typed READY payload when available.
        pub async fn connect_with_ready(&mut self) -> Result<Option<crate::ipc::ReadyEvent>> {
            self.inner.connect_with_ready().await
        }

        /// Parse a raw IPC payload into a READY event if this payload is a READY dispatch.
        pub fn ready_event_from_payload(payload: &Value) -> Result<Option<crate::ipc::ReadyEvent>> {
            AsyncDiscordIpcClient::<SmolConnection>::ready_event_from_payload(payload)
        }

        /// Returns `true` once a handshake has been successfully completed.
        pub fn is_connected(&self) -> bool {
            self.inner.is_connected()
        }

        /// Sets Discord Rich Presence activity
        pub async fn set_activity(&mut self, activity: &crate::activity::Activity) -> Result<()> {
            self.inner.set_activity(activity).await
        }

        /// Clears Discord Rich Presence activity
        pub async fn clear_activity(&mut self) -> Result<Value> {
            self.inner.clear_activity().await
        }

        /// Subscribe to a Discord IPC event.
        pub async fn subscribe<S: Into<String>>(&mut self, event: S, args: Value) -> Result<()> {
            self.inner.subscribe(event, args).await
        }

        /// Unsubscribe from a Discord IPC event.
        pub async fn unsubscribe<S: Into<String>>(&mut self, event: S, args: Value) -> Result<()> {
            self.inner.unsubscribe(event, args).await
        }

        /// Wait for the next IPC event.
        pub async fn next_event(&mut self) -> Result<crate::ipc::EventData> {
            self.inner.next_event().await
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

pub use client::*;
