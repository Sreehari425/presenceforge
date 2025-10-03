# API Reference

Complete API reference for PresenceForge.

> Note: This is currently in early development (v0.0.0). Things might break.

## Table of Contents

- [DiscordIpcClient](#discordipclient)
- [ActivityBuilder](#activitybuilder)
- [Activity](#activity)
- [PipeConfig](#pipeconfig)
- [IpcConnection](#ipcconnection)
- [Error Types](#error-types)
- [Async Clients](#async-clients)

---

## DiscordIpcClient

The synchronous Discord IPC client for managing Rich Presence.

### Creating a Client

#### `DiscordIpcClient::new(client_id: impl Into<String>) -> Result<Self>`

Creates a new Discord IPC client with automatic pipe discovery.

```rust
use presenceforge::DiscordIpcClient;

let client = DiscordIpcClient::new("your_client_id")?;
```

**Parameters:**

- `client_id` - Your Discord Application ID (from Developer Portal)

**Returns:** `Result<DiscordIpcClient, DiscordIpcError>`

**Errors:**

- `DiscordIpcError::ConnectionFailed` - Could not find or connect to Discord

---

#### `DiscordIpcClient::new_with_config(client_id: impl Into<String>, config: Option<PipeConfig>) -> Result<Self>`

Creates a new Discord IPC client with custom pipe configuration.

```rust
use presenceforge::{DiscordIpcClient, PipeConfig};

// Auto-discovery (equivalent to ::new())
let client = DiscordIpcClient::new_with_config("client_id", None)?;

// Connect to specific pipe path
let client = DiscordIpcClient::new_with_config(
    "client_id",
    Some(PipeConfig::CustomPath("/tmp/discord-ipc-0".to_string()))
)?;
```

**Parameters:**

- `client_id` - Your Discord Application ID
- `config` - Optional pipe configuration. If `None`, uses auto-discovery

**Returns:** `Result<DiscordIpcClient, DiscordIpcError>`

---

### Connection Methods

#### `connect(&mut self) -> Result<()>`

Establishes connection and performs handshake with Discord.

```rust
let mut client = DiscordIpcClient::new("client_id")?;
client.connect()?;
```

**Must be called before** setting or clearing activities.

**Returns:** `Result<(), DiscordIpcError>`

**Errors:**

- `DiscordIpcError::ConnectionFailed` - Handshake failed
- `DiscordIpcError::ProtocolError` - Invalid response from Discord

---

#### `reconnect(&mut self) -> Result<()>`

Reconnects to Discord (useful if connection is lost).

```rust
// If connection is lost
if let Err(_) = client.set_activity(&activity) {
    client.reconnect()?;
    client.set_activity(&activity)?;
}
```

**Returns:** `Result<(), DiscordIpcError>`

---

### Activity Methods

#### `set_activity(&mut self, activity: &Activity) -> Result<()>`

Sets the current Rich Presence activity.

```rust
use presenceforge::ActivityBuilder;

let activity = ActivityBuilder::new()
    .state("Playing")
    .details("Main Menu")
    .build();

client.set_activity(&activity)?;
```

**Parameters:**

- `activity` - The activity to display (typically built with `ActivityBuilder`)

**Returns:** `Result<(), DiscordIpcError>`

**Note:** You can call this multiple times to update the presence.

---

#### `clear_activity(&mut self) -> Result<()>`

Clears the current Rich Presence activity.

```rust
client.clear_activity()?;
```

**Returns:** `Result<(), DiscordIpcError>`

**Note:** This removes the Rich Presence from your profile entirely.

---

## ActivityBuilder

Builder pattern for creating Rich Presence activities.

### Creating a Builder

#### `ActivityBuilder::new() -> Self`

Creates a new activity builder.

```rust
use presenceforge::ActivityBuilder;

let activity = ActivityBuilder::new()
    .state("Hello!")
    .build();
```

---

### Text Methods

#### `state(self, state: impl Into<String>) -> Self`

Sets the state text (smaller text, first line).

```rust
.state("In a Match")
```

**Character limit:** 128 characters

---

#### `details(self, details: impl Into<String>) -> Self`

Sets the details text (larger text, second line).

```rust
.details("Competitive Mode")
```

**Character limit:** 128 characters

---

### Image Methods

#### `large_image(self, key: impl Into<String>) -> Self`

Sets the large image by asset key.

```rust
.large_image("game_logo")
```

**Note:** Asset keys must be uploaded to Discord Developer Portal under Rich Presence → Art Assets.

---

#### `large_text(self, text: impl Into<String>) -> Self`

Sets the hover text for the large image.

```rust
.large_text("My Awesome Game v1.0")
```

---

#### `small_image(self, key: impl Into<String>) -> Self`

Sets the small image (circular overlay on large image).

```rust
.small_image("character_icon")
```

---

#### `small_text(self, text: impl Into<String>) -> Self`

Sets the hover text for the small image.

```rust
.small_text("Level 50 Warrior")
```

---

### Timestamp Methods

#### `start_timestamp(self, timestamp: i64) -> Self`

Sets the start timestamp (shows elapsed time).

```rust
use std::time::{SystemTime, UNIX_EPOCH};

let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

.start_timestamp(now)
```

**Parameter:** Unix timestamp in seconds

**Display:** Shows as "XX:XX elapsed" in Discord

---

#### `start_timestamp_now(self) -> Self`

Sets the start timestamp to the current time.

```rust
.start_timestamp_now()
```

**Equivalent to:**

```rust
.start_timestamp(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64)
```

---

#### `end_timestamp(self, timestamp: i64) -> Self`

Sets the end timestamp (shows remaining time).

```rust
use std::time::{SystemTime, UNIX_EPOCH};

let in_10_min = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64 + 600;

.end_timestamp(in_10_min)
```

**Parameter:** Unix timestamp in seconds

**Display:** Shows as "XX:XX left" in Discord

---

### Button Methods

#### `add_button(self, label: impl Into<String>, url: impl Into<String>) -> Self`

Adds a button to the Rich Presence (max 2 buttons).

```rust
.add_button("Watch Stream", "https://twitch.tv/username")
.add_button("GitHub", "https://github.com/username")
```

**Parameters:**

- `label` - Button text (max 32 characters)
- `url` - URL to open when clicked

**Note:** Only the first 2 buttons will be displayed.

---

### Party Methods

#### `party_id(self, id: impl Into<String>) -> Self`

Sets the party ID (for grouping players).

```rust
.party_id("party_12345")
```

---

#### `party_size(self, current: i32, max: i32) -> Self`

Sets the party size display.

```rust
.party_size(2, 4)  // Shows "2 of 4"
```

**Parameters:**

- `current` - Current number of players
- `max` - Maximum number of players

---

### Secret Methods

**Note:** Secrets are for join/spectate invitations (advanced feature, partial implementation).

#### `match_secret(self, secret: impl Into<String>) -> Self`

Sets the match secret for joining.

```rust
.match_secret("match_secret_xyz")
```

---

#### `join_secret(self, secret: impl Into<String>) -> Self`

Sets the join secret.

```rust
.join_secret("join_secret_abc")
```

---

#### `spectate_secret(self, secret: impl Into<String>) -> Self`

Sets the spectate secret.

```rust
.spectate_secret("spectate_secret_def")
```

---

### Other Methods

#### `instance(self, is_instance: bool) -> Self`

Sets whether this is a game instance.

```rust
.instance(true)
```

**Default:** `false`

---

#### `build(self) -> Activity`

Builds the final `Activity` object.

```rust
let activity = ActivityBuilder::new()
    .state("Hello")
    .build();
```

**Returns:** `Activity` ready to be sent to Discord

---

## Activity

Represents a Rich Presence activity. Typically created via `ActivityBuilder`.

### Manual Creation

```rust
use presenceforge::activity::{Activity, Assets, Timestamps};

let activity = Activity {
    state: Some("Playing".to_string()),
    details: Some("Main Menu".to_string()),
    timestamps: Some(Timestamps {
        start: Some(1234567890),
        end: None,
    }),
    assets: Some(Assets {
        large_image: Some("logo".to_string()),
        large_text: Some("Game".to_string()),
        small_image: None,
        small_text: None,
    }),
    ..Default::default()
};
```

**Recommendation:** Use `ActivityBuilder` instead of manual creation.

---

## PipeConfig

Configuration for Discord IPC pipe selection.

### Variants

#### `PipeConfig::Auto`

Automatically discovers and connects to the first available Discord pipe.

```rust
use presenceforge::PipeConfig;

let config = PipeConfig::Auto;
```

**Default behavior** when using `DiscordIpcClient::new()`.

---

#### `PipeConfig::CustomPath(String)`

Connects to a specific pipe path.

```rust
use presenceforge::PipeConfig;

// Linux/macOS
let config = PipeConfig::CustomPath("/tmp/discord-ipc-0".to_string());

// Windows
let config = PipeConfig::CustomPath(r"\\.\pipe\discord-ipc-0".to_string());

// Flatpak (Linux)
let config = PipeConfig::CustomPath(
    format!("/run/user/{}/app/com.discordapp.Discord/discord-ipc-0",
    std::env::var("UID").unwrap_or_else(|_| "1000".to_string()))
);
```

---

## IpcConnection

Low-level IPC connection management.

### Discovery Methods

#### `IpcConnection::discover_pipes() -> Vec<DiscoveredPipe>`

Discovers all available Discord IPC pipes on the system.

```rust
use presenceforge::IpcConnection;

let pipes = IpcConnection::discover_pipes();
for pipe in pipes {
    println!("Found pipe {}: {}", pipe.pipe_number, pipe.path);
}
```

**Returns:** Vector of `DiscoveredPipe` structs

**Note:** Automatically checks standard Discord and Flatpak locations on Linux.

---

### DiscoveredPipe

Information about a discovered Discord pipe.

```rust
pub struct DiscoveredPipe {
    pub pipe_number: u8,  // Pipe number (0-9)
    pub path: String,     // Full path to the pipe
}
```

**Example usage:**

```rust
let pipes = IpcConnection::discover_pipes();
if let Some(pipe) = pipes.first() {
    let client = DiscordIpcClient::new_with_config(
        "client_id",
        Some(PipeConfig::CustomPath(pipe.path.clone()))
    )?;
}
```

---

## Error Types

### DiscordIpcError

Main error type for Discord IPC operations.

```rust
pub enum DiscordIpcError {
    ConnectionFailed(String),
    ProtocolError(String),
    SerializationError(String),
    InvalidClientId,
    NotConnected,
    IoError(std::io::Error),
    // ... other variants
}
```

### Common Errors

#### `ConnectionFailed(String)`

Failed to connect to Discord.

**Common causes:**

- Discord is not running
- No Discord pipes available
- Permission issues with IPC socket/pipe

**Example:**

```rust
match DiscordIpcClient::new("client_id") {
    Err(DiscordIpcError::ConnectionFailed(msg)) => {
        eprintln!("Connection failed: {}. Is Discord running?", msg);
    }
    Ok(client) => { /* ... */ }
    Err(e) => eprintln!("Other error: {}", e),
}
```

---

#### `ProtocolError(String)`

Discord IPC protocol error.

**Common causes:**

- Invalid response from Discord
- Protocol version mismatch
- Corrupted data

---

#### `SerializationError(String)`

Failed to serialize/deserialize JSON data.

**Common causes:**

- Invalid activity structure
- Missing required fields
- Type mismatches

---

#### `InvalidClientId`

The provided client ID is invalid.

**Solution:** Verify your Application ID from Discord Developer Portal.

---

#### `NotConnected`

Attempted to send command before connecting.

**Solution:** Call `client.connect()` before setting activities.

```rust
let mut client = DiscordIpcClient::new("client_id")?;
client.connect()?;  // Don't forget this!
client.set_activity(&activity)?;
```

---

### Error Helper Methods

#### `category(&self) -> ErrorCategory`

Returns the error category.

```rust
let category = error.category();
match category {
    ErrorCategory::Connection => println!("Connection problem"),
    ErrorCategory::Protocol => println!("Protocol issue"),
    // ...
}
```

**Categories:**

- `Connection` - Connection-related errors
- `Protocol` - IPC protocol errors
- `Serialization` - JSON serialization errors
- `Application` - Discord application errors
- `Other` - Unspecified errors

---

#### `is_connection_error(&self) -> bool`

Checks if error is connection-related.

```rust
if error.is_connection_error() {
    println!("Tip: Make sure Discord is running!");
}
```

---

#### `is_recoverable(&self) -> bool`

Checks if error might be recoverable.

```rust
if error.is_recoverable() {
    println!("You can try reconnecting");
}
```

---

## Async Clients

Async versions of the IPC client for different runtimes.

### Tokio

```rust
use presenceforge::async_io::tokio::client::new_discord_ipc_client;

#[tokio::main]
async fn main() -> presenceforge::Result {
    let mut client = new_discord_ipc_client("client_id").await?;
    client.connect().await?;

    let activity = ActivityBuilder::new()
        .state("Async Hello!")
        .build();

    client.set_activity(&activity).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    client.clear_activity().await?;
    Ok(())
}
```

**Feature flag:** `tokio-runtime`

---

### async-std

```rust
use presenceforge::async_io::async_std::client::new_discord_ipc_client;

#[async_std::main]
async fn main() -> presenceforge::Result {
    let mut client = new_discord_ipc_client("client_id").await?;
    client.connect().await?;

    let activity = ActivityBuilder::new()
        .state("Async Hello!")
        .build();

    client.set_activity(&activity).await?;

    async_std::task::sleep(std::time::Duration::from_secs(10)).await;

    client.clear_activity().await?;
    Ok(())
}
```

**Feature flag:** `async-std-runtime`

---

### smol

```rust
use presenceforge::async_io::smol::client::new_discord_ipc_client;

fn main() -> presenceforge::Result {
    smol::block_on(async {
        let mut client = new_discord_ipc_client("client_id").await?;
        client.connect().await?;

        let activity = ActivityBuilder::new()
            .state("Async Hello!")
            .build();

        client.set_activity(&activity).await?;

        smol::Timer::after(std::time::Duration::from_secs(10)).await;

        client.clear_activity().await?;
        Ok(())
    })
}
```

**Feature flag:** `smol-runtime`

---

## Type Aliases

```rust
pub type Result<T = (), E = DiscordIpcError> = std::result::Result<T, E>;
```

Convenient type alias for Results with `DiscordIpcError`.

**Usage:**

```rust
use presenceforge::Result;

fn my_function() -> Result {
    // Returns Result<(), DiscordIpcError>
    Ok(())
}

fn returns_value() -> Result<String> {
    // Returns Result<String, DiscordIpcError>
    Ok("Hello".to_string())
}
```

---

## See Also

- [Getting Started Guide](GETTING_STARTED.md)
- [Activity Builder Reference](ACTIVITY_BUILDER_REFERENCE.md)
- [Async Runtimes Guide](ASYNC_RUNTIMES.md)
- [Error Handling Guide](ERROR_HANDLING.md)
- [Examples](../examples/)
