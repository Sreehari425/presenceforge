//! Connection retry utilities
//!
//! This module provides utilities for implementing retry logic with exponential backoff
//! and connection recovery patterns.

use crate::error::{DiscordIpcError, Result};
use std::time::Duration;

/// Configuration for retry attempts
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff (typically 2.0)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration with custom settings
    pub fn new(
        max_attempts: u32,
        initial_delay_ms: u64,
        max_delay_ms: u64,
        backoff_multiplier: f64,
    ) -> Self {
        Self {
            max_attempts,
            initial_delay_ms,
            max_delay_ms,
            backoff_multiplier,
        }
    }

    /// Create a retry configuration with a specific number of attempts and default delays
    pub fn with_max_attempts(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }

    /// Calculate the delay for a specific attempt number (0-indexed)
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay = (self.initial_delay_ms as f64) * self.backoff_multiplier.powi(attempt as i32);
        let delay_ms = delay.min(self.max_delay_ms as f64) as u64;
        Duration::from_millis(delay_ms)
    }
}

/// Retry a fallible operation with exponential backoff
///
/// This function will retry the operation up to `config.max_attempts` times,
/// only if the error is recoverable (as determined by `is_recoverable` function).
///
/// # Arguments
///
/// * `config` - Retry configuration
/// * `operation` - The operation to retry
///
/// # Returns
///
/// The result of the operation if successful
///
/// # Example
///
/// ```no_run
/// use presenceforge::{DiscordIpcClient, retry::{with_retry, RetryConfig}};
///
/// let config = RetryConfig::with_max_attempts(5);
///
/// let client = with_retry(&config, || {
///     DiscordIpcClient::new("your-client-id")
/// })?;
/// # Ok::<(), presenceforge::DiscordIpcError>(())
/// ```
pub fn with_retry<T, F>(config: &RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut attempt = 0;
    let mut last_error = None;

    while attempt < config.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if e.is_recoverable() && attempt + 1 < config.max_attempts => {
                let delay = config.delay_for_attempt(attempt);
                std::thread::sleep(delay);
                last_error = Some(e);
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }

    // If we exhausted all attempts, return the last error
    Err(last_error.unwrap_or_else(|| {
        DiscordIpcError::ConnectionFailed(std::io::Error::other("Retry attempts exhausted"))
    }))
}

/// Retry an async operation with exponential backoff
///
/// This is the async version of `with_retry`.
///
/// # Arguments
///
/// * `config` - Retry configuration
/// * `operation` - The async operation to retry
///
/// # Example
///
/// ```no_run
/// use presenceforge::async_io::tokio::TokioDiscordIpcClient;
/// use presenceforge::retry::{with_retry_async, RetryConfig};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), presenceforge::DiscordIpcError> {
/// let config = RetryConfig::with_max_attempts(5);
///
/// let mut client = with_retry_async(&config, || {
///     Box::pin(async { TokioDiscordIpcClient::new("your-client-id").await })
/// }).await?;
/// # Ok(())
/// # }
/// ```
/// Retry an async operation with exponential backoff (unified API)

#[cfg(feature = "tokio-runtime")]
pub async fn with_retry_async_tokio<T, F, Fut>(config: &RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut last_error = None;

    while attempt < config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_recoverable() && attempt + 1 < config.max_attempts => {
                let delay = config.delay_for_attempt(attempt);
                tokio::time::sleep(delay).await;
                last_error = Some(e);
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        DiscordIpcError::ConnectionFailed(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Retry attempts exhausted",
        ))
    }))
}

/// Retry an async operation with exponential backoff (async-std version)
#[cfg(feature = "async-std-runtime")]
pub async fn with_retry_async_std<T, F, Fut>(config: &RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut last_error = None;

    while attempt < config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_recoverable() && attempt + 1 < config.max_attempts => {
                let delay = config.delay_for_attempt(attempt);
                async_std::task::sleep(delay).await;
                last_error = Some(e);
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        DiscordIpcError::ConnectionFailed(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Retry attempts exhausted",
        ))
    }))
}

/// Retry an async operation with exponential backoff (smol version)
#[cfg(feature = "smol-runtime")]
pub async fn with_retry_async_smol<T, F, Fut>(config: &RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut last_error = None;

    while attempt < config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_recoverable() && attempt + 1 < config.max_attempts => {
                let delay = config.delay_for_attempt(attempt);
                smol::Timer::after(delay).await;
                last_error = Some(e);
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        DiscordIpcError::ConnectionFailed(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Retry attempts exhausted",
        ))
    }));
}

#[cfg(feature = "tokio-runtime")]
pub use with_retry_async_tokio as with_retry_async;

#[cfg(feature = "async-std-runtime")]
pub use with_retry_async_std as with_retry_async;

#[cfg(feature = "smol-runtime")]
pub use with_retry_async_smol as with_retry_async;

#[cfg(test)]
#[test]
fn test_retry_config_creation() {
    let config = RetryConfig::default();
    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.initial_delay_ms, 1000);

    let custom = RetryConfig::with_max_attempts(5);
    assert_eq!(custom.max_attempts, 5);
}

#[test]
fn test_retry_config_delay_calculation() {
    let config = RetryConfig::new(5, 1000, 10000, 2.0);

    // Test exponential backoff
    let delay0 = config.delay_for_attempt(0);
    let delay1 = config.delay_for_attempt(1);
    let delay2 = config.delay_for_attempt(2);

    assert_eq!(delay0.as_millis(), 1000);
    assert_eq!(delay1.as_millis(), 2000);
    assert_eq!(delay2.as_millis(), 4000);
}

#[test]
fn test_retry_config_max_delay() {
    let config = RetryConfig::new(10, 1000, 5000, 2.0);

    // Delay should cap at max_delay_ms
    let delay10 = config.delay_for_attempt(10);
    assert_eq!(delay10.as_millis(), 5000);
}

#[test]
fn test_retry_exhausts_attempts() {
    let config = RetryConfig::with_max_attempts(3);

    let mut attempt_count = 0;
    let result: std::result::Result<(), DiscordIpcError> = with_retry(&config, || {
        attempt_count += 1;
        // SocketClosed is recoverable
        Err(DiscordIpcError::SocketClosed)
    });

    assert!(result.is_err());
    assert_eq!(attempt_count, 3);
}

#[test]
fn test_non_recoverable_error_no_retry() {
    let config = RetryConfig::with_max_attempts(3);

    let mut attempt_count = 0;
    let result: std::result::Result<(), DiscordIpcError> = with_retry(&config, || {
        attempt_count += 1;
        // ConnectionFailed is NOT recoverable
        Err(DiscordIpcError::ConnectionFailed(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "test",
        )))
    });

    assert!(result.is_err());
    assert_eq!(attempt_count, 1); // Should NOT retry
}

#[test]
fn test_retry_succeeds_on_first_attempt() {
    let config = RetryConfig::with_max_attempts(3);

    let mut attempt_count = 0;
    let result = with_retry(&config, || {
        attempt_count += 1;
        Ok::<_, DiscordIpcError>(42)
    });

    assert_eq!(result.unwrap(), 42);
    assert_eq!(attempt_count, 1);
}

#[test]
fn test_retry_stops_on_non_recoverable_error() {
    let config = RetryConfig::with_max_attempts(5);

    let mut attempt_count = 0;
    let result: std::result::Result<(), DiscordIpcError> = with_retry(&config, || {
        attempt_count += 1;
        // InvalidActivity is not recoverable
        Err(DiscordIpcError::InvalidActivity("test".to_string()))
    });

    assert!(result.is_err());
    assert_eq!(attempt_count, 1); // Should fail immediately
}
