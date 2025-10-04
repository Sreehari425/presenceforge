# Discord RPC from Scratch: A Complete Beginner's Guide

> _"In which we learn to speak Discord's secret language, one JSON packet at a time"_

> **Note:** This guide is based on what I found while experimenting with Discord RPC. I could be wrong about some things, so feel free to correct me! :)

Welcome, brave soul! You've decided to venture into the mystical realm of Discord Rich Presence Protocol (RPC). By the end of this guide, you'll understand every single byte that flows through those Unix sockets (or named pipes, if you're on Windows). Let's build a simple, synchronous Discord RPC client together!

## Table of Contents

1. [What Even Is Discord RPC?](#what-even-is-discord-rpc)
2. [The Big Picture](#the-big-picture)
3. [Step 0: Understanding IPC](#step-0-understanding-ipc)
4. [Step 1: Finding Discord](#step-1-finding-discord)
5. [Step 2: Connecting to Discord](#step-2-connecting-to-discord)
6. [Step 3: The Handshake](#step-3-the-handshake)
7. [Step 4: Setting Your Presence](#step-4-setting-your-presence)
8. [Step 5: Keeping the Connection Alive](#step-5-keeping-the-connection-alive)
9. [Step 6: Graceful Shutdown](#step-6-graceful-shutdown)
10. [Complete Example Code](#complete-example-code)
11. [Troubleshooting](#troubleshooting)

---

## What Even Is Discord RPC?

Discord Rich Presence (RPC) is a way for applications to tell Discord:

- "Hey, I'm running right now!"
- "The user is doing X thing"
- "Here's some cool info to show on their profile"

You know when you see someone playing a game and their Discord status shows fancy details like "In a match" or "Level 42"? That's Rich Presence in action!

```
┌─────────────────────────────────┐
│  Your Application               │
│  "I'm a music player!"          │
│         │                       │
│         │ (sends RPC messages)  │
│         ▼                       │
│  ┌──────────────┐               │
│  │ Discord IPC  │               │
│  └──────────────┘               │
└──────────┬──────────────────────┘
           │
           ▼
   ┌────────────────┐
   │ Discord Client │ ──► Shows on your profile!
   └────────────────┘
```

---

## The Big Picture

Before we dive into code, let's understand the flow:

```
┌───────────────────────────────────────────────┐
│           Your Application                    │
└───────────────────┬───────────────────────────┘
                    │
                    │ 1. Find Discord's socket/pipe
                    ▼
┌───────────────────────────────────────────────┐
│      Unix Socket / Named Pipe                 │
│   (Linux/Mac: /tmp/discord-ipc-0)             │
│   (Windows: \\.\pipe\discord-ipc-0)           │
└───────────────────┬───────────────────────────┘
                    │
                    │ 2. Connect
                    ▼
┌───────────────────────────────────────────────┐
│                                               │
│  3. Handshake (send your app ID)              │
│     You: "Hi! I'm app 123456789"              │
│     Discord: "Cool! Welcome!"                 │
│                                               │
│  4. Set Presence (send activity data)         │
│     You: "User is listening to Spotify"       │
│     Discord: "Got it! Updating..."            │
│                                               │
│  5. Keep connection alive (optional)          │
│                                               │
│  6. Close when done                           │
│                                               │
└───────────────────────────────────────────────┘
```

**Key concept:** Discord RPC uses **IPC** (Inter-Process Communication). Your app talks to the Discord client running on the same computer. It's local, fast, and doesn't need the internet!

---

## Step 0: Understanding IPC

### What's IPC?

IPC stands for Inter-Process Communication. It's how two programs on the same computer talk to each other. Think of it like two friends passing notes in class, except the "notes" are binary data packets.

### The Discord Protocol

Discord uses a simple framing protocol:

```

Every message has two parts:
┌──────────────┬────────────────────────────┐
│   Header     │         Payload            │
│  (8 bytes)   │    (variable length)       │
└──────────────┴────────────────────────────┘

Header breakdown:
┌──────────────┬────────────────────────────┐
│   Opcode     │         Length             │
│  (4 bytes)   │       (4 bytes)            │
│  (u32, LE)   │       (u32, LE)            │
└──────────────┴────────────────────────────┘

```

- **Opcode**: What kind of message is this? (handshake, frame, close, etc.)
- **Length**: How many bytes is the payload?
- **Payload**: The actual JSON data (UTF-8 encoded)

**Important:** Numbers are **little-endian** (LE). If you don't know what that means, don't worry! Your programming language probably handles it automatically.

### Opcodes

Discord defines these opcodes:

```rust
Opcode 0 = HANDSHAKE    // First message you send
Opcode 1 = FRAME        // Regular messages (set presence, etc.)
Opcode 2 = CLOSE        // Closing the connection
Opcode 3 = PING         // Keep-alive ping
Opcode 4 = PONG         // Keep-alive response
```

For a basic client, we mainly use:

- **HANDSHAKE** (opcode 0): To introduce ourselves
- **FRAME** (opcode 1): To send commands and receive responses

---

## Step 1: Finding Discord

Discord creates IPC endpoints (sockets/pipes) that your app needs to find. It creates up to 10 of them, numbered 0-9.

### On Linux/macOS

Discord creates Unix sockets in `/tmp/` or `$XDG_RUNTIME_DIR`:

```
/tmp/discord-ipc-0
/tmp/discord-ipc-1
/tmp/discord-ipc-2
... up to discord-ipc-9
```

Sometimes they're in:

```
$XDG_RUNTIME_DIR/discord-ipc-0
```

Or if you're using Flatpak:

```
$XDG_RUNTIME_DIR/app/com.discordapp.Discord/discord-ipc-0
```

### On Windows

Discord creates named pipes:

```
\\.\pipe\discord-ipc-0
\\.\pipe\discord-ipc-1
\\.\pipe\discord-ipc-2
... up to discord-ipc-9
```

### Finding Algorithm

Here's the logic (pseudocode):

```
for i in 0..10:
    pipe_path = get_pipe_path(i)  // Platform-specific
    if can_connect(pipe_path):
        return pipe_path

raise Error("Discord not found! Is it running?")
```

**Pro tip:** Discord usually uses `discord-ipc-0`, but if multiple Discord instances are running (or it's restarted), it might use a higher number.

---

## Step 2: Connecting to Discord

Once you've found the pipe/socket, you need to connect to it.

### The Connection ASCII Art

```
Your App                    Discord Socket
   │                              │
   │  "Knock knock!"              │
   ├─────────────────────────────>│
   │                              │
   │  "Who's there?"              │
   │<─────────────────────────────┤
   │                              │
   │  [Connection established]    │
   │                              │
```

### Platform-Specific Connection

**On Linux/macOS:**

- Open the Unix socket
- It's just a file! (Everything is a file in Unix)
- Use `std::os::unix::net::UnixStream` in Rust

**On Windows:**

- Open the named pipe using Windows API
- Use `std::os::windows::io` or a wrapper
- Named pipes are... special (Windows is quirky like that)

### What You Need

At this point, you have:

- An open connection to Discord
- A way to read bytes from it
- A way to write bytes to it

Think of it like a two-way telephone line. Now let's start talking!

---

## Step 3: The Handshake

The handshake is your introduction to Discord. It's like saying "Hi, I'm Bob from App #123456789."

### Handshake Packet Structure

```
┌─────────────────────────────────────────────────────┐
│                    HANDSHAKE PACKET                 │
├─────────────────────────────────────────────────────┤
│  Header:                                            │
│    Opcode:  0 (HANDSHAKE)                           │
│    Length:  <size of JSON payload>                  │
│                                                     │
│  Payload (JSON):                                    │
│    {                                                │
│      "v": 1,                                        │
│      "client_id": "your_app_id_here"                │
│    }                                                │
└─────────────────────────────────────────────────────┘
```

### Detailed Breakdown

Let's say your Discord application ID is `1234567890123456789`.

**Step 1:** Create the JSON payload:

```json
{
  "v": 1,
  "client_id": "1234567890123456789"
}
```

- `v`: Protocol version (always 1)
- `client_id`: Your Discord application ID (as a string!)

**Step 2:** Encode the JSON to bytes:

```
JSON string: {"v":1,"client_id":"1234567890123456789"}
Bytes: [123, 34, 118, 34, 58, 49, 44, 34, 99, ...]
Length: 44 bytes (example)
```

**Step 3:** Create the header:

```
Opcode: 0x00000000 (4 bytes, little-endian)
Length: 0x2C000000 (44 in hex is 0x2C, little-endian)

In bytes: [00, 00, 00, 00, 2C, 00, 00, 00]
           └───opcode───┘ └────length────┘
```

**Step 4:** Combine header + payload:

```
[00, 00, 00, 00, 2C, 00, 00, 00, 123, 34, 118, ...]
 └────header────┘ └──────payload starts────────>
```

**Step 5:** Write these bytes to the socket!

### Discord's Response

Discord will respond with its own packet:

```
┌─────────────────────────────────────────────────────┐
│                  RESPONSE PACKET                    │
├─────────────────────────────────────────────────────┤
│  Header:                                            │
│    Opcode:  1 (FRAME)                               │
│    Length:  <size of JSON payload>                  │
│                                                     │
│  Payload (JSON):                                    │
│    {                                                │
│      "cmd": "DISPATCH",                             │
│      "data": {                                      │
│        "v": 1,                                      │
│        "config": { ... },                           │
│        "user": {                                    │
│          "id": "user_id",                           │
│          "username": "CoolPerson",                  │
│          "discriminator": "0001",                   │
│          "avatar": "avatar_hash"                    │
│        }                                            │
│      },                                             │
│      "evt": "READY",                                │
│      "nonce": null                                  │
│    }                                                │
└─────────────────────────────────────────────────────┘
```

If you get `"evt": "READY"`, congratulations! 🎉 You're connected!

If you get an error, Discord will send:

```json
{
  "cmd": "DISPATCH",
  "evt": "ERROR",
  "data": {
    "code": 4000,
    "message": "Invalid Client ID"
  }
}
```

Common error codes:

- `4000`: Invalid Client ID (check your app ID!)
- `4001`: Invalid Origin (shouldn't happen with IPC)
- `4002`: Rate limited (slow down, partner!)
- found through testing btw :)

---

## Step 4: Setting Your Presence

Now the fun part! Let's tell Discord what you're doing.

### SET_ACTIVITY Command

This is how you update your presence:

```
┌─────────────────────────────────────────────────────┐
│               SET_ACTIVITY PACKET                   │
├─────────────────────────────────────────────────────┤
│  Header:                                            │
│    Opcode:  1 (FRAME)                               │
│    Length:  <size of JSON payload>                  │
│                                                     │
│  Payload (JSON):                                    │
│    {                                                │
│      "cmd": "SET_ACTIVITY",                         │
│      "args": {                                      │
│        "pid": 12345,                                │
│        "activity": {                                │
│          "state": "In a match",                     │
│          "details": "Playing as Tank",              │
│          "timestamps": {                            │
│            "start": 1234567890                      │
│          },                                         │
│          "assets": {                                │
│            "large_image": "game_logo",              │
│            "large_text": "My Cool Game",            │
│            "small_image": "character_icon",         │
│            "small_text": "Tank Class"               │
│          },                                         │
│          "party": {                                 │
│            "id": "party_id",                        │
│            "size": [2, 4]                           │
│          },                                         │
│          "buttons": [                               │
│            {                                        │
│              "label": "Join Game",                  │
│              "url": "https://game.com/join"         │
│            }                                        │
│          ]                                          │
│        }                                            │
│      },                                             │
│      "nonce": "unique-id-12345"                     │
│    }                                                │
└─────────────────────────────────────────────────────┘
```

### Field Breakdown

Let's explain each field:

#### Top-level fields:

- **`cmd`**: Always `"SET_ACTIVITY"` for presence updates
- **`args`**: The arguments object containing your activity
- **`nonce`**: A unique identifier for this request (optional, but good for tracking responses)

#### Inside `args`:

- **`pid`**: Your application's process ID

  - Get it with `std::process::id()` in Rust
  - Or `os.getpid()` in Python
  - This helps Discord know your app is actually running

- **`activity`**: The juicy part! Contains all the presence info

#### Inside `activity`:

**Text fields:**

- **`state`**: Small text at the bottom (e.g., "In a match")
- **`details`**: Larger text at the top (e.g., "Playing as Tank")
- Max 128 characters each

**Timestamps:**

```json
"timestamps": {
  "start": 1234567890,  // Unix timestamp (seconds since 1970)
  "end": 1234599999     // Optional: when the activity ends
}
```

Discord will show "elapsed" or "remaining" time based on these!

**Assets (images):**

```json
"assets": {
  "large_image": "key_name",      // Image key from Developer Portal
  "large_text": "Hover text",     // Shows on hover
  "small_image": "key_name",      // Small image (bottom-right)
  "small_text": "Hover text"      // Shows on hover
}
```

**Important:** Images must be uploaded to your Discord application in the Developer Portal first!

**Party (group info):**

```json
"party": {
  "id": "unique_party_id",
  "size": [2, 4]  // [current_size, max_size]
}
```

This shows "2 of 4" in a party!

**Buttons:**

```json
"buttons": [
  { "label": "Join Game", "url": "https://..." },
  { "label": "Watch Stream", "url": "https://..." }
]
```

Max 2 buttons. URLs only (no JavaScript, obviously).

### Visual Representation

Here's how it looks in Discord:

```
┌──────────────────────────────────────────┐
│  ┌──────┐                                │
│  │      │  Your Cool Game                │ ← large_image + large_text
│  │ IMG  │                                │
│  │      │  Playing as Tank               │ ← details
│  │    ┌─┐  In a match                    │ ← state
│  └────┤█├┘                               │
│       └─┘ ← small_image                  │
│                                          │
│  ⏱️  00:15:42 elapsed                    │ ← from timestamps.start
│                                          │
│  2 of 4 in party                         │ ← from party
│                                          │
│  ┌──────────────┐  ┌──────────────┐      │
│  │  Join Game   │  │ Watch Stream │      │ ← buttons
│  └──────────────┘  └──────────────┘      │
└──────────────────────────────────────────┘
```

### Discord's Response

After you send `SET_ACTIVITY`, Discord responds with:

```json
{
  "cmd": "SET_ACTIVITY",
  "data": {
    "state": "In a match",
    "details": "Playing as Tank"
    // ... (echoes back your activity)
  },
  "evt": null,
  "nonce": "unique-id-12345" // Same nonce you sent
}
```

If successful, you'll see the presence on your Discord profile!

### Simple Example

Want the bare minimum?

```json
{
  "cmd": "SET_ACTIVITY",
  "args": {
    "pid": 12345,
    "activity": {
      "details": "Just hanging out",
      "state": "Being awesome"
    }
  },
  "nonce": "1"
}
```

That's it! Just `details` and `state`. Discord will show it.

---

## Step 5: Keeping the Connection Alive

### Do You Need This?

**Short answer:** Not really for simple apps!

Discord doesn't strictly require heartbeats (PING/PONG) for basic presence updates. The connection stays alive as long as:

1. You don't close the socket
2. Discord doesn't close it
3. Neither of you crashes

**However**, if you want to be extra sure, or if you're building a long-running app, you can implement heartbeats.

### Heartbeat Flow

```
Your App                 Discord
   │                         │
   │  [PING]                 │
   ├────────────────────────>│
   │                         │
   │  [PONG]                 │
   │<────────────────────────┤
   │                         │
   │  (wait a bit...)        │
   │                         │
   │  [PING]                 │
   ├────────────────────────>│
   │                         │
```

### PING Packet

```
┌─────────────────────────────────────────┐
│  Header:                                │
│    Opcode:  3 (PING)                    │
│    Length:  0 (no payload usually)      │
│                                         │
│  Payload: {} or empty                   │
└─────────────────────────────────────────┘
```

Just send:

```
[03, 00, 00, 00, 00, 00, 00, 00]
 └─opcode 3──┘ └─length 0──┘
```

Discord will respond with opcode 4 (PONG).

### When to PING?

Every 30-60 seconds is fine. Think of it as saying "Still here!" to Discord.

**Pro tip:** For a simple app that just sets presence once and exits, skip this entirely.

---

## Step 6: Graceful Shutdown

When you're done, be polite and close the connection properly!

### CLOSE Packet

```
┌─────────────────────────────────────────┐
│  Header:                                │
│    Opcode:  2 (CLOSE)                   │
│    Length:  0                           │
│                                         │
│  Payload: (empty)                       │
└─────────────────────────────────────────┘
```

Send:

```
[02, 00, 00, 00, 00, 00, 00, 00]
 └─opcode 2──┘ └─length 0──┘
```

Then close the socket. Done!

### Or Just Close the Socket

Honestly? Just closing the socket works too. Discord will figure it out. But sending a CLOSE opcode is the "proper" way.

```
Your App                 Discord
   │                         │
   │  "Goodbye!"             │
   │  [CLOSE packet]         │
   ├────────────────────────>│
   │                         │
   │  [closes socket]        │
   ├─ × ─ × ─ × ─ × ─ × ────>│
   │                         │
```

---

## Complete Example Code

Alright! Let's put it all together. Here's a complete, working example in Rust (since you're working on a Rust project!).

### The Code (Synchronous)

```rust
// Filename: simple_discord_rpc.rs
//
// A simple, synchronous Discord RPC client that sets your presence.
// No async, no complexity, just pure synchronous goodness.

use std::io::{Read, Write};
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(windows)]
use std::fs::OpenOptions;

// Opcodes as defined by Discord
const OPCODE_HANDSHAKE: u32 = 0;
const OPCODE_FRAME: u32 = 1;
const OPCODE_CLOSE: u32 = 2;

// Platform-specific connection type
#[cfg(unix)]
type Connection = UnixStream;

#[cfg(windows)]
type Connection = std::fs::File;

/// Find Discord's IPC pipe/socket
fn find_discord_pipe() -> Result<PathBuf, String> {
    #[cfg(unix)]
    {
        // Try different locations
        let locations = vec![
            std::env::var("XDG_RUNTIME_DIR").ok(),
            std::env::var("TMPDIR").ok(),
            std::env::var("TMP").ok(),
            std::env::var("TEMP").ok(),
            Some("/tmp".to_string()),
        ];

        for location in locations.into_iter().flatten() {
            for i in 0..10 {
                let path = PathBuf::from(&location).join(format!("discord-ipc-{}", i));
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        Err("Discord not found! Is it running?".to_string())
    }

    #[cfg(windows)]
    {
        // On Windows, we try to open the named pipes
        for i in 0..10 {
            let pipe_name = format!(r"\\.\pipe\discord-ipc-{}", i);
            let path = PathBuf::from(&pipe_name);

            // Try to open it
            if let Ok(_) = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&pipe_name)
            {
                return Ok(path);
            }
        }

        Err("Discord not found! Is it running?".to_string())
    }
}

/// Connect to Discord
fn connect(path: &PathBuf) -> Result<Connection, String> {
    #[cfg(unix)]
    {
        UnixStream::connect(path)
            .map_err(|e| format!("Failed to connect: {}", e))
    }

    #[cfg(windows)]
    {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| format!("Failed to connect: {}", e))
    }
}

/// Write a packet to Discord
fn write_packet(conn: &mut Connection, opcode: u32, payload: &str) -> Result<(), String> {
    let payload_bytes = payload.as_bytes();
    let length = payload_bytes.len() as u32;

    // Create header (8 bytes: opcode + length)
    let mut packet = Vec::new();
    packet.extend_from_slice(&opcode.to_le_bytes());
    packet.extend_from_slice(&length.to_le_bytes());
    packet.extend_from_slice(payload_bytes);

    conn.write_all(&packet)
        .map_err(|e| format!("Failed to write packet: {}", e))
}

/// Read a packet from Discord
fn read_packet(conn: &mut Connection) -> Result<(u32, String), String> {
    // Read header (8 bytes)
    let mut header = [0u8; 8];
    conn.read_exact(&mut header)
        .map_err(|e| format!("Failed to read header: {}", e))?;

    // Parse header
    let opcode = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
    let length = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);

    // Read payload
    let mut payload = vec![0u8; length as usize];
    conn.read_exact(&mut payload)
        .map_err(|e| format!("Failed to read payload: {}", e))?;

    let payload_str = String::from_utf8(payload)
        .map_err(|e| format!("Invalid UTF-8: {}", e))?;

    Ok((opcode, payload_str))
}

/// Perform handshake with Discord
fn handshake(conn: &mut Connection, client_id: &str) -> Result<(), String> {
    // Send handshake
    let handshake_json = format!(r#"{{"v":1,"client_id":"{}"}}"#, client_id);
    write_packet(conn, OPCODE_HANDSHAKE, &handshake_json)?;

    // Read response
    let (opcode, response) = read_packet(conn)?;

    if opcode != OPCODE_FRAME {
        return Err(format!("Unexpected opcode: {}", opcode));
    }

    // Check if it's a READY event
    if response.contains(r#""evt":"READY""#) {
        println!("✅ Handshake successful!");
        Ok(())
    } else {
        Err(format!("Handshake failed: {}", response))
    }
}

/// Set activity (presence)
fn set_activity(conn: &mut Connection, details: &str, state: &str) -> Result<(), String> {
    let pid = std::process::id();

    let activity_json = format!(
        r#"{{
            "cmd": "SET_ACTIVITY",
            "args": {{
                "pid": {},
                "activity": {{
                    "details": "{}",
                    "state": "{}"
                }}
            }},
            "nonce": "1"
        }}"#,
        pid, details, state
    );

    write_packet(conn, OPCODE_FRAME, &activity_json)?;

    // Read response
    let (opcode, response) = read_packet(conn)?;

    if opcode == OPCODE_FRAME && response.contains(r#""cmd":"SET_ACTIVITY""#) {
        println!("✅ Activity set successfully!");
        Ok(())
    } else {
        Err(format!("Failed to set activity: {}", response))
    }
}

/// Close connection
fn close_connection(conn: &mut Connection) -> Result<(), String> {
    write_packet(conn, OPCODE_CLOSE, "")?;
    println!("👋 Connection closed");
    Ok(())
}

fn main() {
    println!("🚀 Simple Discord RPC Client\n");

    // Replace with your Discord Application ID!
    let client_id = "YOUR_CLIENT_ID_HERE";

    // Step 1: Find Discord
    println!("🔍 Looking for Discord...");
    let pipe_path = match find_discord_pipe() {
        Ok(path) => {
            println!("✅ Found Discord at: {}", path.display());
            path
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            return;
        }
    };

    // Step 2: Connect
    println!("\n🔌 Connecting to Discord...");
    let mut conn = match connect(&pipe_path) {
        Ok(c) => {
            println!("✅ Connected!");
            c
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            return;
        }
    };

    // Step 3: Handshake
    println!("\n🤝 Performing handshake...");
    if let Err(e) = handshake(&mut conn, client_id) {
        eprintln!("❌ {}", e);
        return;
    }

    // Step 4: Set presence
    println!("\n🎮 Setting activity...");
    if let Err(e) = set_activity(
        &mut conn,
        "Building cool stuff",
        "Learning Discord RPC"
    ) {
        eprintln!("❌ {}", e);
        return;
    }

    // Wait a bit so you can see it
    println!("\n⏳ Activity is now live! Check your Discord profile.");
    println!("   (Waiting 10 seconds before closing...)");
    std::thread::sleep(std::time::Duration::from_secs(10));

    // Step 5: Close
    println!("\n🛑 Closing connection...");
    let _ = close_connection(&mut conn);

    println!("\n✨ All done! That wasn't so hard, was it?");
}
```

### How to Run This

1. **Create a Discord Application:**

   - Go to https://discord.com/developers/applications
   - Click "New Application"
   - Copy your Application ID

2. **Replace `YOUR_CLIENT_ID_HERE`** with your actual ID

3. **Make sure Discord is running** on your computer

4. **Compile and run:**

   ```bash
   rustc simple_discord_rpc.rs
   ./simple_discord_rpc
   ```

5. **Check your Discord profile!** You should see:
   ```
   Building cool stuff
   Learning Discord RPC
   ```

---

## Troubleshooting

### "Discord not found!"

**Problem:** Can't find the IPC pipe/socket.

**Solutions:**

- [x] Make sure Discord is actually running
- [x] Check if you're using Discord PTB or Canary (they have different pipe names)
- [x] On Linux, check `$XDG_RUNTIME_DIR` and `/tmp`
- [x] On Windows, make sure you have permissions to access named pipes

### "Handshake failed: Invalid Client ID"

**Problem:** Discord doesn't recognize your app ID.

**Solutions:**

- [x] Double-check your Client ID from the Developer Portal
- [x] Make sure it's a string in the JSON: `"1234567890"`, not a number
- [x] Remove any spaces or extra characters

### "Connection refused" or "Broken pipe"

**Problem:** Discord closed the connection.

**Solutions:**

- [x] Discord might have crashed or restarted
- [x] Try a different pipe number (discord-ipc-1, discord-ipc-2, etc.)
- [x] Restart Discord

### Activity doesn't show up

**Problem:** Connected successfully, but presence isn't visible.

**Solutions:**

- [x] Check your Discord privacy settings: User Settings → Activity Privacy → "Display current activity as a status message"
- [x] Wait a few seconds; there's sometimes a slight delay
- [x] Make sure you're looking at the right Discord account (logged in on desktop)

### "Invalid UTF-8" error

**Problem:** Can't decode Discord's response.

**Solutions:**

- [x] This is rare; Discord always sends valid UTF-8
- [x] Check if you're reading the correct number of bytes
- [x] Make sure you're reading the full payload (use the length from header)

---

## Bonus: Understanding the JSON Packets in Detail

Let's break down each JSON packet type you'll encounter:

### 1. Handshake Request (You → Discord)

```json
{
  "v": 1,
  "client_id": "1234567890123456789"
}
```

**Fields:**

- `v`: **Version number** of the RPC protocol. Always `1`.
- `client_id`: **Your Discord Application ID**. Get it from the Developer Portal. Must be a string!

**Why it matters:** This tells Discord "I'm app X, let me in!" Discord checks if this app ID exists and if it's valid.

---

### 2. Handshake Response (Discord → You)

```json
{
  "cmd": "DISPATCH",
  "data": {
    "v": 1,
    "config": {
      "cdn_host": "cdn.discordapp.com",
      "api_endpoint": "//discord.com/api",
      "environment": "production"
    },
    "user": {
      "id": "123456789012345678",
      "username": "CoolDev",
      "discriminator": "0001",
      "avatar": "a_1234567890abcdef",
      "flags": 0,
      "premium_type": 0
    }
  },
  "evt": "READY",
  "nonce": null
}
```

**Fields:**

- `cmd`: Command type, `"DISPATCH"` means it's an event
- `evt`: Event name, `"READY"` means connection successful
- `data`: Contains config and **user info** (the Discord user running your app)
- `nonce`: Will be `null` for this response

**User object:**

- `id`: The user's Discord ID
- `username`: Their display name
- `discriminator`: Their 4-digit tag (or "0" for new usernames)
- `avatar`: Avatar hash (use this to construct avatar URL)

**Fun fact:** You can use this to personalize your app! "Welcome, CoolDev!"

---

### 3. SET_ACTIVITY Request (You → Discord)

```json
{
  "cmd": "SET_ACTIVITY",
  "args": {
    "pid": 12345,
    "activity": {
      "details": "Main text",
      "state": "Subtext",
      "timestamps": {
        "start": 1696435200
      },
      "assets": {
        "large_image": "game_logo",
        "large_text": "Hover text for large image",
        "small_image": "status_icon",
        "small_text": "Hover text for small image"
      },
      "party": {
        "id": "party_12345",
        "size": [2, 5]
      },
      "secrets": {
        "join": "joinSecret123",
        "spectate": "spectateSecret456"
      },
      "buttons": [
        {
          "label": "View Profile",
          "url": "https://example.com/profile"
        }
      ],
      "instance": true
    }
  },
  "nonce": "unique-nonce-123"
}
```

**Top-level:**

- `cmd`: Always `"SET_ACTIVITY"`
- `args`: Contains the actual activity data
- `nonce`: Optional unique ID to match response

**args:**

- `pid`: **Process ID** of your application
- `activity`: The presence object (see below)

**activity object (all fields optional!):**

| Field                | Type    | Description                | Example            |
| -------------------- | ------- | -------------------------- | ------------------ |
| `details`            | string  | First line of text         | "Playing Solo"     |
| `state`              | string  | Second line of text        | "In the Main Menu" |
| `timestamps.start`   | number  | Unix timestamp (seconds)   | 1696435200         |
| `timestamps.end`     | number  | Unix timestamp (seconds)   | 1696435800         |
| `assets.large_image` | string  | Key from Developer Portal  | "main_logo"        |
| `assets.large_text`  | string  | Hover tooltip              | "My Game v1.0"     |
| `assets.small_image` | string  | Key from Developer Portal  | "status_online"    |
| `assets.small_text`  | string  | Hover tooltip              | "Online"           |
| `party.id`           | string  | Unique party identifier    | "party_abc123"     |
| `party.size`         | array   | [current, max]             | [3, 6]             |
| `secrets.join`       | string  | Secret for join button     | (encrypted string) |
| `secrets.spectate`   | string  | Secret for spectate button | (encrypted string) |
| `buttons`            | array   | Up to 2 buttons            | See below          |
| `instance`           | boolean | Is this a game instance?   | true               |

**Buttons:**
Each button has:

- `label`: Text on the button (max 32 chars)
- `url`: HTTPS URL to open

**Secrets** (advanced): Used for "Ask to Join" and "Spectate" features. Requires additional OAuth setup.

---

### 4. SET_ACTIVITY Response (Discord → You)

```json
{
  "cmd": "SET_ACTIVITY",
  "data": {
    "details": "Main text",
    "state": "Subtext",
    ...
  },
  "evt": null,
  "nonce": "unique-nonce-123"
}
```

Discord echoes back your activity. If you see this, it worked! ✅

**Error response:**

```json
{
  "cmd": "SET_ACTIVITY",
  "data": {
    "code": 4000,
    "message": "Invalid payload"
  },
  "evt": "ERROR",
  "nonce": "unique-nonce-123"
}
```

Common error codes:

- `4000`: Invalid payload (check your JSON syntax!)
- `5000`: Unknown error (try again?)

---

### 5. CLEAR_ACTIVITY Request (Bonus!)

Want to remove your presence?

```json
{
  "cmd": "SET_ACTIVITY",
  "args": {
    "pid": 12345,
    "activity": null
  },
  "nonce": "clear-123"
}
```

Just set `activity` to `null`!

---

## ASCII Diagrams Reference

### Complete RPC Flow

```
┌──────────────┐
│  Your App    │
└──────┬───────┘
       │
       │ 1. Find Discord pipe
       │    (check /tmp/discord-ipc-*)
       ▼
┌─────────────────────────────────┐
│  Discord IPC Pipe/Socket        │
│  /tmp/discord-ipc-0             │
└─────────────┬───────────────────┘
              │
              │ 2. Connect
              │    (open socket)
              ▼
         ┌─────────┐
         │Connected│
         └────┬────┘
              │
              │ 3. Send HANDSHAKE (opcode 0)
              │    {"v":1,"client_id":"..."}
              ▼
         ┌─────────┐
         │ Discord │ ──► Validates client_id
         └────┬────┘
              │
              │ 4. Receive READY event
              │    {"evt":"READY","data":{...}}
              ▼
         ┌──────────┐
         │Authorized│
         └────┬─────┘
              │
              │ 5. Send SET_ACTIVITY (opcode 1)
              │    {"cmd":"SET_ACTIVITY","args":{...}}
              ▼
         ┌─────────┐
         │ Discord │ ──► Updates presence
         └────┬────┘     Shows on user's profile
              │
              │ 6. Receive acknowledgment
              │    {"cmd":"SET_ACTIVITY","data":{...}}
              ▼
         ┌─────────┐
         │ Active  │ ──► Presence is live!
         └────┬────┘
              │
              │ (Optional) Send PING (opcode 3)
              │ Receive PONG (opcode 4)
              │
              │ 7. Send CLOSE (opcode 2)
              │    or just close socket
              ▼
         ┌─────────┐
         │ Closed  │
         └─────────┘
```

### Packet Structure Diagram

```
Every packet sent over the wire:

Byte:   0       1       2       3       4       5       6       7       8...
      ┌───────┬───────┬───────┬───────┬───────┬───────┬───────┬───────┬───────────────>
      │       │       │       │       │       │       │       │       │
      │  Opcode (u32, little-endian)  │  Length (u32, little-endian)  │  Payload...
      │         bytes 0-3             │         bytes 4-7             │  bytes 8+
      └───────────────────────────────┴───────────────────────────────┴───────────────>

Example: HANDSHAKE with 44-byte JSON payload

Bytes:  00 00 00 00 | 2C 00 00 00 | 7B 22 76 22 3A 31 ...
        └─opcode 0─┘ └─length 44┘ └─JSON starts here──>
```

### Discord Presence Display

```
┌────────────────────────────────────────────────────────┐
│ John's Profile                                         │
├────────────────────────────────────────────────────────┤
│                                                        │
│  ┌──────────────┐                                      │
│  │              │  🎮 My Cool Game                     │  ← Large image + large_text
│  │  Large IMG   │                                      │
│  │              │     Playing as Warrior               │  ← details
│  │         ┌──┐ │     Level 42                         │  ← state
│  └─────────│██│─┘                                      │
│            └──┘       ← Small image                    │
│                         (small_text on hover)          │
│                                                        │
│  ⏱️ 01:23:45 elapsed                                   │  ← from timestamps.start
│                                                        │
│  3 of 5 in party                                       │  ← from party.size
│                                                        │
│  ┌──────────────────────┐  ┌──────────────────────┐    │
│  │   🎮 Join Game       │  │   📺 Watch Stream    │    │  ← buttons
│  └──────────────────────┘  └──────────────────────┘    │
│                                                        │
└────────────────────────────────────────────────────────┘
```

---

## Fun Facts & Tips

### Image Assets

To use images in your presence:

1. Go to Discord Developer Portal
2. Open your application
3. Go to "Rich Presence" → "Art Assets"
4. Upload images (max 5 MB each)
5. Give them a key name (e.g., "game_logo")
6. Use that key in `assets.large_image`

**Pro tip:** You can use external URLs for buttons, but images must be uploaded!

### Timestamps

```rust
// Get current Unix timestamp in Rust
let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs();
```

Discord calculates elapsed/remaining time automatically. If you set `start`, it shows "XX:YY elapsed". If you set `end`, it shows "XX:YY remaining".

### Presence vs. Activity

These terms are often used interchangeably, but technically:

- **Presence**: Your overall status (online, idle, dnd, offline)
- **Activity**: What you're doing (playing a game, listening to music, etc.)

Rich Presence = custom activity data. You're not changing your online status, just what you're doing.

### Rate Limits

Discord has rate limits:

- **Global**: 50 requests per minute across all endpoints
- **Per-client**: Don't spam SET_ACTIVITY; once every few seconds is fine

If you hit the limit, you'll get error code 4002. Just slow down!

### Debugging Tips

1. **Print everything:** Log all packets sent and received
2. **Check JSON syntax:** Use a JSON validator (most bugs are typos!)
3. **Test with minimal payload:** Start with just `details` and `state`
4. **Use Developer Mode:** Enable in Discord settings to see extra debug info

### Advanced: Understanding Nonces

A "nonce" (Number used ONCE) is a unique identifier for requests. It helps match responses to requests:

```rust
let nonce = uuid::Uuid::new_v4().to_string();

send_request(format!(r#"{{"cmd":"SET_ACTIVITY","nonce":"{}","args":...}}"#, nonce));

let response = read_response();
if response.nonce == nonce {
    // This is the response to our request!
}
```

Useful for async apps handling multiple requests simultaneously.

---

## What's Next?

Congratulations! 🎉 You now understand Discord RPC from the ground up. You know:

- [x] How IPC works
- [x] The packet structure (opcode + length + payload)
- [x] How to handshake with Discord
- [x] How to set rich presence
- [x] Every field in the activity object
- [x] How to debug common issues

### Where to Go From Here

1. **Async version**: Convert this to async for better performance
2. **Error handling**: Add proper error types instead of `String`
3. **Reconnection**: Handle Discord restarts automatically
4. **Subscribe to events**: Listen for button clicks, join requests, etc.
5. **Build a real app**: Music player, code editor extension, game integration

### Resources

- **Discord Developer Portal**: https://discord.com/developers/docs/topics/rpc
- **Example Apps**: Check out other RPC implementations on GitHub
- **Your Project**: `presenceforge` is a great reference! :)

---

## Closing Thoughts

Discord RPC might seem intimidating at first, but it's actually quite simple:

1. Open a socket
2. Send some JSON
3. ???
4. Profit!

The magic is in understanding the protocol. Once you know that every message is just an opcode + length + JSON payload, everything clicks into place.

Now go forth and build something awesome! And remember: if it doesn't work, it's probably a typo in your JSON.
