# PresenceForge

A Rust library for Discord Rich Presence that actually works without the headaches. No more fighting with the Discord SDK or dealing with complex C bindings.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/Sreehari425/presenceforge#license)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)

> **Note**: This is currently in early development (v0.0.0). Things might break
> This is a learning/hobby project.
>
> ⚠️ FINAL WARNING: PresenceForge is a learning/hobby project. If you need production-ready Discord Rich Presence, use a mature library like pypresence, discord-rpc, or CraftPresence.

## Features

✨ **Unified Async API** - Write once, run on any async runtime (Tokio, async-std, or smol)  
🚀 **Simple & Ergonomic** - Intuitive builder pattern for creating activities  
🔄 **Runtime Agnostic** - Switch async runtimes with just a feature flag  
🎯 **Cross-Platform** - Works on Linux, macOS, and Windows  
📦 **Zero Config** - Automatic Discord detection (including Flatpak)  
🛡️ **Type Safe** - Compile-time guarantees with Rust's type system

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
- [x] Windows support (named pipes) - needs testing
- [x] Flatpak Discord support (automatic detection)
- [x] Basic Rich Presence activities
- [x] Activity builder pattern
- [x] Images, buttons, and timestamps
- [x] **Unified async API with runtime-agnostic design**
- [x] Support for tokio, async-std, and smol
- [x] Flexible pipe/socket selection
- [ ] Error handling could be better
- [ ] Party/lobby features (partial implementation only)

## Quick Start

Add PresenceForge to your `Cargo.toml`:

```toml
[dependencies]
presenceforge = { git = "https://github.com/Sreehari425/presenceforge" }
```

For async support, add one of the runtime features:

```toml
[dependencies]
# Choose ONE of these based on your async runtime:
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

### Unified Async API

The library provides a single `AsyncDiscordIpcClient` that works with any async runtime!  
**Same code, any runtime** - just change the feature flag. ✨

#### With Tokio

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

#### With async-std

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

#### With smol

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

> 💡 **Pro Tip**: The same async code works across all three runtimes! Just enable the appropriate feature flag in `Cargo.toml`.

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

## Why PresenceForge?

### 🎯 Unified Async API

Unlike other libraries that require different imports for each async runtime, PresenceForge provides a single unified API:

```rust
// ✅ PresenceForge - Same import for all runtimes
use presenceforge::AsyncDiscordIpcClient;

// ❌ Other libraries - Different imports per runtime
use other_lib::tokio::TokioClient;
use other_lib::async_std::AsyncStdClient;
```

**Benefits:**
- **Write Once, Run Anywhere**: Switch runtimes with just a feature flag
- **No Code Changes**: Your application code stays the same
- **Future-Proof**: Easy to migrate between runtimes as your needs change
- **DRY Principle**: Eliminates repetitive runtime-specific code

### 🚀 Simple & Ergonomic

Builder pattern makes creating activities intuitive:

```rust
let activity = ActivityBuilder::new()
    .state("Playing")
    .details("In a match")
    .start_timestamp_now()
    .large_image("logo")
    .build();
```

No complex structs or manual JSON serialization required!

## API Reference

### Synchronous Client

- `DiscordIpcClient::new(client_id)` - Create a new client
- `client.connect()` - Connect to Discord
- `client.set_activity(activity)` - Set Rich Presence activity
- `client.clear_activity()` - Clear current activity

### Async Client

- `AsyncDiscordIpcClient::new(client_id).await` - Create a new async client
- `client.connect().await` - Connect to Discord (async)
- `client.set_activity(activity).await` - Set Rich Presence activity (async)
- `client.clear_activity().await` - Clear current activity (async)

> **Note**: `AsyncDiscordIpcClient` automatically adapts to your chosen runtime (Tokio, async-std, or smol) based on the feature flag you enable.

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

- [x] Better error messages
- [ ] Party/lobby functionality (partial implementation)
- [x] Async support (tokio, async-std, and smol)
- [x] **Unified async API with runtime-agnostic design**
- [x] More comprehensive examples
- [ ] Publish to crates.io
- [ ] CI/CD pipeline
- [x] Proper documentation
- [x] Connection retry logic with exponential backoff
- [x] Activity validation

## License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
