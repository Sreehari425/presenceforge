//! # PresenceForge
//!
//! A Rust library for Discord Rich Presence (IPC) integration.
//!
//! ## Example
//!
//! ```rust
//! use presenceforge::{DiscordIpcClient, ActivityBuilder};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut client = DiscordIpcClient::new("your_client_id")?;
//! client.connect()?;
//!
//! let activity = ActivityBuilder::new()
//!     .state("Playing a game")
//!     .details("In the menu")
//!     .start_timestamp_now()
//!     .large_image("game_logo")
//!     .large_text("My Awesome Game")
//!     .build();
//!
//! client.set_activity(&activity)?;
//!
//! // Keep the activity for some time...
//! std::thread::sleep(std::time::Duration::from_secs(10));
//!
//! client.clear_activity()?;
//! # Ok(())
//! # }
//! ```

pub mod activity;
pub mod client;
pub mod error;
pub mod ipc;

// Re-export the main public API
pub use activity::{
    Activity, ActivityAssets, ActivityBuilder, ActivityButton, ActivityParty, ActivitySecrets,
    ActivityTimestamps,
};
pub use client::DiscordIpcClient;
pub use error::{DiscordIpcError, Result};
pub use ipc::{Command, Opcode};
