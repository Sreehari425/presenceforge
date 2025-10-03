# Getting Started with PresenceForge

Welcome to PresenceForge! This guide will help you get started with integrating Discord Rich Presence into your Rust application.

> ⚠️ **WARNING:** PresenceForge is an experimental, hobby project (v0.0.0). Features are partially tested, may break, and should **not** be used in production.
> ⚠️ **NOTE:** This feature is experimental/untested. Use at your own risk.

## What is Discord Rich Presence?

Discord Rich Presence allows your application to display custom status information in Discord, showing users what you're doing in real-time. This can include:

- Current game status or activity
- Time elapsed or remaining
- Custom images and text
- Party/group information
- Interactive buttons

## Prerequisites

Before you start, you'll need:

1. **Rust 1.70 or later** - Install from [rustup.rs](https://rustup.rs/)
2. **A Discord Application** - Create one at [Discord Developer Portal](https://discord.com/developers/applications)
3. **Discord Running** - The Discord client must be running on your system

## Creating a Discord Application

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application" and give it a name
3. Copy your **Application ID** (also called Client ID) - you'll need this!
4. (Optional) Under "Rich Presence" → "Art Assets", upload images for your presence

## Installation

Add PresenceForge to your `Cargo.toml`:

```toml
[dependencies]
presenceforge = { git = "https://github.com/Sreehari425/presenceforge" }
```

> **Note**: PresenceForge is not yet published to crates.io. Use the git dependency for now.

### With Async Support

If you need async support, add one of the runtime features:

```toml
[dependencies]
# For Tokio users
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["tokio-runtime"] }

# For async-std users
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["async-std-runtime"] }

# For smol users
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["smol-runtime"] }
```

## Your First Rich Presence

Let's create a simple Rich Presence that displays "Hello, Discord!" in your profile.

### Step 1: Create a new Rust project

```bash
cargo new my-discord-presence
cd my-discord-presence
```

### Step 2: Add PresenceForge to Cargo.toml

```toml
[dependencies]
presenceforge = { git = "https://github.com/Sreehari425/presenceforge" }
```

### Step 3: Write your first presence

Edit `src/main.rs`:

```rust
use presenceforge::{DiscordIpcClient, ActivityBuilder};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Replace with your Discord Application ID
    let client_id = "YOUR_CLIENT_ID_HERE";

    // Create and connect to Discord
    let mut client = DiscordIpcClient::new(client_id)?;
    client.connect()?;

    println!("Connected to Discord!");

    // Create your activity
    let activity = ActivityBuilder::new()
        .state("Hello, Discord!")
        .details("Using PresenceForge")
        .start_timestamp_now()
        .build();

    // Set the activity
    client.set_activity(&activity)?;

    println!("Rich Presence is now active! Check your Discord profile.");
    println!("Press Ctrl+C to stop...");

    // Keep the presence active
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
```

### Step 4: Run it!

```bash
cargo run
```

You should see "Connected to Discord!" and your Discord profile should now show your custom Rich Presence!

## Understanding the Code

Let's break down what each part does:

```rust
// 1. Create a client with your Application ID
let mut client = DiscordIpcClient::new(client_id)?;

// 2. Connect to Discord (this performs the handshake)
client.connect()?;

// 3. Build your activity with the builder pattern
let activity = ActivityBuilder::new()
    .state("Hello, Discord!")          // Smaller text line
    .details("Using PresenceForge")    // Larger text line
    .start_timestamp_now()              // Shows "elapsed" time
    .build();

// 4. Send the activity to Discord
client.set_activity(&activity)?;
```

## Adding More Features

### Adding Images

First, upload your images to Discord Developer Portal → Rich Presence → Art Assets. Then use them:

```rust
let activity = ActivityBuilder::new()
    .state("In Game")
    .details("Playing Adventure Mode")
    .large_image("game_logo")           // Asset key from Developer Portal
    .large_text("My Awesome Game")      // Hover text
    .small_image("character_icon")      // Small circular overlay
    .small_text("Level 50 Warrior")     // Hover text for small image
    .build();
```

### Adding Buttons

```rust
let activity = ActivityBuilder::new()
    .state("Streaming")
    .details("Making cool stuff")
    .add_button("Watch Stream", "https://twitch.tv/username")
    .add_button("GitHub", "https://github.com/username")
    .build();
```

### Using Timestamps

```rust
use std::time::{SystemTime, UNIX_EPOCH};

// Show elapsed time (started 5 minutes ago)
let five_min_ago = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() - 300;

let activity = ActivityBuilder::new()
    .state("In Match")
    .start_timestamp(five_min_ago)  // Shows "00:05 elapsed"
    .build();

// Show remaining time (ends in 10 minutes)
let in_ten_min = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() + 600;

let activity = ActivityBuilder::new()
    .state("Match Ending Soon")
    .end_timestamp(in_ten_min)  // Shows "10:00 left"
    .build();
```

## Updating Your Presence

You can update your presence at any time by calling `set_activity()` again:

```rust
// Initial presence
client.set_activity(&ActivityBuilder::new()
    .state("In Lobby")
    .build())?;

thread::sleep(Duration::from_secs(5));

// Update to show you're in a match
client.set_activity(&ActivityBuilder::new()
    .state("In Match")
    .details("Competitive Mode")
    .start_timestamp_now()
    .build())?;
```

## Clearing Your Presence

When you're done, clear the presence:

```rust
client.clear_activity()?;
```

This removes your Rich Presence from Discord entirely.

## Error Handling

Always handle errors properly. Here's a better main function:

```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        eprintln!("\nTroubleshooting:");
        eprintln!("- Is Discord running?");
        eprintln!("- Is your client ID correct?");
        eprintln!("- On Linux: Check that /tmp/discord-ipc-0 exists");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DiscordIpcClient::new("your_client_id")?;
    client.connect()?;

    let activity = ActivityBuilder::new()
        .state("Hello!")
        .build();

    client.set_activity(&activity)?;

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
```

## Common Issues

### "Connection Failed" Error

**Problem**: Can't connect to Discord

**Solutions**:

- Make sure Discord is running
- Try restarting Discord
- On Linux: Check if `/tmp/discord-ipc-0` or equivalent exists

### "Invalid Client ID" Error

**Problem**: Discord doesn't recognize your Application ID

**Solutions**:

- Double-check your Application ID from Discord Developer Portal
- Make sure you're using the Application ID, not a different ID
- The ID should be a long number (e.g., "1234567890123456789")

### Presence Doesn't Show Up

**Problem**: No error but presence isn't visible

**Solutions**:

- Check your Discord Activity Privacy settings
- Make sure to set your client_id

## Next Steps

Now that you have the basics working, explore more features:

- **[Activity Builder Reference](ACTIVITY_BUILDER_REFERENCE.md)** - Guide to ActivityBuilder options
- **[Async Runtimes](ASYNC_RUNTIMES.md)** - Using async/await with Tokio, async-std, or smol
- **[API Reference](API_REFERENCE.md)** - API documentation (WIP)
- **[Examples](../examples/)** - More code examples

## Need Help?

- Check the [FAQ and Troubleshooting Guide](FAQ.md)
- Look at the [examples directory](../examples/)
- Open an issue on [GitHub](https://github.com/Sreehari425/presenceforge/issues)

Happy coding! 🦀✨
