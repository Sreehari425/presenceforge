# PresenceForge Examples

This directory contains examples demonstrating how to use the PresenceForge library for Discord Rich Presence integration.

## Running Examples

```bash
# Basic example - Simple Rich Presence setup (synchronous)
cargo run --example basic

# Basic Flatpak - Simple example for Flatpak Discord with custom path
cargo run --example basic_flatpak

# Game demo - Dynamic game status that changes over time
cargo run --example game_demo

# Coding status - Developer activity status
cargo run --example coding_status

# Custom activity - Manual activity creation without builder pattern
cargo run --example custom_activity

# Async example with Tokio
cargo run --example async_tokio --features tokio-runtime

# Async example with async-std
cargo run --example async_std --features async-std-runtime

# Async example with smol
cargo run --example async_smol --features smol-runtime

# Pipe selection - Discover and select specific Discord pipes
cargo run --example pipe_selection

# Flatpak Discord - Connect to Flatpak Discord using custom path configuration
cargo run --example flatpak_discord
```

## Examples Overview

### `basic.rs`

Simple example showing:

- Basic client setup
- Setting activity with builder pattern
- Using assets and buttons
- Clearing activity

### `basic_flatpak.rs`

Simple Flatpak Discord example showing:

- Discovering Flatpak Discord pipe
- Connecting using `PipeConfig::CustomPath`
- Same simple structure as `basic.rs` but with custom path configuration
- Perfect starting point for Flatpak Discord integration

**Note:** If Flatpak Discord is not found, the example will show an error. For automatic fallback, use `basic.rs` with auto-discovery.

### `game_demo.rs`

Game integration showing:

- Dynamic status updates
- Multiple game states
- Time-based progression
- Game-specific assets

### `coding_status.rs`

Developer workflow example showing:

- Coding activity status
- Status updates (coding → debugging)
- Development-focused assets

### `custom_activity.rs`

Low-level example showing:

- Manual `Activity` struct creation
- Full control over all fields
- Advanced customization

### `async_tokio.rs`

Async example with Tokio showing:

- Asynchronous client setup with Tokio
- Async/await pattern for Discord IPC
- Using Tokio's async runtime
- Using the same Activity builder API

### `async_std.rs`

Async example with async-std showing:

- Asynchronous client setup with async-std
- Async/await pattern for Discord IPC
- Using async-std's async runtime
- Using the same Activity builder API

### `async_smol.rs`

Async example with smol showing:

- Asynchronous client setup with smol
- Async/await pattern for Discord IPC
- Using smol's lightweight async runtime
- Using `smol::block_on()` and `smol::Timer`
- Using the same Activity builder API

### `pipe_selection.rs`

Advanced pipe discovery example showing:

- Discovering all available Discord IPC pipes
- Connecting using auto-discovery (default)
- Connecting to specific pipes using custom paths
- Connection with timeout configuration
- Working with both Unix sockets and Windows named pipes

### `flatpak_discord.rs`

Flatpak Discord example showing:

- Discovering Flatpak Discord installations
- Identifying Flatpak vs standard Discord pipes
- Connecting using custom path configuration (`PipeConfig::CustomPath`)
- Setting activity on Flatpak Discord
- Step-by-step guide with detailed output

**Note:** This example works with both Flatpak and standard Discord. It automatically detects which version is available.

## Prerequisites

1. **Discord Application**: Create one at https://discord.com/developers/applications
2. **Client ID**: Replace `"YOUR-CLIENT-ID"` with your app's client ID from the Discord Developer Portal
3. **Assets**: Upload images to your Discord app's Rich Presence assets (optional, examples will work without them)
4. **Discord Running**: Make sure Discord is running while testing

## Getting Your Client ID

1. Go to https://discord.com/developers/applications
2. Click "New Application" and give it a name
3. Copy the "Application ID" from the General Information page
4. Replace `"YOUR-CLIENT-ID"` in the examples with this ID

## Asset Keys Used in Examples

The examples reference these asset keys (upload to your Discord app):

- `car` - Example car image
- `rust_logo` - Rust programming language logo
- `vscode` - VS Code editor icon
- `menu_bg`, `forest_map`, `castle_map` - Game backgrounds
- `player_avatar`, `debug_icon` - Game/dev icons
