# Pipe Selection and Discovery Features

## Overview

This document describes the new pipe selection and discovery features added to PresenceForge, giving developers full control over which Discord IPC pipe to connect to.

## Key Features

### 1. **PipeConfig Enum**

The `PipeConfig` enum provides three ways to specify which pipe to use:

```rust
pub enum PipeConfig {
    /// Automatically discover and connect to the first available pipe (default)
    Auto,

    /// Connect to a specific pipe number (0-9)
    PipeNumber(u8),

    /// Connect to a custom pipe path (advanced usage)
    CustomPath(String),
}
```

### 2. **Pipe Discovery**

Discover all available Discord IPC pipes on the system:

```rust
use presenceforge::IpcConnection;

let pipes = IpcConnection::discover_pipes();
for pipe in pipes {
    println!("Found pipe {}: {}", pipe.pipe_number, pipe.path);
}
```

### 3. **Flexible Connection Options**

#### Auto-Discovery (Default Behavior)

```rust
// Keeps existing behavior - no breaking changes
let client = DiscordIpcClient::new("client_id")?;
```

#### Specific Pipe Number

```rust
use presenceforge::PipeConfig;

let client = DiscordIpcClient::new_with_config(
    "client_id",
    Some(PipeConfig::PipeNumber(0))
)?;
```

#### Custom Pipe Path (Unix)

```rust
#[cfg(unix)]
let client = DiscordIpcClient::new_with_config(
    "client_id",
    Some(PipeConfig::CustomPath("/tmp/discord-ipc-0".to_string()))
)?;
```

#### With Timeout

```rust
// Auto-discovery with timeout
let client = DiscordIpcClient::new_with_timeout("client_id", 5000)?;

// Specific pipe with timeout
let client = DiscordIpcClient::new_with_config_and_timeout(
    "client_id",
    Some(PipeConfig::PipeNumber(0)),
    5000
)?;
```

## API Reference

### Synchronous API

```rust
impl DiscordIpcClient {
    /// Auto-discovery (default)
    pub fn new<S: Into<String>>(client_id: S) -> Result<Self>;

    /// With pipe configuration
    pub fn new_with_config<S: Into<String>>(
        client_id: S,
        config: Option<PipeConfig>
    ) -> Result<Self>;

    /// Auto-discovery with timeout
    pub fn new_with_timeout<S: Into<String>>(
        client_id: S,
        timeout_ms: u64
    ) -> Result<Self>;

    /// With pipe configuration and timeout
    pub fn new_with_config_and_timeout<S: Into<String>>(
        client_id: S,
        config: Option<PipeConfig>,
        timeout_ms: u64
    ) -> Result<Self>;
}
```

### Async API (Tokio, async-std, smol)

All async runtime implementations support the same API:

```rust
// Tokio example
use presenceforge::async_io::tokio::TokioConnection;

// Auto-discovery
let conn = TokioConnection::new().await?;

// With configuration
let conn = TokioConnection::new_with_config(
    Some(PipeConfig::PipeNumber(0))
).await?;

// With timeout
let conn = TokioConnection::new_with_timeout(5000).await?;

// With configuration and timeout
let conn = TokioConnection::new_with_config_and_timeout(
    Some(PipeConfig::PipeNumber(0)),
    5000
).await?;
```

### Discovery API

```rust
impl IpcConnection {
    /// Discover all available Discord IPC pipes
    pub fn discover_pipes() -> Vec<DiscoveredPipe>;
}

pub struct DiscoveredPipe {
    /// The pipe number (0-9)
    pub pipe_number: u8,
    /// The full path to the pipe
    pub path: String,
}
```

## Use Cases

### 1. **Multi-Instance Discord**

If a user runs multiple Discord instances, you can let them choose which one to connect to:

```rust
let pipes = IpcConnection::discover_pipes();
println!("Available Discord instances:");
for (i, pipe) in pipes.iter().enumerate() {
    println!("{}. Pipe {} - {}", i + 1, pipe.pipe_number, pipe.path);
}
// Let user select...
```

### 2. **Debugging and Testing**

Connect to a specific pipe for testing:

```rust
let test_client = DiscordIpcClient::new_with_config(
    "test_client_id",
    Some(PipeConfig::PipeNumber(0))
)?;
```

### 3. **Docker/Container Environments**

Use custom paths for non-standard setups:

```rust
#[cfg(unix)]
let client = DiscordIpcClient::new_with_config(
    "client_id",
    Some(PipeConfig::CustomPath("/custom/path/discord-ipc-0".to_string()))
)?;
```

### 4. **Automatic Retry with Specific Pipes**

Retry connection with different pipes:

```rust
let pipes = IpcConnection::discover_pipes();
for pipe in pipes {
    match DiscordIpcClient::new_with_config(
        "client_id",
        Some(PipeConfig::PipeNumber(pipe.pipe_number))
    ) {
        Ok(mut client) => {
            client.connect()?;
            // Success!
            break;
        }
        Err(e) => {
            eprintln!("Failed to connect to pipe {}: {}", pipe.pipe_number, e);
            continue;
        }
    }
}
```

## Error Handling

New error variant added:

```rust
#[error("Invalid pipe number: {0}. Pipe number must be between 0 and 9")]
InvalidPipeNumber(u8)
```

Example:

```rust
match DiscordIpcClient::new_with_config("client_id", Some(PipeConfig::PipeNumber(15))) {
    Err(DiscordIpcError::InvalidPipeNumber(num)) => {
        eprintln!("Invalid pipe number: {}", num);
    }
    _ => {}
}
```

## Backward Compatibility

All existing code continues to work without modification:

```rust
// Old code still works exactly the same
let mut client = DiscordIpcClient::new("client_id")?;
client.connect()?;
```

The new features are opt-in and don't break existing functionality.

## Platform Support

- **Unix (Linux, macOS)**: Full support for all features including custom paths
- **Windows**: Full support for auto-discovery and pipe numbers; custom paths supported via Windows named pipe syntax

## Example

See `examples/pipe_selection.rs` for a complete working example demonstrating all features.

## Summary

This feature gives developers:

- ✅ Full control over pipe selection
- ✅ Ability to discover available pipes
- ✅ Support for custom paths
- ✅ Timeout support for all connection methods
- ✅ Backward compatibility
- ✅ Cross-platform support (Unix & Windows)
- ✅ Works with all async runtimes (tokio, async-std, smol)
