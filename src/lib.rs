//! # PresenceForge
//!
//! A cross-platform Rust library for Discord Rich Presence (IPC) integration.
//!
//! Supports both Unix-like systems (Linux, macOS) using Unix domain sockets
//! and Windows using named pipes.
//!
//! ## Features
//!
//! - Synchronous and asynchronous API
//! - Runtime-agnostic async design (supports tokio, async-std, and smol)
//! - Activity builder pattern
//! - Cross-platform support (Linux, macOS, Windows)
//!
//! ## Synchronous Example
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
//!
//! ## Async Example with Tokio
//!
//! ```rust,ignore
//! use presenceforge::{ActivityBuilder, Result};
//! use presenceforge::async_io::tokio::client::new_discord_ipc_client;
//!
//! # #[tokio::main]
//! # async fn main() -> Result {
//! let client_id = "your_client_id";
//! let mut client = new_discord_ipc_client(client_id).await?;
//!
//! // Perform handshake
//! client.connect().await?;
//!
//! // Create activity using the builder pattern
//! let activity = ActivityBuilder::new()
//!     .state("Playing a game")
//!     .details("In the menu")
//!     .start_timestamp_now()
//!     .large_image("game_logo")
//!     .large_text("My Awesome Game")
//!     .build();
//!
//! // Set the activity
//! client.set_activity(&activity).await?;
//!
//! // Keep activity for some time
//! tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
//!
//! // Clear the activity
//! client.clear_activity().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Async Example with async-std
//!
//! ```rust,ignore
//! use presenceforge::{ActivityBuilder, Result};
//! use presenceforge::async_io::async_std::client::new_discord_ipc_client;
//! use async_std::task;
//! use std::time::Duration;
//!
//! # #[async_std::main]
//! # async fn main() -> Result {
//! let client_id = "your_client_id";
//! let mut client = new_discord_ipc_client(client_id).await?;
//!
//! // Perform handshake
//! client.connect().await?;
//!
//! // Create activity using the builder pattern
//! let activity = ActivityBuilder::new()
//!     .state("Playing a game")
//!     .details("In the menu")
//!     .start_timestamp_now()
//!     .large_image("game_logo")
//!     .large_text("My Awesome Game")
//!     .build();
//!
//! // Set the activity
//! client.set_activity(&activity).await?;
//!
//! // Keep activity for some time
//! task::sleep(Duration::from_secs(10)).await;
//!
//! // Clear the activity
//! client.clear_activity().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Async Example with smol
//!
//! ```rust,ignore
//! use presenceforge::{ActivityBuilder, Result};
//! use presenceforge::async_io::smol::client::new_discord_ipc_client;
//! use std::time::Duration;
//!
//! fn main() -> Result {
//!     smol::block_on(async {
//!         let client_id = "your_client_id";
//!         let mut client = new_discord_ipc_client(client_id).await?;
//!
//!         // Perform handshake
//!         client.connect().await?;
//!
//!         // Create activity using the builder pattern
//!         let activity = ActivityBuilder::new()
//!             .state("Playing a game")
//!             .details("In the menu")
//!             .start_timestamp_now()
//!             .large_image("game_logo")
//!             .large_text("My Awesome Game")
//!             .build();
//!
//!         // Set the activity
//!         client.set_activity(&activity).await?;
//!
//!         // Keep activity for some time
//!         smol::Timer::after(Duration::from_secs(10)).await;
//!
//!         // Clear the activity
//!         client.clear_activity().await?;
//!         Ok(())
//!     })
//! }
//! ```

pub mod activity;
pub mod async_io;
pub mod client;
pub mod error;
pub mod ipc;
pub mod macros;
pub mod utils;

// Re-export the main public API
pub use activity::{
    Activity, ActivityAssets, ActivityBuilder, ActivityButton, ActivityParty, ActivitySecrets,
    ActivityTimestamps,
};
pub use error::{DiscordIpcError, ProtocolContext, Result};
pub use ipc::protocol::IpcConfig;
pub use ipc::{Command, DiscoveredPipe, IpcConnection, Opcode, PipeConfig};
pub use macros::is_debug_enabled;

// Re-export the synchronous API for backwards compatibility
pub use sync::client::DiscordIpcClient;

// The sync module is also accessible for more explicit imports
pub mod sync;
