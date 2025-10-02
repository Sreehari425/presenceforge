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
use crate::debug_println;
use crate::error::{DiscordIpcError, Result};
use crate::ipc::constants;

// Constants for Windows named pipe handling
#[cfg(windows)]
const WINDOWS_PIPE_STABILIZATION_DELAY_MS: u64 = 100; // Delay to ensure pipe stability between operations
#[cfg(windows)]
const WINDOWS_HANDLE_CREATION_DELAY_MS: u64 = 100; // Delay to avoid race conditions between handle creation
#[cfg(windows)]
const WINDOWS_READ_RETRY_DELAY_MS: u64 = 50; // Delay between read retry attempts
#[cfg(windows)]
const MAX_READ_RETRIES: u8 = 3; // Maximum number of read retry attempts

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
        use std::time::Duration;

        let mut last_error = None;

        for i in 0..constants::MAX_IPC_SOCKETS {
            let pipe_path = format!(r"\\.\pipe\discord-ipc-{}", i);

            // Try to open the named pipe
            debug_println!("Attempting to connect to named pipe: {}", pipe_path);
            match OpenOptions::new()
                .read(true)
                .write(true)
                .open(&pipe_path)
                .await
            {
                Ok(file) => {
                    debug_println!("Successfully opened named pipe: {}", pipe_path);
                    // On Windows, we need two separate file handles
                    let writer = file;

                    // Small delay before opening second handle to avoid race conditions on Windows named pipes
                    // This prevents ERROR_PIPE_BUSY errors when opening multiple handles to the same pipe
                    async_std::task::sleep(Duration::from_millis(WINDOWS_HANDLE_CREATION_DELAY_MS))
                        .await;

                    // Open a second handle for reading
                    let reader = match OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(&pipe_path)
                        .await
                    {
                        Ok(f) => {
                            debug_println!("Successfully opened second handle to named pipe");
                            f
                        }
                        Err(e) => {
                            debug_println!("Failed to open second handle to named pipe: {}", e);
                            return Err(DiscordIpcError::ConnectionFailed(e));
                        }
                    };

                    return Ok(Self::Windows { reader, writer });
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
                Self::Windows { reader, .. } => {
                    // Improved read implementation with retry logic for Windows
                    let mut total_read = 0;
                    let mut retries = 0;

                    while total_read == 0 && retries < MAX_READ_RETRIES {
                        match futures::io::AsyncReadExt::read(reader, buf).await {
                            Ok(0) => {
                                // No data yet, retry after a short delay
                                // This addresses Windows named pipe behavior where reads may initially return
                                // no data even when data is available shortly after
                                async_std::task::sleep(Duration::from_millis(
                                    WINDOWS_READ_RETRY_DELAY_MS,
                                ))
                                .await;
                                retries += 1;
                            }
                            Ok(n) => {
                                total_read = n;
                                break;
                            }
                            Err(e) => return Err(e),
                        }
                    }

                    if total_read == 0 && retries >= MAX_READ_RETRIES {
                        // If we still have no data after retries, return would block
                        Err(io::Error::new(
                            io::ErrorKind::WouldBlock,
                            "No data available after retries",
                        ))
                    } else {
                        Ok(total_read)
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
                Self::Windows { writer, .. } => {
                    // Improved write implementation for Windows
                    let result = futures::io::AsyncWriteExt::write(writer, buf).await?;

                    // Ensure data is flushed immediately on Windows
                    futures::io::AsyncWriteExt::flush(writer).await?;

                    Ok(result)
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
    use crate::debug_println;
    use crate::error::Result;

    /// Create a new async-std-based Discord IPC client
    pub async fn new_discord_ipc_client(
        client_id: impl Into<String>,
    ) -> Result<AsyncDiscordIpcClient<AsyncStdConnection>> {
        let client_id_str = client_id.into();
        debug_println!(
            "Creating Discord IPC client with client ID: {}",
            client_id_str
        );

        debug_println!("Attempting to establish connection to Discord...");
        let connection = match AsyncStdConnection::new().await {
            Ok(conn) => {
                debug_println!("Connection established successfully");
                conn
            }
            Err(e) => {
                debug_println!("Failed to connect to Discord: {:?}", e);
                return Err(e);
            }
        };

        // Give the connection a moment to stabilize (important for Windows)
        // This delay ensures that the pipe is fully ready before sending/receiving data
        // Windows named pipes can sometimes report as connected but not be fully ready
        #[cfg(windows)]
        async_std::task::sleep(Duration::from_millis(WINDOWS_PIPE_STABILIZATION_DELAY_MS)).await;

        let client = AsyncDiscordIpcClient::new(client_id_str, connection);

        Ok(client)
    }

    /// Create a new async-std-based Discord IPC client with a connection timeout
    pub async fn new_discord_ipc_client_with_timeout(
        client_id: impl Into<String>,
        timeout_ms: u64,
    ) -> Result<AsyncDiscordIpcClient<AsyncStdConnection>> {
        let client_id_str = client_id.into();
        debug_println!(
            "Creating Discord IPC client with timeout {}ms and client ID: {}",
            timeout_ms,
            client_id_str
        );

        debug_println!("Attempting to establish connection to Discord with timeout...");
        let connection = match AsyncStdConnection::new_with_timeout(timeout_ms).await {
            Ok(conn) => {
                debug_println!("Connection established successfully within timeout");
                conn
            }
            Err(e) => {
                debug_println!("Failed to connect to Discord within timeout: {:?}", e);
                return Err(e);
            }
        };

        // Give the connection a moment to stabilize (important for Windows)
        // This delay ensures that the pipe is fully ready before sending/receiving data
        // Windows named pipes can sometimes report as connected but not be fully ready
        #[cfg(windows)]
        async_std::task::sleep(Duration::from_millis(WINDOWS_PIPE_STABILIZATION_DELAY_MS)).await;
        let client = AsyncDiscordIpcClient::new(client_id_str, connection);

        Ok(client)
    }
}
