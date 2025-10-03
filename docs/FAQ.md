# FAQ and Troubleshooting

Frequently asked questions and solutions to common problems.

## Table of Contents

- [General Questions](#general-questions)
- [Installation Issues](#installation-issues)
- [Connection Problems](#connection-problems)
- [Activity Display Issues](#activity-display-issues)
- [Platform-Specific Issues](#platform-specific-issues)
- [Development Questions](#development-questions)

---

## General Questions

### What is PresenceForge?

PresenceForge is a Rust library for integrating Discord Rich Presence into your applications. It allows you to display custom status information in Discord, showing what users are doing in real-time.


---

### What platforms are supported?

- Linux (standard and Flatpak Discord)
- macOS (need testing)
- Windows (needs more testing)

---

## Installation Issues

### Error: "package not found" when using git dependency

**Problem:**

```bash
error: failed to get `presenceforge` as a dependency of package `my_app`
```

**Solution:**
Make sure you have git installed and can access GitHub:

```bash
# Test GitHub access
git ls-remote https://github.com/Sreehari425/presenceforge.git

# If that fails, try SSH
git ls-remote git@github.com:Sreehari425/presenceforge.git
```

If SSH works, use this in `Cargo.toml`:

```toml
[dependencies]
presenceforge = { git = "ssh://git@github.com/Sreehari425/presenceforge.git" }
```

---

### Feature flag errors

**Problem:**

```bash
error: Package `presenceforge` does not have feature `tokio`
```

**Solution:**
The feature is called `tokio-runtime`, not `tokio`:

```toml
[dependencies]
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["tokio-runtime"] }
```

Valid features: `tokio-runtime`, `async-std-runtime`, `smol-runtime`

---

### Conflicting dependencies

**Problem:**

```bash
error: failed to select a version for `tokio`
```

**Solution:**
Make sure you're using compatible versions:

```toml
[dependencies]
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["tokio-runtime"] }
tokio = { version = "1", features = ["full"] }  # Use version 1.x
```

---

## Connection Problems

### "Connection Failed" - Discord not found

**Problem:**

```rust
Error: ConnectionFailed("No Discord pipes found")
```

**Solutions:**

1. **Check Discord is running:**

   ```bash
   # Linux/macOS
   ps aux | grep -i discord

   # Windows
   tasklist | findstr discord
   ```

2. **Check IPC socket exists:**

   ```bash
   # Linux (standard)
   ls -l $XDG_RUNTIME_DIR/discord-ipc-*

   # Linux (Flatpak)
   ls -l /run/user/$(id -u)/app/com.discordapp.Discord/discord-ipc-*

   # macOS
   ls -l /tmp/discord-ipc-* $TMPDIR/discord-ipc-*

   # Windows (PowerShell)
   Get-ChildItem \\.\pipe\ | Select-String discord
   ```

3. **Try restarting Discord**

4. **For Flatpak users:** See [Platform-Specific Guide](PLATFORM_SPECIFIC.md#flatpak-discord)

---

### Connection works but handshake fails

**Problem:**

```rust
Error: ProtocolError("Handshake failed")
```

**Solutions:**

1. **Update Discord** to the latest version
2. **Check your Client ID** is correct
3. **Restart both** Discord and your application
4. **Try a different pipe:**

   ```rust
   use presenceforge::{IpcConnection, PipeConfig, DiscordIpcClient};

   let pipes = IpcConnection::discover_pipes();
   for pipe in pipes {
       println!("Trying pipe: {}", pipe.path);
       if let Ok(mut client) = DiscordIpcClient::new_with_config(
           "client_id",
           Some(PipeConfig::CustomPath(pipe.path))
       ) {
           if client.connect().is_ok() {
               println!("Success!");
               break;
           }
       }
   }
   ```

---

### "Permission Denied" error

**Problem:**

```rust
Error: IoError(Os { code: 13, kind: PermissionDenied, message: "Permission denied" })
```

**Solutions:**

**Linux:**

```bash
# Check socket permissions
ls -l$ XDG_RUNTIME_DIR/discord-ipc-*

# Should be owned by your user
# If not, restart Discord
```

---

### Connection drops randomly

**Problem:** Connection works initially but drops after a while.

**Solutions:**

1. **Implement reconnection logic:**

   ```rust
   loop {
       match client.set_activity(&activity) {
           Ok(_) => println!("Updated!"),
           Err(e) if e.is_connection_error() => {
               eprintln!("Connection lost, reconnecting...");
               client.reconnect()?;
               client.set_activity(&activity)?;
           }
           Err(e) => return Err(e.into()),
       }

       std::thread::sleep(Duration::from_secs(15));
   }
   ```

2. **Send updates regularly** (every 15-60 seconds) to keep connection alive

3. **Handle Discord restarts gracefully**

---

## Activity Display Issues

### Activity doesn't show up in Discord

**Problem:** No errors but Rich Presence isn't visible.

**Solutions:**

1. **Check Activity Privacy Settings:**

   - Discord Settings → Activity Privacy
   - Enable "Display current activity as a status message"

2. **Wait a few seconds** - It can take 1-5 seconds to appear

3. **Check you called connect():**

   ```rust
   let mut client = DiscordIpcClient::new("client_id")?;
   client.connect()?;  // Don't forget this!
   client.set_activity(&activity)?;
   ```

4. **Verify your Client ID is correct**

5. **Make sure your app stays running** - presence disappears when app exits

---

### Images don't show up

**Problem:** Activity shows but images are missing.

**Solutions:**

1. **Upload images to Discord Developer Portal:**

   - Go to https://discord.com/developers/applications
   - Select your application
   - Go to "Rich Presence" → "Art Assets"
   - Upload your images

2. **Use the correct asset key:**

   ```rust
   // Use the "name" you gave the asset, not the filename
   .large_image("game_logo")  //  Asset name
   .large_image("logo.png")   //  Filename won't work
   ```

3. **Wait for assets to propagate** - Can take a few minutes after upload

4. **Image requirements:**
   - Format: PNG or JPG
   - Recommended size: 1024x1024 (large), 256x256 (small)
   - Max file size: 5MB

---

### Timestamps showing wrong time

**Problem:** "Elapsed" or "Left" time is incorrect.

**Solutions:**

1. **Use Unix timestamps (seconds, not milliseconds):**

   ```rust
   use std::time::{SystemTime, UNIX_EPOCH};

   let now = SystemTime::now()
       .duration_since(UNIX_EPOCH)
       .unwrap()
       .as_secs() as i64;  //  as_secs(), not as_millis()

   .start_timestamp(now)
   ```

2. **Use convenience method:**

   ```rust
   .start_timestamp_now()  // Automatically uses correct format
   ```

3. **Check your system time is correct:**
   ```bash
   date
   ```

---

### Buttons not working

**Problem:** Buttons show up but don't work.

**Solutions:**

1. **Use full URLs:**

   ```rust
   .add_button("Website", "https://example.com")  //
   .add_button("Website", "example.com")          //
   ```

2. **Check URL is valid** - Must be http:// or https://

3. **Max 2 buttons** - Discord only shows the first 2 buttons

4. **Button label limits:**
   - Max 32 characters per button label
   - Use concise text

---

### State/Details text cut off

**Problem:** Text is truncated with "..."

**Solution:**

Keep text under character limits:

- State: 128 characters max
- Details: 128 characters max

```rust
let long_text = "Very long text...".to_string();

// Truncate if needed
let state = if long_text.len() > 128 {
    format!("{}...", &long_text[..125])
} else {
    long_text
};

.state(state)
```

---

## Platform-Specific Issues

### Linux: Flatpak Discord not detected

**Problem:** Using Flatpak Discord but auto-detection fails.

**Solution:**

Manually specify the Flatpak path:

```rust
use presenceforge::{DiscordIpcClient, PipeConfig};

let uid = std::env::var("UID")
    .or_else(|_| std::fs::read_to_string("/proc/self/loginuid")
        .map(|s| s.trim().to_string()))
    .unwrap_or_else(|_| "1000".to_string());

let path = format!(
    "/run/user/{}/app/com.discordapp.Discord/discord-ipc-0",
    uid
);

let mut client = DiscordIpcClient::new_with_config(
    "client_id",
    Some(PipeConfig::CustomPath(path))
)?;
```

See [Platform-Specific Guide](PLATFORM_SPECIFIC.md#flatpak-discord) for more.

---

---

## Development Questions

### How do I get a Discord Application ID?

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application"
3. Give it a name
4. Copy the "Application ID" from the General Information page

That's your Client ID!

---

### Can I test without uploading images?

Yes! Text-only activities work fine:

```rust
let activity = ActivityBuilder::new()
    .state("Testing")
    .details("No images needed")
    .build();
```

Images are optional. Upload them when you're ready.

---

### How often should I update presence?

**Recommendations:**

- **Update on significant changes** (new level, different task, etc.)
- **Periodic updates:** Every 15-60 seconds to keep connection alive
- **Don't spam:** Avoid updates more than once per second

```rust
use std::time::Duration;

loop {
    client.set_activity(&activity)?;
    thread::sleep(Duration::from_secs(15));  // Good
    // thread::sleep(Duration::from_millis(100));  // Too fast!
}
```

---

### Can I show party/multiplayer info?

Yes! Use party methods:

```rust
let activity = ActivityBuilder::new()
    .state("In a Party")
    .party_size(2, 4)      // 2 of 4 players
    .party_id("party123")  // Unique party ID
    .build();
```

**Note:** Full party/lobby features are partially implemented.

---

### How do I handle multiple Discord accounts?

Discord IPC connects to the currently active Discord client. If multiple Discord clients are running:

```rust
use presenceforge::IpcConnection;

// Discover all available pipes
let pipes = IpcConnection::discover_pipes();
println!("Found {} Discord client(s)", pipes.len());

// Connect to a specific one
let client = DiscordIpcClient::new_with_config(
    "client_id",
    Some(PipeConfig::CustomPath(pipes[0].path.clone()))
)?;
```

=

### How do I debug IPC communication?

Enable debug output (if implemented):

```rust
// Check if sockets/pipes exist
#[cfg(unix)]
{
    println!("Checking for Discord sockets:");
    for i in 0..10 {
        let path = format!("/tmp/discord-ipc-{}", i);
        if std::path::Path::new(&path).exists() {
            println!("  Found: {}", path);
        }
    }
}

// Try discovery
use presenceforge::IpcConnection;
let pipes = IpcConnection::discover_pipes();
println!("Discovered {} pipe(s):", pipes.len());
for pipe in pipes {
    println!("  Pipe {}: {}", pipe.pipe_number, pipe.path);
}
```

## Still Having Issues?

If your problem isn't listed here:

1. **Check the other docs:**

   - [Getting Started Guide](GETTING_STARTED.md)
   - [API Reference](API_REFERENCE.md)
   - [Error Handling Guide](ERROR_HANDLING.md)

2. **Look at examples:**

   - [Examples directory](../examples/)
   - Start with `basic.rs` for sync
   - Try `async_tokio.rs` for async

3. **Open an issue:**

   - [GitHub Issues](https://github.com/Sreehari425/presenceforge/issues)
   - Include: OS, Discord version, error messages, code sample

4. **Check Discord's documentation:**
   - [Discord Developer Portal](https://discord.com/developers/docs/rich-presence/how-to)

---

## Contributing

Found a solution not listed here? [Contribute to the docs!](https://github.com/Sreehari425/presenceforge)
