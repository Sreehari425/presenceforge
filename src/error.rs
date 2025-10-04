use std::fmt::{self, Display};
use std::io;
use thiserror::Error;

/// Context information for protocol violations
#[derive(Debug, Clone)]
pub struct ProtocolContext {
    pub expected_opcode: Option<u32>,
    pub received_opcode: Option<u32>,
    pub payload_size: Option<usize>,
}

impl ProtocolContext {
    /// Create a new ProtocolContext with all fields empty
    pub fn new() -> Self {
        Self {
            expected_opcode: None,
            received_opcode: None,
            payload_size: None,
        }
    }

    /// Create a ProtocolContext with expected and received opcodes
    pub fn with_opcodes(expected: u32, received: u32) -> Self {
        Self {
            expected_opcode: Some(expected),
            received_opcode: Some(received),
            payload_size: None,
        }
    }

    /// Create a ProtocolContext with a received opcode and payload size
    pub fn with_payload(received_opcode: u32, payload_size: usize) -> Self {
        Self {
            expected_opcode: None,
            received_opcode: Some(received_opcode),
            payload_size: Some(payload_size),
        }
    }
}

impl Default for ProtocolContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Errors related to connecting to Discord
    Connection,
    /// Errors related to the IPC protocol
    Protocol,
    /// Errors related to serialization/deserialization
    Serialization,
    /// Errors related to the Discord application itself
    Application,
    /// Other unspecified errors
    Other,
}

impl Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection => write!(f, "connection"),
            Self::Protocol => write!(f, "protocol"),
            Self::Serialization => write!(f, "serialization"),
            Self::Application => write!(f, "application"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Errors that can occur during Discord IPC operations
///
/// # Error Handling Examples
///
/// Basic error handling:
/// ```rust
/// use presenceforge::{DiscordIpcClient, DiscordIpcError};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut client = match DiscordIpcClient::new("your-client-id") {
///         Ok(client) => client,
///         Err(DiscordIpcError::ConnectionFailed(e)) => {
///             eprintln!("Failed to connect to Discord: {}", e);
///             eprintln!("Is Discord running?");
///             return Err(Box::new(e));
///         },
///         Err(e) => return Err(Box::new(e)),
///     };
///     
///     // Use the client...
///     Ok(())
/// }
/// ```
///
/// Using utility functions for recoverable errors:
/// ```rust
/// use presenceforge::{DiscordIpcClient, DiscordIpcError, Activity};
/// use std::time::Duration;
///
/// fn connect_with_retry(client_id: &str, max_attempts: u32) -> Result<DiscordIpcClient, DiscordIpcError> {
///     let mut attempt = 1;
///     
///     while attempt <= max_attempts {
///         match DiscordIpcClient::new(client_id) {
///             Ok(client) => return Ok(client),
///             Err(e) if e.is_recoverable() && attempt < max_attempts => {
///                 eprintln!("Connection attempt {} failed: {}. Retrying...", attempt, e);
///                 std::thread::sleep(Duration::from_secs(2));
///                 attempt += 1;
///             },
///             Err(e) => return Err(e),
///         }
///     }
///     
///     unreachable!()
/// }
/// ```
///
/// See the `examples/error_handling.rs` file for more comprehensive examples.
#[derive(Error, Debug)]
pub enum DiscordIpcError {
    /// Failed to connect to Discord IPC socket or pipe
    #[error("Failed to connect to Discord IPC socket: {0}")]
    ConnectionFailed(#[source] io::Error),

    /// Failed to discover Discord socket after trying multiple paths
    #[error("Failed to discover Discord socket. Attempted paths: {}", attempted_paths.join(", "))]
    SocketDiscoveryFailed {
        #[source]
        source: io::Error,
        attempted_paths: Vec<String>,
    },

    /// Connection timed out
    #[error("Connection timeout after {timeout_ms}ms")]
    ConnectionTimeout {
        timeout_ms: u64,
        last_error: Option<String>,
    },

    /// Failed to find a valid Discord IPC socket or pipe
    #[error("No Discord IPC socket found. Is Discord running?")]
    NoValidSocket,

    /// Failed to serialize JSON payload
    #[error("Failed to serialize JSON payload: {0}")]
    SerializationFailed(#[source] serde_json::Error),

    /// Failed to deserialize JSON payload from Discord
    #[error("Failed to deserialize response from Discord: {0}")]
    DeserializationFailed(#[source] serde_json::Error),

    /// Received an invalid or unexpected response from Discord
    #[error("Invalid response from Discord: {0}")]
    InvalidResponse(String),

    /// Handshake with Discord failed
    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),

    /// Socket connection was closed unexpectedly
    #[error("Socket connection was closed unexpectedly")]
    SocketClosed,

    /// Received an invalid opcode from Discord
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u32),

    /// Protocol violation occurred during IPC communication
    #[error("Protocol violation: {message}")]
    ProtocolViolation {
        message: String,
        context: ProtocolContext,
    },

    #[error("Discord error: {code} - {message}")]
    DiscordError {
        /// The error code returned by Discord
        code: i32,
        /// The error message returned by Discord
        message: String,
    },

    #[error("Invalid activity: {0}")]
    InvalidActivity(String),
}

impl DiscordIpcError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::ConnectionFailed(_)
            | Self::SocketDiscoveryFailed { .. }
            | Self::ConnectionTimeout { .. }
            | Self::NoValidSocket
            | Self::SocketClosed => ErrorCategory::Connection,

            Self::SerializationFailed(_) | Self::DeserializationFailed(_) => {
                ErrorCategory::Serialization
            }

            Self::InvalidResponse(_)
            | Self::HandshakeFailed(_)
            | Self::InvalidOpcode(_)
            | Self::ProtocolViolation { .. } => ErrorCategory::Protocol,

            Self::DiscordError { .. } => ErrorCategory::Application,

            Self::InvalidActivity(_) => ErrorCategory::Other,
        }
    }

    pub fn is_connection_error(&self) -> bool {
        matches!(self.category(), ErrorCategory::Connection)
    }

    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionTimeout { .. }
                | Self::SocketClosed
                | Self::InvalidResponse(_)
                | Self::SocketDiscoveryFailed { .. }
        )
    }

    pub fn discord_error(code: i32, message: impl Into<String>) -> Self {
        Self::DiscordError {
            code,
            message: message.into(),
        }
    }

    /// Create a SocketDiscoveryFailed error with the attempted paths
    pub fn socket_discovery_failed(
        source: io::Error,
        attempted_paths: Vec<String>,
    ) -> Self {
        Self::SocketDiscoveryFailed {
            source,
            attempted_paths,
        }
    }

    /// Create a ConnectionTimeout error with optional last error
    pub fn connection_timeout(timeout_ms: u64, last_error: Option<String>) -> Self {
        Self::ConnectionTimeout {
            timeout_ms,
            last_error,
        }
    }

    /// Create a ProtocolViolation error with message and context
    pub fn protocol_violation(
        message: impl Into<String>,
        context: ProtocolContext,
    ) -> Self {
        Self::ProtocolViolation {
            message: message.into(),
            context,
        }
    }
}

impl From<io::Error> for DiscordIpcError {
    fn from(error: io::Error) -> Self {
        Self::ConnectionFailed(error)
    }
}

impl From<serde_json::Error> for DiscordIpcError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationFailed(error)
    }
}

/// Result type for Discord IPC operations
pub type Result<T = ()> = std::result::Result<T, DiscordIpcError>;

pub mod utils {
    use super::DiscordIpcError;
    use std::error::Error;
    use std::fmt::{self, Display};

    /// A wrapper error type that can be used to convert DiscordIpcError to application errors
    #[derive(Debug)]
    pub struct AppError {
        source: DiscordIpcError,
        context: Option<String>,
    }

    impl AppError {
        pub fn new(source: DiscordIpcError, context: impl Into<String>) -> Self {
            Self {
                source,
                context: Some(context.into()),
            }
        }

        pub fn from_error(source: DiscordIpcError) -> Self {
            Self {
                source,
                context: None,
            }
        }

        /// Get the underlying Discord IPC error
        pub fn discord_error(&self) -> &DiscordIpcError {
            &self.source
        }

        pub fn context(&self) -> Option<&str> {
            self.context.as_deref()
        }
    }

    impl Display for AppError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if let Some(context) = &self.context {
                write!(f, "{}: {}", context, self.source)
            } else {
                write!(f, "{}", self.source)
            }
        }
    }

    impl Error for AppError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.source)
        }
    }

    /// Extension trait for Result<T, DiscordIpcError> to convert to application errors
    pub trait ResultExt<T> {
        /// Add context to the error
        fn with_context(self, context: impl Into<String>) -> std::result::Result<T, AppError>;

        /// Convert to a different error type
        fn map_err_to<E>(self, f: impl FnOnce(DiscordIpcError) -> E) -> std::result::Result<T, E>;

        /// Handle recoverable errors and attempt to retry the operation
        fn retry_if<F>(
            self,
            is_recoverable: fn(&DiscordIpcError) -> bool,
            retry_op: F,
        ) -> std::result::Result<T, DiscordIpcError>
        where
            F: FnOnce() -> std::result::Result<T, DiscordIpcError>;
    }

    impl<T> ResultExt<T> for std::result::Result<T, DiscordIpcError> {
        fn with_context(self, context: impl Into<String>) -> std::result::Result<T, AppError> {
            self.map_err(|err| AppError::new(err, context))
        }

        fn map_err_to<E>(self, f: impl FnOnce(DiscordIpcError) -> E) -> std::result::Result<T, E> {
            self.map_err(f)
        }

        fn retry_if<F>(
            self,
            is_recoverable: fn(&DiscordIpcError) -> bool,
            retry_op: F,
        ) -> std::result::Result<T, DiscordIpcError>
        where
            F: FnOnce() -> std::result::Result<T, DiscordIpcError>,
        {
            match self {
                Ok(value) => Ok(value),
                Err(err) if is_recoverable(&err) => retry_op(),
                Err(err) => Err(err),
            }
        }
    }
}
