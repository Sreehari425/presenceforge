# Pipe Selection and Discovery Features

## Overview

This document describes the pipe selection and discovery features in PresenceForge, giving developers control over which Discord IPC pipe to connect to.

> ⚠️ **WARNING:** PresenceForge is an experimental, hobby project (v0.0.0). Features are partially tested, may break, and should **not** be used in production.
> ⚠️ **NOTE:** This feature is experimental/untested. Use at your own risk.

## Key Features

### 1. **PipeConfig Enum**

The `PipeConfig` enum provides two ways to specify which pipe to use:

```rust
pub enum PipeConfig {
    /// Automatically discover and connect to the first available pipe (default)
    Auto,

    /// Connect to a custom pipe path
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

**Note:** Discovery automatically checks both standard Discord installations and Flatpak-packaged Discord on Linux systems.

The `DiscoveredPipe` struct contains:

```rust
pub struct DiscoveredPipe {
    /// The pipe number (0-9) - informational only
    pub pipe_number: u8,
    /// The full path to the pipe - use this with CustomPath
    pub path: String,
}
```

### 3. **Flexible Connection Options**

#### Auto-Discovery (Default Behavior - Recommended)

```rust
// Automatically finds and connects to any available Discord instance
let client = DiscordIpcClient::new("client_id")?;
```

#### Specific Pipe via Custom Path

If you need to connect to a specific pipe, first discover available pipes, then use the path:

```rust
use presenceforge::{PipeConfig, IpcConnection};

// Discover available pipes
let pipes = IpcConnection::discover_pipes();

// Connect to the first pipe (or choose based on your logic)
if let Some(pipe) = pipes.first() {
    let client = DiscordIpcClient::new_with_config(
        "client_id",
        Some(PipeConfig::CustomPath(pipe.path.clone()))
    )?;
}
```

#### Flatpak Discord

```rust
// Discover pipes and find Flatpak Discord
let pipes = IpcConnection::discover_pipes();
let flatpak_pipe = pipes.iter()
    .find(|p| p.path.contains("app/com.discordapp.Discord"));

if let Some(pipe) = flatpak_pipe {
    let client = DiscordIpcClient::new_with_config(
        "client_id",
        Some(PipeConfig::CustomPath(pipe.path.clone()))
    )?;
}
```

#### With Timeout

```rust
// Auto-discovery with timeout
let client = DiscordIpcClient::new_with_timeout("client_id", 5000)?;

// Specific pipe with timeout
let pipes = IpcConnection::discover_pipes();
if let Some(pipe) = pipes.first() {
    let client = DiscordIpcClient::new_with_config_and_timeout(
        "client_id",
        Some(PipeConfig::CustomPath(pipe.path.clone())),
        5000
    )?;
}
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

// With configuration and discovered path
let pipes = IpcConnection::discover_pipes();
if let Some(pipe) = pipes.first() {
    let conn = TokioConnection::new_with_config(
        Some(PipeConfig::CustomPath(pipe.path.clone()))
    ).await?;
}

// With timeout
let conn = TokioConnection::new_with_timeout(5000).await?;

// With configuration and timeout
let pipes = IpcConnection::discover_pipes();
if let Some(pipe) = pipes.first() {
    let conn = TokioConnection::new_with_config_and_timeout(
        Some(PipeConfig::CustomPath(pipe.path.clone())),
        5000
    ).await?;
}
```

### Discovery API

```rust
impl IpcConnection {
    /// Discover all available Discord IPC pipes
    pub fn discover_pipes() -> Vec<DiscoveredPipe>;
}

pub struct DiscoveredPipe {
    /// The pipe number (0-9) - for informational purposes
    pub pipe_number: u8,
    /// The full path to the pipe - use this with CustomPath
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

// Let user select, then connect using the path
let selected_pipe = &pipes[user_selection];
let client = DiscordIpcClient::new_with_config(
    "client_id",
    Some(PipeConfig::CustomPath(selected_pipe.path.clone()))
)?;
```

### 2. **Debugging and Testing**

Connect to a specific pipe for testing:

```rust
let pipes = IpcConnection::discover_pipes();
let test_pipe = pipes.iter()
    .find(|p| p.pipe_number == 0)
    .expect("Pipe 0 not found");

let test_client = DiscordIpcClient::new_with_config(
    "test_client_id",
    Some(PipeConfig::CustomPath(test_pipe.path.clone()))
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

### 4. **Automatic Retry with All Available Pipes**

Retry connection with different pipes:

```rust
let pipes = IpcConnection::discover_pipes();
let mut connected = false;

for pipe in pipes {
    match DiscordIpcClient::new_with_config(
        "client_id",
        Some(PipeConfig::CustomPath(pipe.path.clone()))
    ) {
        Ok(mut client) => {
            match client.connect() {
                Ok(_) => {
                    println!("✓ Connected to pipe {} at {}", pipe.pipe_number, pipe.path);
                    connected = true;
                    break;
                }
                Err(e) => {
                    eprintln!("✗ Handshake failed for pipe {}: {}", pipe.pipe_number, e);
                    continue;
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to create client for pipe {}: {}", pipe.pipe_number, e);
            continue;
        }
    }
}

if !connected {
    eprintln!("Failed to connect to any Discord instance");
}
```

### 5. **Flatpak Discord Support**

The library automatically detects and connects to Flatpak-packaged Discord on Linux:

```rust
// Auto-discovery works seamlessly with Flatpak Discord (recommended)
let client = DiscordIpcClient::new("client_id")?;
client.connect()?;

// Or explicitly connect to Flatpak Discord socket
let pipes = IpcConnection::discover_pipes();
let flatpak_pipe = pipes.iter()
    .find(|p| p.path.contains("app/com.discordapp.Discord"))
    .expect("Flatpak Discord not found");

let client = DiscordIpcClient::new_with_config(
    "client_id",
    Some(PipeConfig::CustomPath(flatpak_pipe.path.clone()))
)?;
```

## Error Handling

Connection errors are handled through the standard `DiscordIpcError` enum:

```rust
use presenceforge::{DiscordIpcError, PipeConfig, DiscordIpcClient};

match DiscordIpcClient::new_with_config(
    "client_id",
    Some(PipeConfig::CustomPath("/invalid/path".to_string()))
) {
    Err(DiscordIpcError::ConnectionFailed(e)) => {
        eprintln!("Failed to connect: {}", e);
    }
    Err(DiscordIpcError::NoValidSocket) => {
        eprintln!("No Discord instance found. Is Discord running?");
    }
    Ok(mut client) => {
        // Connection successful
        client.connect()?;
    }
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

## Design Philosophy

The `PipeConfig` API is designed around two simple, explicit options:

1. **Auto**: Let the library handle discovery automatically (works for 99% of cases)
2. **CustomPath**: Full control with explicit paths when you need it

**Benefits of this approach:**

1. **More Explicit**: You see exactly which socket/pipe you're connecting to
2. **Cross-Platform Clarity**: Works consistently across Unix and Windows
3. **Better Debugging**: Full paths make troubleshooting easier
4. **Flatpak Support**: Makes it obvious when connecting to Flatpak vs standard Discord
5. **Simpler API**: Two clear choices (Auto or CustomPath)

## Platform Support

- **Unix (Linux, macOS)**: Support for all features including custom paths (experimental)
  - Standard Discord: `$XDG_RUNTIME_DIR/discord-ipc-*` or `/tmp/discord-ipc-*`
  - Flatpak Discord: `$XDG_RUNTIME_DIR/app/com.discordapp.Discord/discord-ipc-*`
  - Automatically checks both standard and Flatpak paths during auto-discovery
- **Windows**: Support for auto-discovery and custom paths via Windows named pipe syntax (`\\.\pipe\discord-ipc-*`) (experimental)

## Examples

See these examples for working demonstrations:

- **`examples/pipe_selection.rs`** - Example showing discovery, auto-connection, and custom path usage
- **`examples/basic_flatpak.rs`** - Simple example for Flatpak Discord with fallback to standard Discord

## Quick Start

**Most users should use auto-discovery:**

```rust
let mut client = DiscordIpcClient::new("client_id")?;
client.connect()?;
```

**For advanced use cases (multi-instance, Flatpak, testing):**

```rust
// Discover pipes
let pipes = IpcConnection::discover_pipes();

// Choose a pipe and connect
let client = DiscordIpcClient::new_with_config(
    "client_id",
    Some(PipeConfig::CustomPath(pipes[0].path.clone()))
)?;
```
