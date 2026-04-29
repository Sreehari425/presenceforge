// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025-2026 Sreehari Anil and project contributors

//! Tokio-specific implementations for async Discord IPC

use std::future::Future;
use std::io;
use std::pin::Pin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(unix)]
use tokio::net::UnixStream;

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};

use crate::async_io::traits::{AsyncRead, AsyncWrite};
use crate::error::{DiscordIpcError, Result};
use crate::ipc::protocol::IpcConfig;
use crate::ipc::PipeConfig;

/// A Discord IPC connection using Tokio
pub(crate) enum TokioConnection {
    #[cfg(unix)]
    Unix(UnixStream),

    #[cfg(windows)]
    Windows(NamedPipeClient),
}

impl TokioConnection {
    /// Create a new Tokio connection with pipe configuration
    #[allow(dead_code)]
    pub async fn new_with_config(config: Option<PipeConfig>) -> Result<Self> {
        Self::new_with_config_and_ipc_config(config, IpcConfig::default()).await
    }

    /// Create a new Tokio connection with pipe and protocol configuration.
    pub async fn new_with_config_and_ipc_config(
        config: Option<PipeConfig>,
        ipc_config: IpcConfig,
    ) -> Result<Self> {
        let config = config.unwrap_or_default();

        #[cfg(unix)]
        {
            Self::connect_unix_with_config(&config, &ipc_config).await
        }

        #[cfg(windows)]
        {
            Self::connect_windows_with_config(&config, &ipc_config).await
        }
    }

    /// Create a new connection with pipe configuration and timeout
    #[allow(dead_code)]
    pub async fn new_with_config_and_timeout(
        config: Option<PipeConfig>,
        timeout_ms: u64,
    ) -> Result<Self> {
        Self::new_with_config_timeout_and_ipc_config(config, timeout_ms, IpcConfig::default()).await
    }

    /// Create a new connection with pipe configuration, timeout, and protocol configuration.
    pub async fn new_with_config_timeout_and_ipc_config(
        config: Option<PipeConfig>,
        timeout_ms: u64,
        ipc_config: IpcConfig,
    ) -> Result<Self> {
        use tokio::time::{sleep, Duration, Instant};

        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);
        let config = config.unwrap_or_default();
        let mut last_error_message = None;

        while start.elapsed() < timeout {
            match Self::new_with_config_and_ipc_config(Some(config.clone()), ipc_config.clone())
                .await
            {
                Ok(connection) => return Ok(connection),
                Err(DiscordIpcError::NoValidSocket) => {
                    last_error_message = Some("No valid Discord socket found".to_string());
                }
                Err(DiscordIpcError::ConnectionFailed(ref source)) => {
                    last_error_message = Some(source.to_string());
                }
                Err(err) => return Err(err),
            }

            sleep(Duration::from_millis(ipc_config.retry_interval_ms)).await;
        }

        Err(DiscordIpcError::connection_timeout(
            timeout_ms,
            last_error_message,
        ))
    }

    #[cfg(unix)]
    /// Connect to Discord IPC socket on Unix systems with configuration
    async fn connect_unix_with_config(config: &PipeConfig, ipc_config: &IpcConfig) -> Result<Self> {
        match config {
            PipeConfig::Auto => Self::connect_unix_auto(ipc_config).await,
            PipeConfig::CustomPath(path) => UnixStream::connect(path)
                .await
                .map(Self::Unix)
                .map_err(DiscordIpcError::ConnectionFailed),
        }
    }

    #[cfg(unix)]
    /// Connect to Discord IPC socket using auto-discovery
    async fn connect_unix_auto(ipc_config: &IpcConfig) -> Result<Self> {
        let mut last_error = None;

        for socket_path in
            crate::ipc::discovery::get_socket_paths_with_limit(ipc_config.max_sockets)
        {
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
    async fn connect_windows_with_config(
        config: &PipeConfig,
        ipc_config: &IpcConfig,
    ) -> Result<Self> {
        match config {
            PipeConfig::Auto => Self::connect_windows_auto(ipc_config).await,
            PipeConfig::CustomPath(path) => ClientOptions::new()
                .open(path)
                .map(Self::Windows)
                .map_err(DiscordIpcError::ConnectionFailed),
        }
    }

    #[cfg(windows)]
    /// Connect to Discord IPC named pipe using auto-discovery
    async fn connect_windows_auto(ipc_config: &IpcConfig) -> Result<Self> {
        use crate::debug_println;
        let mut last_error = None;

        for pipe_path in crate::ipc::discovery::get_pipe_paths_with_limit(ipc_config.max_sockets) {
            // Try to open the named pipe
            debug_println!("Attempting to connect to Windows named pipe: {}", pipe_path);
            match ClientOptions::new().open(pipe_path.clone()) {
                Ok(client) => {
                    debug_println!("Successfully connected to named pipe: {}", pipe_path);
                    return Ok(Self::Windows(client));
                }
                Err(err) => {
                    debug_println!("Failed to connect to named pipe {}: {}", pipe_path, err);
                    last_error = Some(err);
                    continue; // Try next pipe number
                }
            }
        }

        // If we got here, no valid pipe was found
        debug_println!("No valid Discord IPC pipe found after trying all options");
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

impl AsyncRead for TokioConnection {
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                #[cfg(unix)]
                Self::Unix(stream) => stream.read(buf).await,

                #[cfg(windows)]
                Self::Windows(pipe) => pipe.read(buf).await,
            }
        })
    }
}

impl AsyncWrite for TokioConnection {
    fn write<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                #[cfg(unix)]
                Self::Unix(stream) => stream.write(buf).await,

                #[cfg(windows)]
                Self::Windows(pipe) => pipe.write(buf).await,
            }
        })
    }

    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                #[cfg(unix)]
                Self::Unix(stream) => stream.flush().await,

                #[cfg(windows)]
                Self::Windows(pipe) => pipe.flush().await,
            }
        })
    }
}

/// Tokio-specific implementation of AsyncDiscordIpcClient
pub mod client {
    use super::TokioConnection;
    use crate::async_io::client::AsyncDiscordIpcClient;
    use crate::error::{DiscordIpcError, Result};
    use crate::ipc::protocol::IpcConfig;
    use crate::ipc::PipeConfig;
    use serde_json::Value;
    use std::time::Duration;
    use tokio::time::timeout;

    /// A reconnectable Tokio-based Discord IPC client
    ///
    /// This wrapper stores the connection configuration and client ID,
    /// allowing you to reconnect after connection loss.
    pub struct TokioDiscordIpcClient {
        inner: AsyncDiscordIpcClient<TokioConnection>,
        client_id: String,
        pipe_config: Option<PipeConfig>,
        timeout_ms: Option<u64>,
        ipc_config: IpcConfig,
    }

    impl TokioDiscordIpcClient {
        /// Creates a new reconnectable Tokio-based Discord IPC client
        async fn new_internal(
            client_id: impl Into<String>,
            pipe_config: Option<PipeConfig>,
            timeout_ms: Option<u64>,
            ipc_config: IpcConfig,
        ) -> Result<Self> {
            let client_id = client_id.into();

            let connection = if let Some(timeout) = timeout_ms {
                TokioConnection::new_with_config_timeout_and_ipc_config(
                    pipe_config.clone(),
                    timeout,
                    ipc_config.clone(),
                )
                .await?
            } else {
                TokioConnection::new_with_config_and_ipc_config(
                    pipe_config.clone(),
                    ipc_config.clone(),
                )
                .await?
            };

            Ok(Self {
                inner: AsyncDiscordIpcClient::new_with_ipc_config(
                    client_id.clone(),
                    connection,
                    ipc_config.clone(),
                ),
                client_id,
                pipe_config,
                timeout_ms,
                ipc_config,
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
            AsyncDiscordIpcClient::<TokioConnection>::ready_event_from_payload(payload)
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
        /// then performs the handshake again. This is useful when the connection
        /// is lost or Discord is restarted.
        ///
        /// # Returns
        ///
        /// The Discord handshake response as a JSON Value
        ///
        /// # Errors
        ///
        /// Returns a `DiscordIpcError` if the reconnection or handshake fails
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use presenceforge::async_io::tokio::client::TokioDiscordIpcClient;
        /// use presenceforge::ActivityBuilder;
        ///
        /// # #[tokio::main]
        /// # async fn main() -> Result<(), presenceforge::DiscordIpcError> {
        /// let mut client = TokioDiscordIpcClient::new("client_id").await?;
        /// client.connect().await?;
        ///
        /// let activity = ActivityBuilder::new().state("Testing").build();
        ///
        /// // Later, if connection is lost
        /// if let Err(e) = client.set_activity(&activity).await {
        ///     if e.is_connection_error() {
        ///         println!("Connection lost, reconnecting...");
        ///         client.reconnect().await?;
        ///         client.set_activity(&activity).await?;
        ///     }
        /// }
        /// # Ok(())
        /// # }
        /// ```
        pub async fn reconnect(&mut self) -> Result<Value> {
            // Create a new connection with the same configuration
            let connection = if let Some(timeout) = self.timeout_ms {
                TokioConnection::new_with_config_timeout_and_ipc_config(
                    self.pipe_config.clone(),
                    timeout,
                    self.ipc_config.clone(),
                )
                .await?
            } else {
                TokioConnection::new_with_config_and_ipc_config(
                    self.pipe_config.clone(),
                    self.ipc_config.clone(),
                )
                .await?
            };

            // Replace the inner client with a new one
            self.inner = AsyncDiscordIpcClient::new_with_ipc_config(
                self.client_id.clone(),
                connection,
                self.ipc_config.clone(),
            );

            // Perform handshake
            self.inner.connect().await
        }

        /// Create a new Tokio-based Discord IPC client (uses auto-discovery)
        pub async fn new(client_id: impl Into<String>) -> Result<Self> {
            Self::new_internal(client_id, None, None, IpcConfig::default()).await
        }

        /// Create a new Tokio-based Discord IPC client with custom protocol configuration.
        pub async fn new_with_ipc_config(
            client_id: impl Into<String>,
            ipc_config: IpcConfig,
        ) -> Result<Self> {
            Self::new_internal(client_id, None, None, ipc_config).await
        }

        /// Create a new Tokio-based Discord IPC client with pipe configuration
        pub async fn new_with_config(
            client_id: impl Into<String>,
            config: Option<PipeConfig>,
        ) -> Result<Self> {
            Self::new_internal(client_id, config, None, IpcConfig::default()).await
        }

        /// Create a new Tokio-based Discord IPC client with pipe and protocol configuration.
        pub async fn new_with_config_and_ipc_config(
            client_id: impl Into<String>,
            config: Option<PipeConfig>,
            ipc_config: IpcConfig,
        ) -> Result<Self> {
            Self::new_internal(client_id, config, None, ipc_config).await
        }

        /// Create a new Tokio-based Discord IPC client with a connection timeout
        pub async fn new_with_timeout(
            client_id: impl Into<String>,
            timeout_ms: u64,
        ) -> Result<Self> {
            Self::new_internal(client_id, None, Some(timeout_ms), IpcConfig::default()).await
        }

        /// Create a new Tokio-based Discord IPC client with timeout and protocol configuration.
        pub async fn new_with_timeout_and_ipc_config(
            client_id: impl Into<String>,
            timeout_ms: u64,
            ipc_config: IpcConfig,
        ) -> Result<Self> {
            Self::new_internal(client_id, None, Some(timeout_ms), ipc_config).await
        }

        /// Create a new Tokio-based Discord IPC client with pipe configuration and timeout
        pub async fn new_with_config_and_timeout(
            client_id: impl Into<String>,
            config: Option<PipeConfig>,
            timeout_ms: u64,
        ) -> Result<Self> {
            Self::new_internal(client_id, config, Some(timeout_ms), IpcConfig::default()).await
        }

        /// Create a new Tokio-based Discord IPC client with pipe configuration, timeout, and protocol configuration.
        pub async fn new_with_config_timeout_and_ipc_config(
            client_id: impl Into<String>,
            config: Option<PipeConfig>,
            timeout_ms: u64,
            ipc_config: IpcConfig,
        ) -> Result<Self> {
            Self::new_internal(client_id, config, Some(timeout_ms), ipc_config).await
        }

        /// Performs handshake with Discord with a timeout
        pub async fn connect_with_timeout(&mut self, timeout_duration: Duration) -> Result<Value> {
            match timeout(timeout_duration, self.inner.connect()).await {
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

    /// Helper extension trait for Tokio-specific timeout operations
    pub trait TokioClientExt {
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

    impl TokioClientExt for AsyncDiscordIpcClient<TokioConnection> {
        async fn connect_with_timeout(&mut self, timeout_duration: Duration) -> Result<Value> {
            match timeout(timeout_duration, self.connect()).await {
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
