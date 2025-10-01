//! # PresenceForge
//! 
//! A cross-platform Rust library for Discord Rich Presence (IPC) integration.
//! 
//! Supports both Unix-like systems (Linux, macOS) using Unix domain sockets
//! and Windows using named pipes.
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

pub mod error;
pub mod ipc;
pub mod activity;
pub mod client;

// Re-export the main public API
pub use client::DiscordIpcClient;
pub use activity::{Activity, ActivityBuilder, ActivityAssets, ActivityTimestamps, ActivityParty, ActivitySecrets, ActivityButton};
pub use error::{DiscordIpcError, Result};
pub use ipc::{Opcode, Command};
