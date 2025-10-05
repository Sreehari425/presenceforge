# ActivityBuilder Reference (WIP)

This document explains every field available in the ActivityBuilder and how it appears in Discord.

> ⚠️ **NOTE:** This feature is experimental/untested. Use at your own risk.

## Visual Layout

```
┌─────────────────────────────────────────────────┐
│  ┌──────────┐                                   │
│  │  Large   │    DETAILS (larger text)          │
│  │  Image   │    State (smaller text)           │
│  │   ┌───┐  │    Party: 2 of 4                  │
│  │   │ S │  │    ⏱ 00:15 elapsed                │
│  └───┴───┘──┘                                   │
│                                                 │
│  ┌─────────────┐    ┌──────────────┐            │
│  │  Button 1   │    │   Button 2   │            │
│  └─────────────┘    └──────────────┘            │
└─────────────────────────────────────────────────┘

S = Small Image (overlays large image in bottom-right corner)
```

## All Available Fields

### 1. **State** (`state()`)

```rust
.state("Playing a game")
```

- **Appears as:** First line of text (smaller, below details)
- **Examples:**
  - Games: "In a Match", "In Lobby", "Exploring the Overworld"
  - IDEs: "Editing main.rs", "Debugging"
  - Music: "Song Name - Artist"
- **Character limit:** 128 characters

---

### 2. **Details** (`details()`)

```rust
.details("Competitive Mode")
```

- **Appears as:** Second line of text (larger, above state)
- **Examples:**
  - Games: "Competitive - Rank 50", "Mission 3: Infiltration"
  - IDEs: "Workspace: my-project", "In presenceforge"
  - Music: "Album: Greatest Hits"
- **Character limit:** 128 characters

---

### 3. **Large Image** (`large_image()` + `large_text()`)

```rust
.large_image("game_logo")        // Asset key
.large_text("Hover text here!")  // Tooltip
```

- **Appears as:** Main large image on the left side
- **large_image:** Asset key uploaded to Discord Developer Portal
- **large_text:** Tooltip text shown when user hovers over the image
- **Upload assets at:** Discord Developer Portal > Your App > Rich Presence > Art Assets
- **Recommended size:** 1024x1024 pixels
- **Formats:** PNG, JPG

---

### 4. **Small Image** (`small_image()` + `small_text()`)

```rust
.small_image("rust_logo")           // Asset key
.small_text("Built with Rust 🦀")   // Tooltip
```

- **Appears as:** Small circular image overlaying the large image (bottom-right corner)
- **small_image:** Asset key uploaded to Discord Developer Portal
- **small_text:** Tooltip text shown when user hovers over the small image
- **Common uses:**
  - Player status icons (online, away, busy)
  - Language/framework logos
  - Game character avatars
  - Current tool/mode indicators
- **Recommended size:** 256x256 pixels

---

### 5. **Timestamps** (`start_timestamp()`, `end_timestamp()`)

#### Start Timestamp (Elapsed Time)

```rust
.start_timestamp_now()  // Start counting from now
// or
.start_timestamp(1234567890)  // Unix timestamp
```

- **Appears as:** "XX:XX elapsed"
- **Shows:** How long the activity has been running
- **Use for:** Game sessions, editing time, listening time

#### End Timestamp (Remaining Time)

```rust
.end_timestamp(unix_timestamp)
```

- **Appears as:** "XX:XX left"
- **Shows:** Countdown until the end time
- **Use for:** Match end times, timer events, scheduled activities
- **Note:** If both start and end are set, Discord shows remaining time

**Getting Unix timestamp:**

```rust
use std::time::{SystemTime, UNIX_EPOCH};

// Current time
let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

// 5 minutes from now
let end_time = now + 300;
```

---

### 6. **Party** (`party()`)

#### Note: Partialy tested feature

```rust
.party("unique-party-id", 2, 4)  // party_id, current_size, max_size
```

- **Appears as:** "2 of 4" below the state text
- **Parameters:**
  - `party_id`: Unique identifier for the party (string)
  - `current_size`: Current number of players (u32)
  - `max_size`: Maximum number of players (u32)
- **Use for:** Multiplayer games, voice channels, collaborative work
- **Examples:**
  - "2 of 4" (2 players in a 4-player game)
  - "5 of 10" (5 people in a 10-person voice channel)

---

### 7. **Buttons** (`button()`)

```rust
.button(" Play Now", "https://example.com/game")
.button(" Docs", "https://docs.rs/presenceforge")
```

- **Appears as:** Clickable buttons at the bottom of the rich presence
- **Maximum:** 2 buttons allowed
- **Parameters:**
  - `label`: Button text (string) - max 32 characters
  - `url`: URL to open when clicked (string) - must start with `http://` or `https://`
- **Common uses:**
  - "View Profile", "Watch Stream", "Join Game"
  - "Documentation", "Website", "Support"
  - "Download", "Learn More", "Join Server"

---

### 8. **Secrets** (For "Ask to Join" and Spectate features)

> **⚠️ Feature Flag Required:** These methods require the `secrets` feature flag to be enabled.
> Add to your `Cargo.toml`: `presenceforge = { git = "...", features = ["secrets"] }`

#### Note: untested feature

#### Join Secret

```rust
.join_secret("join_secret_abc123")
```

- **Enables:** "Ask to Join" button
- **Your app receives:** This secret when another user clicks "Ask to Join"
- **Use for:** Allowing players to join your game through Discord

#### Spectate Secret

##### Note: untested feature

```rust
.spectate_secret("spectate_secret_xyz789")
```

- **Enables:** "Spectate" button
- **Your app receives:** This secret when another user clicks "Spectate"
- **Use for:** Allowing others to watch your gameplay

#### Match Secret

```rust
.match_secret("match_secret_unique_id")
```

- **Purpose:** Unique identifier for the current match/game session
- **Use for:** Internal game session tracking
- **Note:** Not directly visible to users, used by Discord's matchmaking system

**Important:** Secrets are for advanced integrations and require your app to handle join/spectate requests.

---

### 9. **Instance** (`instance()`)

##### Note: untested feature

```rust
.instance(true)   // or false
```

- **Purpose:** Indicates if the activity is an instanced context
- **Values:**
  - `true`: Unique game instance (specific match, specific session)
  - `false`: General activity (browsing, in menu, idle)
- **Use for:**
  - Set to `true` when in an active game/match
  - Set to `false` when in menus or general activities
- **Affects:** How Discord groups and displays activities

---

## Example (all fields)

Here's an activity using all fields:

```rust
use presenceforge::{ActivityBuilder, DiscordIpcClient};

let activity = ActivityBuilder::new()
    // Text
    .state("Playing a custom game")
    .details("Competitive Mode - Rank 50")

    // Images
    .large_image("game_logo")
    .large_text("Epic Game 2024")
    .small_image("player_avatar")
    .small_text("Level 42 Warrior")

    // Time
    .start_timestamp_now()

    // Party
    .party("party-12345", 3, 4)

    // Buttons
    .button("🎮 Join Game", "https://game.example.com/join")
    .button("📊 View Stats", "https://game.example.com/stats")

    // Secrets (advanced)
    .join_secret("join_xyz")
    .spectate_secret("spectate_abc")
    .match_secret("match_unique_id")

    // Instance
    .instance(true)

    .build();

let mut client = DiscordIpcClient::new("your_client_id")?;
client.connect()?;
client.set_activity(&activity)?;
```

## See Also

- **Example:** `examples/builder_all.rs` - Working example with all fields
- **Example:** `examples/basic.rs` - Simple example to get started
- **Example:** `examples/game_demo.rs` - Game-focused example with dynamic updates
