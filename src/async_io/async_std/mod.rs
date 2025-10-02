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
use crate::ipc::constants;

/// A Discord IPC connection using async-std
pub enum AsyncStdConnection {
    #[cfg(unix)]
    Unix(UnixStream),

    #[cfg(windows)]
    Windows(Arc<Mutex<File>>),
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

        let client = AsyncDiscordIpcClient::new(client_id_str, connection);

        Ok(client)
    }
}
