use presenceforge::retry::with_retry;
use presenceforge::{retry::RetryConfig, DiscordIpcError};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn quick_retry_config(max_attempts: u32) -> RetryConfig {
    RetryConfig::new(max_attempts, 5, 20, 2.0)
}

#[test]
fn retry_succeeds_after_transient_error() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let result = with_retry(&quick_retry_config(4), || {
        let current = attempts.fetch_add(1, Ordering::SeqCst);
        if current == 0 {
            Err(DiscordIpcError::SocketClosed)
        } else {
            Ok("connected")
        }
    })
    .expect("retry should eventually succeed");

    assert_eq!(result, "connected");
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[test]
fn retry_stops_on_non_recoverable_error() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let result: presenceforge::Result<&str> = with_retry(&quick_retry_config(5), || {
        attempts.fetch_add(1, Ordering::SeqCst);
        Err(DiscordIpcError::InvalidActivity("bad".into()))
    });

    assert!(result.is_err());
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
}
