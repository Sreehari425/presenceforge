# PresenceForge Examples

This directory contains examples demonstrating how to use the PresenceForge library for Discord Rich Presence integration.

## Configuration

All examples support three ways to provide your Discord Client ID:

### 1. Command-line Argument (Recommended for testing)

```bash
cargo run --example basic -- --client-id YOUR_CLIENT_ID
```

### 2. Environment Variable

```bash
DISCORD_CLIENT_ID=YOUR_CLIENT_ID cargo run --example basic
```

### 3. .env File (Recommended for development)

```bash
# Copy the example file
cp .env.example .env

# Edit .env and add your client ID
# DISCORD_CLIENT_ID=your_client_id_here

# Then run any example
cargo run --example basic
```

**Priority Order**: Command-line argument → Environment variable → .env file

## Running Examples

```bash
# Basic example - Simple Rich Presence setup (synchronous)
cargo run --example basic -- --client-id YOUR_CLIENT_ID

# Complete Builder Reference - Shows ALL ActivityBuilder options with explanations
cargo run --example builder_all -- --client-id YOUR_CLIENT_ID

# Basic Flatpak - Simple example for Flatpak Discord with custom path
cargo run --example basic_flatpak -- --client-id YOUR_CLIENT_ID

# Game demo - Dynamic game status that changes over time
cargo run --example game_demo -- --client-id YOUR_CLIENT_ID

# Coding status - Developer activity status
cargo run --example coding_status -- --client-id YOUR_CLIENT_ID

# Custom activity - Manual activity creation without builder pattern
cargo run --example custom_activity -- --client-id YOUR_CLIENT_ID

# Async example with Tokio
cargo run --example async_tokio --features tokio-runtime -- --client-id YOUR_CLIENT_ID

# Async example with async-std
cargo run --example async_std --features async-std-runtime -- --client-id YOUR_CLIENT_ID

# Async example with smol
cargo run --example async_smol --features smol-runtime -- --client-id YOUR_CLIENT_ID

# Pipe selection - Discover and select specific Discord pipes
cargo run --example pipe_selection -- --client-id YOUR_CLIENT_ID

# Connection retry - Error handling and reconnection strategies (synchronous)
cargo run --example connection_retry -- --client-id YOUR_CLIENT_ID

# Async Tokio reconnect - Connection retry with Tokio async runtime
cargo run --example async_tokio_reconnect --features tokio-runtime -- --client-id YOUR_CLIENT_ID

# Update activity with Tokio - Update state/details without resetting the timer
cargo run --example update_activity_tokio --features tokio-runtime -- --client-id YOUR_CLIENT_ID

# Flatpak Discord - Connect to Flatpak Discord using custom path configuration
cargo run --example flatpak_discord -- --client-id YOUR_CLIENT_ID
```

## Examples Overview

### `basic.rs`

Simple example showing:

- Basic client setup
- Setting activity with builder pattern
- Using assets and buttons
- Clearing activity

### `builder_all.rs` **Comprehensive Reference**

**Complete ActivityBuilder documentation example** showing:

- Note all methods at the time of writing dosent work
- **Every single builder method** with detailed explanations
- **Visual layout guide** showing where each field appears in Discord
- **Practical tips** for each option (images, timestamps, party, buttons, secrets)
- **What is what**: Clear explanations of confusing terms:
  - What's the difference between large_image and small_image?
  - What do "state" and "details" mean?
  - How do timestamps work (elapsed vs remaining)?
  - What are secrets used for?
  - What does "instance" do?
- **Perfect reference** when you need to know all available options

**Use this example when:** You want to see everything the ActivityBuilder can do!

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

### `connection_retry.rs`

Error handling and recovery example (synchronous) showing:

- Basic retry with `with_retry()` functions
- Manual reconnection using `reconnect()` method
- Custom retry configuration (max attempts, delays, backoff)
- Handling recoverable vs non-recoverable errors
- Long-running connection resilience patterns

### `async_tokio_reconnect.rs`

Async error handling with Tokio showing:

- Using `TokioDiscordIpcClient` with reconnect support
- Manual reconnection in async context
- Automatic retry with `with_retry_async()`
- Resilient connection loop with exponential backoff
- Async error recovery patterns

### `update_activity_tokio.rs`

Activity state updates without timer reset (Tokio async) showing:

- **Updating activity state/details while keeping the same timer**
- Using a consistent `start_timestamp` across all updates
- Demonstrating multiple state changes (main menu → in game → multiplayer → loading → back to menu)
- Perfect for games or apps that need to update status without resetting elapsed time
- Shows how to maintain session continuity across state changes

**Key Feature:** The elapsed time on Discord continues uninterrupted when you update the activity while maintaining the original timestamp!

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
2. **Client ID**: Get your app's client ID from the Discord Developer Portal (see below)
3. **Assets**: Upload images to your Discord app's Rich Presence assets (optional, examples will work without them)
4. **Discord Running**: Make sure Discord is running while testing

## Getting Your Client ID

1. Go to https://discord.com/developers/applications
2. Click "New Application" and give it a name
3. Copy the "Application ID" from the General Information page
4. Use this ID with any of the configuration methods above:
   - Command line: `-- --client-id YOUR_APPLICATION_ID`
   - Environment: `DISCORD_CLIENT_ID=YOUR_APPLICATION_ID`
   - .env file: `DISCORD_CLIENT_ID=YOUR_APPLICATION_ID`

## Asset Keys Used in Examples

The examples reference these asset keys (upload to your Discord app):

- `car` - Example car image
- `rust_logo` - Rust programming language logo
- `vscode` - VS Code editor icon
- `menu_bg`, `forest_map`, `castle_map` - Game backgrounds
- `player_avatar`, `debug_icon` - Game/dev icons
