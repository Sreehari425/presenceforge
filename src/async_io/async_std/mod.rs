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
use async_fs::File;
#[cfg(windows)]
use futures::io::AsyncReadExt;
#[cfg(windows)]
use futures::io::AsyncWriteExt;

use crate::async_io::traits::{AsyncRead, AsyncWrite};
use crate::error::{DiscordIpcError, Result};
use crate::ipc::constants;

/// A Discord IPC connection using async-std
pub enum AsyncStdConnection {
    #[cfg(unix)]
    Unix(UnixStream),

    #[cfg(windows)]
    Windows { reader: File, writer: File },
}

impl AsyncStdConnection {
    /// Create a new async-std connection to Discord
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
        use async_std::future::timeout;
        use std::time::Duration;
        
        #[cfg(windows)]
        println!("Attempting to connect to Discord IPC with timeout {} ms (Windows)", timeout_ms);
        
        #[cfg(unix)]
        println!("Attempting to connect to Discord IPC with timeout {} ms (Unix)", timeout_ms);
        
        let timeout_duration = Duration::from_millis(timeout_ms);
        
        match timeout(timeout_duration, Self::try_connect()).await {
            Ok(result) => result,
            Err(_) => Err(DiscordIpcError::ConnectionTimeout(timeout_ms)),
        }
    }    /// Try to connect to Discord
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
    /// Connect to Discord IPC named pipe on Windows
    async fn connect_windows() -> Result<Self> {
        use async_fs::OpenOptions;

        let mut last_error = None;

        for i in 0..constants::MAX_IPC_SOCKETS {
            let pipe_path = format!(r"\\.\pipe\discord-ipc-{}", i);

            // Try to open the named pipe
            match OpenOptions::new()
                .read(true)
                .write(true)
                .open(&pipe_path)
                .await
            {
                Ok(file) => {
                    println!("Successfully opened named pipe: {}", pipe_path);
                    // On Windows, we need two separate file handles
                    let writer = file;
                    let reader = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(&pipe_path)
                        .await
                        .map_err(|e| {
                            println!("Failed to open second handle to named pipe: {}", e);
                            DiscordIpcError::ConnectionFailed(e)
                        })?;
                    println!("Successfully opened second handle to named pipe");
                    return Ok(Self::Windows { reader, writer });
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
                Self::Windows { reader, .. } => futures::io::AsyncReadExt::read(reader, buf).await,
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
                Self::Windows { writer, .. } => {
                    futures::io::AsyncWriteExt::write(writer, buf).await
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
                Self::Windows { writer, .. } => futures::io::AsyncWriteExt::flush(writer).await,
            }
        })
    }
}

/// async-std specific implementation of AsyncDiscordIpcClient
pub mod client {
    use super::AsyncStdConnection;
    use crate::async_io::client::AsyncDiscordIpcClient;
    use crate::error::Result;

    /// Create a new async-std-based Discord IPC client
    pub async fn new_discord_ipc_client(
        client_id: impl Into<String>,
    ) -> Result<AsyncDiscordIpcClient<AsyncStdConnection>> {
        let client_id_str = client_id.into();
        println!("Creating Discord IPC client with client ID: {}", client_id_str);
        
        println!("Attempting to establish connection to Discord...");
        let connection = AsyncStdConnection::new().await?;
        println!("Connection established successfully");
        
        Ok(AsyncDiscordIpcClient::new(client_id_str, connection))
    }

    /// Create a new async-std-based Discord IPC client with a connection timeout
    pub async fn new_discord_ipc_client_with_timeout(
        client_id: impl Into<String>,
        timeout_ms: u64,
    ) -> Result<AsyncDiscordIpcClient<AsyncStdConnection>> {
        let connection = AsyncStdConnection::new_with_timeout(timeout_ms).await?;
        Ok(AsyncDiscordIpcClient::new(client_id, connection))
    }
}
