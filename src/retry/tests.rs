use crate::error::DiscordIpcError;
/// Smoke test for connection retry functionality
/// This test verifies that the retry module compiles and the API works correctly
use crate::retry::{with_retry, RetryConfig};

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
    let result: Result<(), DiscordIpcError> = with_retry(&config, || {
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
    let result: Result<(), DiscordIpcError> = with_retry(&config, || {
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
    let result: Result<(), DiscordIpcError> = with_retry(&config, || {
        attempt_count += 1;
        // InvalidActivity is not recoverable
        Err(DiscordIpcError::InvalidActivity("test".to_string()))
    });

    assert!(result.is_err());
    assert_eq!(attempt_count, 1); // Should fail immediately
}
