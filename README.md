# PresenceForge

A Rust library for Discord Rich Presence that actually works without the headaches. No more fighting with the Discord SDK or dealing with complex C bindings.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/Sreehari425/presenceforge#license)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
![Crates.io Version](https://img.shields.io/crates/v/presenceforge)

> **Note**: This is currently in development (v0.1.0). Things might break.
> This is a learning/hobby project.
> Features and APIs may change in future versions.

## Documentation

- [Getting Started Guide](docs/GETTING_STARTED.md) - Installation and first steps
- [API Reference](docs/API_REFERENCE.md) - Complete API documentation
- [Activity Builder Reference](docs/ACTIVITY_BUILDER_REFERENCE.md) - Detailed guide to all fields
- [Async Runtimes Guide](docs/ASYNC_RUNTIMES.md) - Using async/await with Tokio, async-std, or smol
- [Error Handling Guide](docs/ERROR_HANDLING.md) - Proper error handling patterns
- [FAQ & Troubleshooting](docs/FAQ.md) - Common questions and solutions

**Want to build your own RPC client?**

- [Discord RPC from Scratch](docs/DISCORD_RPC_FROM_SCRATCH.md) - this is What I found while building this library. I could be wrong about some things, so feel free to correct me!

## What Works

- [x] Linux and macOS (Unix domain sockets)
- [x] Windows support (named pipes)
- [x] Flatpak Discord support (automatic detection)
- [x] Basic Rich Presence activities
- [x] Activity builder pattern
- [x] Images, buttons, and timestamps
- [x] Async support with runtime-agnostic design
- [x] Support for tokio, async-std, and smol
- [x] Flexible pipe/socket selection

## Quick Start

Add PresenceForge to your `Cargo.toml`:

```toml
[dependencies]
presenceforge = "0.1.0"
```

For async support, add one of the runtime features:

```toml
[dependencies]
presenceforge = { version = "0.1.0", features = ["tokio-runtime"] }
# OR
presenceforge = { version = "0.1.0", features = ["async-std-runtime"] }
# OR
presenceforge = { version = "0.1.0", features = ["smol-runtime"] }
```

### Basic Usage (Synchronous)

```rust
use presenceforge::ActivityBuilder;
use presenceforge::sync::DiscordIpcClient;
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
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};

#[tokio::main]
async fn main() -> Result {
    let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
    client.connect().await?;

    let activity = ActivityBuilder::new()
        .state("Playing a game")
        .details("In the menu")
        .start_timestamp_now()
        .large_image("game_logo")
        .large_text("My Awesome Game")
        .build();

    client.set_activity(&activity).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    client.clear_activity().await?;

    Ok(())
}
```

### Async Usage with async-std

```rust
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};
use async_std::task;
use std::time::Duration;

#[async_std::main]
async fn main() -> Result {
    let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
    client.connect().await?;

    let activity = ActivityBuilder::new()
        .state("Playing a game")
        .details("In the menu")
        .start_timestamp_now()
        .large_image("game_logo")
        .large_text("My Awesome Game")
        .build();

    client.set_activity(&activity).await?;
    task::sleep(Duration::from_secs(10)).await;
    client.clear_activity().await?;

    Ok(())
}
```

### Async Usage with smol

```rust
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};
use std::time::Duration;

fn main() -> Result {
    smol::block_on(async {
        let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
        client.connect().await?;

        let activity = ActivityBuilder::new()
            .state("Playing a game")
            .details("In the menu")
            .start_timestamp_now()
            .large_image("game_logo")
            .large_text("My Awesome Game")
            .build();

        client.set_activity(&activity).await?;
        smol::Timer::after(Duration::from_secs(10)).await;
        client.clear_activity().await?;

        Ok(())
    })
}
```

## Examples

### Game Integration

```rust
use presenceforge::ActivityBuilder;
use presenceforge::sync::DiscordIpcClient;
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

### Configuration Methods

All examples support three ways to provide your Discord Client ID:

#### 1. Command-line Argument (Recommended for testing)

```bash
cargo run --example basic -- --client-id YOUR_CLIENT_ID
```

#### 2. Environment Variable

```bash
DISCORD_CLIENT_ID=YOUR_CLIENT_ID cargo run --example basic
```

#### 3. .env File (Recommended for development)

```bash
# Copy the example file
cp .env.example .env

# Edit .env and add your client ID
# DISCORD_CLIENT_ID=your_client_id_here

# Then run any example
cargo run --example basic
```

**Priority Order**: Command-line argument → Environment variable → .env file

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
    .party("id",1, 4)               // Party size (current, max)
    .build()
```

## Running Examples

For detailed information about all available examples and configuration options, see the [Examples README](examples/README.md).

Clone the repository and run the included examples:

```bash
git clone https://github.com/Sreehari425/presenceforge.git
cd presenceforge

# Basic example (synchronous)
cargo run --example basic -- --client-id YOUR_CLIENT_ID

# Game demo with dynamic status
cargo run --example game_demo -- --client-id YOUR_CLIENT_ID

# Developer coding status
cargo run --example coding_status -- --client-id YOUR_CLIENT_ID

# Custom activity without builder
cargo run --example custom_activity -- --client-id YOUR_CLIENT_ID

# Async example with Tokio
cargo run --example async_tokio --features tokio-runtime -- --client-id YOUR_CLIENT_ID

# Async example with async-std
cargo run --example async_std --features async-std-runtime -- --client-id YOUR_CLIENT_ID

# Async example with smol
cargo run --example async_smol --features smol-runtime -- --client-id YOUR_CLIENT_ID

# Complete builder reference - Shows ALL ActivityBuilder options
cargo run --example builder_all -- --client-id YOUR_CLIENT_ID

# Connection retry and error handling
cargo run --example connection_retry -- --client-id YOUR_CLIENT_ID

# Pipe selection and discovery
cargo run --example pipe_selection -- --client-id YOUR_CLIENT_ID
```

Or use the .env file method (recommended for development):

```bash
# Set up .env file once
cp .env.example .env
# Edit .env and add: DISCORD_CLIENT_ID=your_client_id_here

# Then run examples without specifying client ID
cargo run --example basic
cargo run --example game_demo
cargo run --example async_tokio --features tokio-runtime
```

**Note**: Replace `YOUR_CLIENT_ID` with your actual Discord application ID, or use the .env file method for convenience.

## Error Handling

PresenceForge uses the `Result` type for error handling:

```rust
use presenceforge::DiscordIpcError;
use presenceforge::sync::DiscordIpcClient;
match client.connect() {
    Ok(_) => println!("Connected successfully!"),
    Err(DiscordIpcError::ConnectionFailed) => {
        eprintln!("Failed to connect - is Discord running?");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## TODO

- [x] Better error messages
- [ ] Party/lobby functionality (partial implementation)
- [x] Async support (tokio, async-std, and smol)
- [x] More comprehensive examples
- [x] Publish to crates.io
- [ ] CI/CD pipeline
- [x] Proper documentation
- [x] Connection retry logic with exponential backoff
- [x] Activity validation

## License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
