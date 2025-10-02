# PresenceForge

A Rust library for Discord Rich Presence that actually works without the headaches. No more fighting with the Discord SDK or dealing with complex C bindings.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/Sreehari425/presenceforge#license)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)

> **Note**: This is currently in early development (v0.0.0). Things might break

## What Works

- [x] Linux and macOS (Unix domain sockets)
- [x] Basic Rich Presence activities
- [x] Activity builder pattern
- [x] Images, buttons, and timestamps
- [x] Async support with runtime-agnostic design
- [x] Support for tokio, async-std, and smol
- [ ] Windows support (named pipes) - needs testing
- [ ] Error handling could be better
- [ ] Party/lobby features (not implemented yet)

## Quick Start

Add PresenceForge to your `Cargo.toml`:

```toml
[dependencies]
presenceforge = { git = "https://github.com/Sreehari425/presenceforge" }
```

For async support, add one of the runtime features:

```toml
[dependencies]
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["tokio-runtime"] }
# OR
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["async-std-runtime"] }
# OR
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["smol-runtime"] }
```

> **Note**: Not published to crates.io yet. Use the git dependency for now.

### Basic Usage (Synchronous)

```rust
use presenceforge::{DiscordIpcClient, ActivityBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DiscordIpcClient::new("your_client_id")?;
    client.connect()?;

    let activity = ActivityBuilder::new()
        .state("Playing a game")
        .details("In the menu")
        .start_timestamp_now()
        .large_image("game_logo")
        .large_text("My Awesome Game")
        .build();

    client.set_activity(&activity)?;

    // Keep the activity active
    std::thread::sleep(std::time::Duration::from_secs(10));

    client.clear_activity()?;
    Ok(())
}
```

### Async Usage with Tokio

```rust
use presenceforge::{ActivityBuilder, Result};
use presenceforge::async_io::tokio::client::new_discord_ipc_client;

#[tokio::main]
async fn main() -> Result {
    let client_id = "your_client_id";
    let mut client = new_discord_ipc_client(client_id).await?;

    // Perform handshake
    client.connect().await?;

    // Create activity using the builder pattern
    let activity = ActivityBuilder::new()
        .state("Playing a game")
        .details("In the menu")
        .start_timestamp_now()
        .large_image("game_logo")
        .large_text("My Awesome Game")
        .build();

    // Set the activity
    client.set_activity(&activity).await?;

    // Keep activity for some time
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Clear the activity
    client.clear_activity().await?;

    Ok(())
}
```

### Async Usage with async-std

```rust
use presenceforge::{ActivityBuilder, Result};
use presenceforge::async_io::async_std::client::new_discord_ipc_client;
use async_std::task;
use std::time::Duration;

#[async_std::main]
async fn main() -> Result {
    let client_id = "your_client_id";
    let mut client = new_discord_ipc_client(client_id).await?;

    // Perform handshake
    client.connect().await?;

    // Create activity using the builder pattern
    let activity = ActivityBuilder::new()
        .state("Playing a game")
        .details("In the menu")
        .start_timestamp_now()
        .large_image("game_logo")
        .large_text("My Awesome Game")
        .build();

    // Set the activity
    client.set_activity(&activity).await?;

    // Keep activity for some time
    task::sleep(Duration::from_secs(10)).await;

    // Clear the activity
    client.clear_activity().await?;

    Ok(())
}
```

### Async Usage with smol

```rust
use presenceforge::{ActivityBuilder, Result};
use presenceforge::async_io::smol::client::new_discord_ipc_client;
use std::time::Duration;

fn main() -> Result {
    smol::block_on(async {
        let client_id = "your_client_id";
        let mut client = new_discord_ipc_client(client_id).await?;

        // Perform handshake
        client.connect().await?;

        // Create activity using the builder pattern
        let activity = ActivityBuilder::new()
            .state("Playing a game")
            .details("In the menu")
            .start_timestamp_now()
            .large_image("game_logo")
            .large_text("My Awesome Game")
            .build();

        // Set the activity
        client.set_activity(&activity).await?;

        // Keep activity for some time
        smol::Timer::after(Duration::from_secs(10)).await;

        // Clear the activity
        client.clear_activity().await?;

        Ok(())
    })
}
```

## Examples

### Game Integration

```rust
use presenceforge::{ActivityBuilder, DiscordIpcClient};

let activity = ActivityBuilder::new()
    .state("Forest Level")
    .details("Fighting goblins")
    .start_timestamp_now()
    .large_image("forest_map")
    .large_text("Enchanted Forest")
    .small_image("player_avatar")
    .small_text("Level 25 Warrior")
    .button("Play Now", "https://your-game.com")
    .button("Leaderboard", "https://your-game.com/leaderboard")
    .build();

client.set_activity(&activity)?;
```

### Developer Tools

```rust
let activity = ActivityBuilder::new()
    .state("Writing Rust code")
    .details("Building Discord RPC library")
    .start_timestamp_now()
    .large_image("rust_logo")
    .large_text("Rust Programming")
    .small_image("vscode")
    .small_text("VS Code")
    .button("View on GitHub", "https://github.com/your-username/repo")
    .build();
```

## Getting Your Discord Application ID

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application or select an existing one
3. Copy the **Application ID** from the General Information page
4. (Optional) Upload images in the Rich Presence Art Assets section

## Platform Support

| Platform | IPC Method          | Status |
| -------- | ------------------- | ------ |
| Linux    | Unix Domain Sockets | [x]    |
| macOS    | Unix Domain Sockets | [x]    |
| Windows  | Named Pipes         | [x]    |

## API Reference

### Client

- `DiscordIpcClient::new(client_id)` - Create a new client
- `client.connect()` - Connect to Discord
- `client.set_activity(activity)` - Set Rich Presence activity
- `client.clear_activity()` - Clear current activity

### Activity Builder

The `ActivityBuilder` provides a fluent interface for creating activities:

```rust
ActivityBuilder::new()
    .state("Custom state")           // What the player is doing
    .details("Custom details")       // Additional context
    .start_timestamp_now()           // Start time (current)
    .start_timestamp(timestamp)      // Start time (custom)
    .end_timestamp(timestamp)        // End time
    .large_image("image_key")        // Large image asset
    .large_text("Hover text")        // Large image hover text
    .small_image("image_key")        // Small image asset
    .small_text("Hover text")        // Small image hover text
    .button("Label", "https://url")  // Clickable button (max 2)
    .party_size(1, 4)               // Party size (current, max)
    .build()
```

## Running Examples

Clone the repository and run the included examples:

```bash
git clone https://github.com/Sreehari425/presenceforge.git
cd presenceforge

# Basic example (synchronous)
cargo run --example basic

# Game demo with dynamic status
cargo run --example game_demo

# Developer coding status
cargo run --example coding_status

# Custom activity without builder
cargo run --example custom_activity

# Async example with Tokio
cargo run --example async_tokio --features tokio-runtime

# Async example with async-std
cargo run --example async_std --features async-std-runtime

# Async example with smol
cargo run --example async_smol --features smol-runtime
```

Remember to replace `"YOUR-CLIENT-ID"` with your actual Discord application ID.
.

## Error Handling

PresenceForge uses the `Result` type for error handling:

```rust
use presenceforge::{DiscordIpcClient, DiscordIpcError};

match client.connect() {
    Ok(_) => println!("Connected successfully!"),
    Err(DiscordIpcError::ConnectionFailed) => {
        eprintln!("Failed to connect - is Discord running?");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## TODO

- [ ] Better error messages (in progress)
- [ ] Party/lobby functionality
- [x] Async support (tokio and async-std integration)
- [ ] More comprehensive examples
- [ ] Publish to crates.io
- [ ] CI/CD pipeline
- [ ] Property documentation
- [ ] Connection retry logic
- [ ] Activity validation

## License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
