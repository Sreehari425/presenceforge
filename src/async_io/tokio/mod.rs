//! Tokio-specific implementations for async Discord IPC

use std::io;
use std::pin::Pin;
use std::future::Future;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};

use crate::ipc::constants;
use crate::async_io::traits::{AsyncRead, AsyncWrite};
use crate::error::{DiscordIpcError, Result};

/// A Discord IPC connection using Tokio
pub enum TokioConnection {
    #[cfg(unix)]
    Unix(UnixStream),
    
    #[cfg(windows)]
    Windows(NamedPipeClient),
}

impl TokioConnection {
    /// Create a new Tokio connection to Discord
    pub async fn new() -> Result<Self> {
        #[cfg(unix)]
        {
            Self::connect_unix().await
        }
        
        #[cfg(windows)]
        {
            Self::connect_windows().await
        }
    }
    
    /// Create a new connection with timeout
    pub async fn new_with_timeout(timeout_ms: u64) -> Result<Self> {
        use tokio::time::{timeout, Duration};
        
        let timeout_duration = Duration::from_millis(timeout_ms);
        
        match timeout(timeout_duration, Self::try_connect()).await {
            Ok(result) => result,
            Err(_) => Err(DiscordIpcError::ConnectionTimeout(timeout_ms)),
        }
    }
    
    /// Try to connect to Discord
    async fn try_connect() -> Result<Self> {
        #[cfg(unix)]
        {
            Self::connect_unix().await
        }
        
        #[cfg(windows)]
        {
            Self::connect_windows().await
        }
    }
    
    #[cfg(unix)]
    /// Connect to Discord IPC socket on Unix systems
    async fn connect_unix() -> Result<Self> {
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
                    },
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
    /// Connect to Discord IPC named pipe on Windows
    async fn connect_windows() -> Result<Self> {
        let mut last_error = None;
        
        for i in 0..constants::MAX_IPC_SOCKETS {
            let pipe_path = format!(r"\\.\pipe\discord-ipc-{}", i);

            // Try to open the named pipe
            match ClientOptions::new().open(pipe_path).await {
                Ok(client) => {
                    return Ok(Self::Windows(client));
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

impl AsyncRead for TokioConnection {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>> {
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
    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>> {
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
    use crate::async_io::client::AsyncDiscordIpcClient;
    use crate::error::Result;
    use super::TokioConnection;
    
    /// Create a new Tokio-based Discord IPC client
    pub async fn new_discord_ipc_client(client_id: impl Into<String>) -> Result<AsyncDiscordIpcClient<TokioConnection>> {
        let connection = TokioConnection::new().await?;
        Ok(AsyncDiscordIpcClient::new(client_id, connection))
    }
    
    /// Create a new Tokio-based Discord IPC client with a connection timeout
    pub async fn new_discord_ipc_client_with_timeout(
        client_id: impl Into<String>,
        timeout_ms: u64
    ) -> Result<AsyncDiscordIpcClient<TokioConnection>> {
        let connection = TokioConnection::new_with_timeout(timeout_ms).await?;
        Ok(AsyncDiscordIpcClient::new(client_id, connection))
    }
}