# PresenceForge Examples

This directory contains examples demonstrating how to use the PresenceForge library for Discord Rich Presence integration.

## Running Examples

```bash
# Basic example - Simple Rich Presence setup
cargo run --example basic

# Game demo - Dynamic game status that changes over time
cargo run --example game_demo

# Coding status - Developer activity status
cargo run --example coding_status

# Custom activity - Manual activity creation without builder pattern
cargo run --example custom_activity
```

## Examples Overview

### `basic.rs`

Simple example showing:

- Basic client setup
- Setting activity with builder pattern
- Using assets and buttons
- Clearing activity

### `game_demo.rs`

integration showing:

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
