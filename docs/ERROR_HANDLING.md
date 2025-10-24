# Error Handling Guide

A comprehensive guide to handling errors and implementing retry logic in PresenceForge.


## Table of Contents

- [Overview](#overview)
- [Error Types](#error-types)
- [Error Categories](#error-categories)
- [Common Error Scenarios](#common-error-scenarios)
- [Connection Retry & Reconnection](#connection-retry--reconnection)
- [Best Practices](#best-practices)
- [Recovery Strategies](#recovery-strategies)

---

## Overview

PresenceForge uses the `DiscordIpcError` enum for all error cases. All fallible operations return `Result<T, DiscordIpcError>`, which is aliased as `presenceforge::Result<T>` for convenience.

### Basic Error Handling

```rust
use presenceforge::Result;
use presenceforge::sync::DiscordIpcClient;

fn main() -> Result<()> {
    let mut client = DiscordIpcClient::new("your_client_id")?;
    client.connect()?;

    // ... use client

    Ok(())
}
```

---

## Error Types

Below are the most common variants in `DiscordIpcError` and when they occur. See the crate docs for the full enum.

### `ConnectionFailed(std::io::Error)`

When the library fails to open/connect the IPC socket/pipe.

Common causes:

- Discord is not running
- No available IPC pipes/sockets
- Permission denied accessing pipe/socket

```rust
use presenceforge::DiscordIpcError;
use presenceforge::sync::DiscordIpcClient;

match DiscordIpcClient::new("client_id") {
    Ok(mut client) => {
        let _ = client.connect()?; // ignore handshake payload
        println!("Connected!");
    }
    Err(DiscordIpcError::ConnectionFailed(e)) => {
        eprintln!("Connection failed: {}", e);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

---

### `SocketDiscoveryFailed { source, attempted_paths }`

Auto-discovery tried multiple standard locations but none were usable. `attempted_paths` helps troubleshooting.

---

### `ConnectionTimeout { timeout_ms, last_error }`

Timed out while retrying connections for the configured duration.

---

### `NoValidSocket`

No Discord IPC sockets were found.

---

### `HandshakeFailed(String)`

Discord responded with an error or an unexpected opcode during the handshake.

```rust
match client.connect() {
    Ok(_) => println!("Handshake successful"),
    Err(DiscordIpcError::HandshakeFailed(msg)) => {
        eprintln!("Handshake failed: {}", msg);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

---

### `ProtocolViolation { message, context }` and `InvalidOpcode(u32)`

Indicates malformed data or unexpected protocol values. The `context` includes opcode and payload size when available.

---

### `SerializationFailed(serde_json::Error)` and `DeserializationFailed(serde_json::Error)`

JSON encoding/decoding problems. Often caused by invalid activities or malformed responses.

```rust
if let Err(DiscordIpcError::SerializationFailed(e)) = client.set_activity(&activity) {
    eprintln!("Serialization error: {}", e);
}
```

---

### `InvalidResponse(String)`

Response shape was valid JSON but not what the library expected (e.g., nonce mismatch).

---

### `DiscordError { code, message }`

Discord reported an application-level error (includes error code and message from Discord).

---

### `SocketClosed`

The underlying connection closed while reading/writing.

---

### `InvalidActivity(String)` and `SystemTimeError(String)`

Activity validation failed, or system time issues occurred while computing timestamps.

---

## Error Categories

PresenceForge groups errors into categories for easier handling:

```rust
use presenceforge::error::ErrorCategory;

let category = error.category();
match category {
    ErrorCategory::Connection => {
        println!("Connection problem - check Discord is running");
    }
    ErrorCategory::Protocol => {
        println!("Protocol issue - try updating Discord");
    }
    ErrorCategory::Serialization => {
        println!("Data serialization problem");
    }
    ErrorCategory::Application => {
        println!("Discord application error");
    }
    ErrorCategory::Other => {
        println!("Other error type");
    }
}
```

### Helper Methods

#### `is_connection_error(&self) -> bool`

```rust
if error.is_connection_error() {
    eprintln!(" Tip: Make sure Discord is running!");
    eprintln!(" Try: ps aux | grep -i discord");
}
```

#### `is_recoverable(&self) -> bool`

```rust
if error.is_recoverable() {
    println!(" This error might be recoverable - trying again...");
    retry_logic();
} else {
    eprintln!(" Fatal error - cannot continue");
    std::process::exit(1);
}
```

---

## Common Error Scenarios

### Scenario 1: Discord Not Running

**Problem:** Connection fails because Discord isn't running.

```rust
use presenceforge:: DiscordIpcError;
use presenceforge::sync::DiscordIpcClient;

fn connect_to_discord(client_id: &str) -> Result<DiscordIpcClient, Box<dyn std::error::Error>> {
    match DiscordIpcClient::new(client_id) {
        Ok(mut client) => {
            client.connect()?;
            Ok(client)
        }
        Err(DiscordIpcError::ConnectionFailed(_)) => {
            eprintln!(" Cannot connect to Discord");
            eprintln!(" Make sure Discord is running");
            eprintln!(" Start Discord and try again");
            Err("Discord not running".into())
        }
        Err(e) => Err(e.into()),
    }
}
```

---

### Scenario 2: Lost Connection During Operation

**Problem:** Connection is lost while the application is running.

```rust
use presenceforge::{ActivityBuilder, DiscordIpcError};
use presenceforge::sync::DiscordIpcClient;
use std::thread;
use std::time::Duration;

fn maintain_presence(mut client: DiscordIpcClient) -> Result<(), Box<dyn std::error::Error>> {
    let activity = ActivityBuilder::new()
        .state("Running")
        .start_timestamp_now()?
        .build();

    loop {
        match client.set_activity(&activity) {
            Ok(_) => {
                println!(" Activity updated");
            }
            Err(e) if e.is_connection_error() => {
                eprintln!(" Connection lost. Recreating client and reconnecting...");
                // Recreate the client (sync API has no reconnect method)
                client = DiscordIpcClient::new("your_client_id")?;
                let _ = client.connect()?;
                client.set_activity(&activity)?;
            }
            Err(e) => {
                eprintln!(" Unexpected error: {}", e);
                return Err(e.into());
            }
        }

        thread::sleep(Duration::from_secs(15));
    }
}
```

---

## Connection Retry & Reconnection

PresenceForge provides built-in support for connection retry and reconnection to handle transient network issues and Discord restarts.

### Using the `reconnect()` Method

The `reconnect()` method closes the existing connection and establishes a new one:

```rust
use presenceforge::ActivityBuilder;
use presenceforge::sync::DiscordIpcClient;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DiscordIpcClient::new("your_client_id")?;
    client.connect()?;

    let activity = ActivityBuilder::new()
        .state("Playing a game")
        .build();

    // Update activity in a loop
    loop {
        match client.set_activity(&activity) {
            Ok(_) => println!("✓ Activity updated"),
            Err(e) if e.is_connection_error() => {
                println!("⚠ Connection lost, reconnecting...");
                client.reconnect()?;
                client.set_activity(&activity)?;
            }
            Err(e) => return Err(e.into()),
        }

        std::thread::sleep(Duration::from_secs(15));
    }
}
```

### Using Retry Utilities

For initial connection, use the `with_retry` function with automatic exponential backoff:

```rust
use presenceforge::retry::{with_retry, RetryConfig};
use presenceforge::sync::DiscordIpcClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Default: 3 attempts, 1s initial delay, exponential backoff
    let config = RetryConfig::default();

    let mut client = with_retry(&config, || {
        println!("Attempting to connect...");
        DiscordIpcClient::new("your_client_id")
    })?;

    client.connect()?;
    println!("✓ Connected successfully!");

    Ok(())
}
```

### Custom Retry Configuration

```rust
use presenceforge::retry::RetryConfig;

// More aggressive retry: 5 attempts, shorter delays
let config = RetryConfig::new(
    5,      // max_attempts
    500,    // initial_delay_ms (0.5s)
    8000,   // max_delay_ms (8s)
    2.0,    // backoff_multiplier (exponential)
);

// Retry delays will be: 500ms, 1s, 2s, 4s, 8s
let mut client = with_retry(&config, || {
    DiscordIpcClient::new("your_client_id")
})?;
```

### Async Retry & Reconnect

#### Tokio

The new reconnectable wrapper provides automatic retry and manual reconnect capabilities:

```rust
use presenceforge::async_io::tokio::{TokioDiscordIpcClient, PipeConfig};
use presenceforge::retry::RetryConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with reconnect support
    let mut client = TokioDiscordIpcClient::new(
        "your_client_id",
        PipeConfig::Auto,
        Some(5000)
    );

    // Connect with retry
    let retry_config = RetryConfig::with_max_attempts(5);
    client.connect_with_retry(&retry_config).await?;

    // Later: manual reconnect if connection is lost
    if let Err(e) = client.set_activity(activity).await {
        if e.is_recoverable() {
            client.reconnect().await?;
        }
    }

    Ok(())
}
```

#### async-std

```rust
use presenceforge::async_io::async_std::{AsyncStdDiscordIpcClient, PipeConfig};

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = AsyncStdDiscordIpcClient::new(
        "your_client_id",
        PipeConfig::Auto,
        Some(5000)
    );

    client.connect().await?;

    // Reconnect when needed
    client.reconnect().await?;
    Ok(())
}
```

#### smol

```rust
use presenceforge::async_io::smol::{SmolDiscordIpcClient, PipeConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::block_on(async {
        let mut client = SmolDiscordIpcClient::new(
            "your_client_id",
            PipeConfig::Auto,
            Some(5000)
        );

        client.connect().await?;

        // Reconnect when needed
        client.reconnect().await?;
        Ok(())
    })
}
```

**For complete examples, see:**

- `examples/connection_retry.rs` (sync)
- `examples/async_tokio_reconnect.rs` (async)

---

### Scenario 3: Invalid Configuration

**Problem:** Environment or configuration issues prevent connection.

```rust
use presenceforge::DiscordIpcError;
use presenceforge::sync::DiscordIpcClient;
use std::env;

fn setup_client() -> Result<DiscordIpcClient, Box<dyn std::error::Error>> {
    // Get client ID from environment
    let client_id = env::var("DISCORD_CLIENT_ID").map_err(|_| {
        eprintln!(" DISCORD_CLIENT_ID environment variable not set");
        eprintln!(" Set it with: export DISCORD_CLIENT_ID='your_app_id'");
        eprintln!(" Get your App ID from: https://discord.com/developers/applications");
        "Missing DISCORD_CLIENT_ID"
    })?;

    // Validate client ID format (should be numeric)
    if !client_id.chars().all(|c| c.is_numeric()) {
        eprintln!(" Invalid client ID format: {}", client_id);
        eprintln!(" Client ID should be a numeric string");
        return Err("Invalid client ID format".into());
    }

    // Create client
    let mut client = match DiscordIpcClient::new(&client_id) {
        Ok(c) => c,
        Err(DiscordIpcError::InvalidClientId) => {
            eprintln!(" Discord rejected client ID: {}", client_id);
            eprintln!(" Verify this is the correct Application ID");
            return Err("Invalid client ID".into());
        }
        Err(e) => return Err(e.into()),
    };

    // Connect
    client.connect()?;

    Ok(client)
}
```

---

## Best Practices

### 1. Always Handle Errors Explicitly

**Don't:**

```rust
let client = DiscordIpcClient::new("client_id").unwrap();
```

**Do:**

```rust
let mut client = match DiscordIpcClient::new("client_id") {
    Ok(c) => c,
    Err(e) => {
        eprintln!("Failed to create client: {}", e);
        return Err(e.into());
    }
};
```

---

### 2. Provide Context in Error Messages

**Don't:**

```rust
client.set_activity(&activity)?;
```

**Do:**

```rust
client.set_activity(&activity)
    .map_err(|e| {
        eprintln!("Failed to set activity: {}", e);
        e
    })?;
```

---

### 3. Use Error Categories for Different Handling

```rust
use presenceforge::error::ErrorCategory;

match operation_result {
    Err(e) => {
        match e.category() {
            ErrorCategory::Connection => {
                // Connection errors might be temporary
                retry_with_backoff();
            }
            ErrorCategory::Protocol => {
                // Protocol errors need investigation
                log_for_debugging(&e);
                return Err(e.into());
            }
            ErrorCategory::Serialization => {
                // Serialization errors indicate bugs
                panic!("Bug in activity creation: {}", e);
            }
            _ => return Err(e.into()),
        }
    }
    Ok(_) => { /* success */ }
}
```

---

### 4. Implement Retry Logic for Transient Errors

PresenceForge includes built-in retry utilities with exponential backoff:

```rust
use presenceforge::retry::{with_retry, RetryConfig};

use presenceforge::sync::DiscordIpcClient;

fn connect_with_retry(client_id: &str) -> Result<DiscordIpcClient, Box<dyn std::error::Error>> {
    // Use default retry config (3 attempts, 1s initial delay, exponential backoff)
    let config = RetryConfig::default();

    let mut client = with_retry(&config, || {
        DiscordIpcClient::new(client_id)
    })?;

    client.connect()?;
    Ok(client)
}
```

**Custom retry configuration:**

```rust
use presenceforge::retry::RetryConfig;

// Create custom retry configuration
let config = RetryConfig::new(
    5,      // max_attempts
    500,    // initial_delay_ms
    8000,   // max_delay_ms
    2.0,    // backoff_multiplier
);

let mut client = with_retry(&config, || {
    DiscordIpcClient::new(client_id)
})?;
```

---

### 5. Clean Up on Errors

```rust
use presenceforge::ActivityBuilder;
use presenceforge::sync::DiscordIpcClient;

fn run_presence() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DiscordIpcClient::new("client_id")?;
    client.connect()?;

    let activity = ActivityBuilder::new()
        .state("Running")
        .build();

    client.set_activity(&activity)?;

    // Ensure cleanup happens even on error
    let result = do_work();

    // Always try to clear activity before exiting
    if let Err(e) = client.clear_activity() {
        eprintln!("Warning: Failed to clear activity: {}", e);
    }

    result
}
```

---

## Recovery Strategies

### Strategy 1: Automatic Reconnection

```rust
struct ResilientClient {
    client_id: String,
    client: Option<DiscordIpcClient>,
}

impl ResilientClient {
    fn new(client_id: String) -> Self {
        Self {
            client_id,
            client: None,
        }
    }

    fn ensure_connected(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.client.is_none() {
            let mut client = DiscordIpcClient::new(&self.client_id)?;
            client.connect()?;
            self.client = Some(client);
        }
        Ok(())
    }

    fn set_activity_resilient(
        &mut self,
        activity: &Activity
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.ensure_connected()?;

        let result = self.client
            .as_mut()
            .unwrap()
            .set_activity(activity);

        if let Err(e) = result {
            if e.is_connection_error() {
                // Connection lost, reset and try once more
                self.client = None;
                self.ensure_connected()?;
                return Ok(self.client.as_mut().unwrap().set_activity(activity)?);
            }
            return Err(e.into());
        }

        Ok(())
    }
}
```

---

### Strategy 2: Graceful Degradation

```rust
fn update_presence_best_effort(
    client: &mut DiscordIpcClient,
    activity: &Activity
) {
    match client.set_activity(activity) {
        Ok(_) => println!(" Presence updated"),
        Err(e) => {
            eprintln!(" Failed to update presence: {}", e);
            eprintln!(" Continuing without Rich Presence");
            // Application continues without Rich Presence
        }
    }
}
```

---

### Strategy 3: User Notification

```rust
fn connect_with_user_feedback(
    client_id: &str
) -> Result<DiscordIpcClient, Box<dyn std::error::Error>> {
    println!(" Connecting to Discord...");

    match DiscordIpcClient::new(client_id) {
        Ok(mut client) => {
            match client.connect() {
                Ok(_) => {
                    println!(" Connected to Discord successfully!");
                    Ok(client)
                }
                Err(e) => {
                    eprintln!(" Handshake failed: {}", e);
                    eprintln!(" Try restarting Discord");
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            eprintln!(" Connection failed: {}", e);
            eprintln!();
            eprintln!(" Troubleshooting checklist:");
            eprintln!("   [ ] Discord is installed");
            eprintln!("   [ ] Discord is running");
            eprintln!("   [ ] Discord is not blocked by firewall");
            eprintln!("   [ ] Your client ID is correct");
            Err(e.into())
        }
    }
}
```

---

## See Also

- [API Reference](API_REFERENCE.md) - Error type documentation (WIP)
- [FAQ](FAQ.md) - Common issues and solutions
- [Getting Started](GETTING_STARTED.md) - Basic setup guide
