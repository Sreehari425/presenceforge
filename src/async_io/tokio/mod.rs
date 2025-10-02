//! Tokio-specific implementations for async Discord IPC

#[cfg(windows)]
use crate::debug_println;
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
use crate::ipc::{constants, PipeConfig};

/// A Discord IPC connection using Tokio
pub enum TokioConnection {
    #[cfg(unix)]
    Unix(UnixStream),

    #[cfg(windows)]
    Windows(NamedPipeClient),
}

impl TokioConnection {
    /// Create a new Tokio connection to Discord (uses auto-discovery)
    pub async fn new() -> Result<Self> {
        Self::new_with_config(None).await
    }

    /// Create a new Tokio connection with pipe configuration
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
    pub async fn new_with_config_and_timeout(config: Option<PipeConfig>, timeout_ms: u64) -> Result<Self> {
        use tokio::time::{timeout, Duration};

        let timeout_duration = Duration::from_millis(timeout_ms);

        match timeout(timeout_duration, Self::new_with_config(config)).await {
            Ok(result) => result,
            Err(_) => Err(DiscordIpcError::ConnectionTimeout(timeout_ms)),
        }
    }

    #[cfg(unix)]
    /// Connect to Discord IPC socket on Unix systems with configuration
    async fn connect_unix_with_config(config: &PipeConfig) -> Result<Self> {
        match config {
            PipeConfig::Auto => Self::connect_unix_auto().await,
            PipeConfig::PipeNumber(pipe_num) => {
                if *pipe_num >= constants::MAX_IPC_SOCKETS {
                    return Err(DiscordIpcError::InvalidPipeNumber(*pipe_num));
                }
                Self::connect_unix_specific(*pipe_num).await
            }
            PipeConfig::CustomPath(path) => {
                UnixStream::connect(path)
                    .await
                    .map(Self::Unix)
                    .map_err(DiscordIpcError::ConnectionFailed)
            }
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

    #[cfg(unix)]
    /// Connect to a specific Discord IPC socket number
    async fn connect_unix_specific(pipe_num: u8) -> Result<Self> {
        let env_keys = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"];
        let mut directories = Vec::new();

        for env_key in &env_keys {
            if let Ok(dir) = std::env::var(env_key) {
                directories.push(dir);
            }
        }

        if directories.is_empty() {
            directories.push(format!("/run/user/{}", unsafe { libc::getuid() }));
        }

        let mut last_error = None;

        for dir in &directories {
            let socket_path = format!("{}/{}{}", dir, constants::IPC_SOCKET_PREFIX, pipe_num);

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

        if let Some(err) = last_error {
            if err.kind() == io::ErrorKind::PermissionDenied {
                Err(DiscordIpcError::ConnectionFailed(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("Permission denied when connecting to Discord IPC socket {}. Check file permissions.", pipe_num)
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
            PipeConfig::PipeNumber(pipe_num) => {
                if *pipe_num >= constants::MAX_IPC_SOCKETS {
                    return Err(DiscordIpcError::InvalidPipeNumber(*pipe_num));
                }
                Self::connect_windows_specific(*pipe_num).await
            }
            PipeConfig::CustomPath(path) => {
                ClientOptions::new()
                    .open(path)
                    .map(Self::Windows)
                    .map_err(DiscordIpcError::ConnectionFailed)
            }
        }
    }

    #[cfg(windows)]
    /// Connect to Discord IPC named pipe using auto-discovery
    async fn connect_windows_auto() -> Result<Self> {
        let mut last_error = None;

        for i in 0..constants::MAX_IPC_SOCKETS {
            let pipe_path = format!(r"\\.\pipe\discord-ipc-{}", i);

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
                    "Permission denied when connecting to Discord IPC pipe. Is Discord running with the right permissions?"
                )))
            } else {
                Err(DiscordIpcError::ConnectionFailed(err))
            }
        } else {
            Err(DiscordIpcError::NoValidSocket)
        }
    }

    #[cfg(windows)]
    /// Connect to a specific Discord IPC named pipe number
    async fn connect_windows_specific(pipe_num: u8) -> Result<Self> {
        let pipe_path = format!(r"\\.\pipe\discord-ipc-{}", pipe_num);

        debug_println!("Attempting to connect to Windows named pipe: {}", pipe_path);
        match ClientOptions::new().open(pipe_path.clone()) {
            Ok(client) => {
                debug_println!("Successfully connected to named pipe: {}", pipe_path);
                Ok(Self::Windows(client))
            }
            Err(err) => {
                debug_println!("Failed to connect to named pipe {}: {}", pipe_path, err);
                if err.kind() == io::ErrorKind::PermissionDenied {
                    Err(DiscordIpcError::ConnectionFailed(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!("Permission denied when connecting to Discord IPC pipe {}. Is Discord running with the right permissions?", pipe_num)
                    )))
                } else {
                    Err(DiscordIpcError::ConnectionFailed(err))
                }
            }
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
    use crate::error::Result;

    /// Create a new Tokio-based Discord IPC client
    pub async fn new_discord_ipc_client(
        client_id: impl Into<String>,
    ) -> Result<AsyncDiscordIpcClient<TokioConnection>> {
        let connection = TokioConnection::new().await?;
        Ok(AsyncDiscordIpcClient::new(client_id, connection))
    }

    /// Create a new Tokio-based Discord IPC client with a connection timeout
    pub async fn new_discord_ipc_client_with_timeout(
        client_id: impl Into<String>,
        timeout_ms: u64,
    ) -> Result<AsyncDiscordIpcClient<TokioConnection>> {
        let connection = TokioConnection::new_with_timeout(timeout_ms).await?;
        Ok(AsyncDiscordIpcClient::new(client_id, connection))
    }
}
