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
//! - **Unified async API** - Write once, run on any async runtime
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
//!     .start_timestamp_now()?
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
//! ## Unified Async API
//!
//! The library provides a single `AsyncDiscordIpcClient` type that automatically
//! adapts to your chosen async runtime through feature flags. No need for
//! runtime-specific imports!
//!
//! ### With Tokio
//!
//! Enable the `tokio-runtime` feature in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! presenceforge = { version = "0.0.0", features = ["tokio-runtime"] }
//! tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
//! ```
//!
//! ```rust,no_run
//! # #[cfg(feature = "tokio-runtime")]
//! # {
//! use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};
//!
//! async fn main() -> Result {
//!     let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
//!     client.connect().await?;
//!
//!     let activity = ActivityBuilder::new()
//!         .state("Playing a game")
//!         .details("In the menu")
//!         .start_timestamp_now()?
//!         .large_image("game_logo")
//!         .large_text("My Awesome Game")
//!         .build();
//!
//!     client.set_activity(&activity).await?;
//! #   tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
//! #   client.clear_activity().await?;
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ### With async-std
//!
//! Enable the `async-std-runtime` feature:
//!
//! ```toml
//! [dependencies]
//! presenceforge = { version = "0.0.0", features = ["async-std-runtime"] }
//! async-std = { version = "1", features = ["attributes"] }
//! ```
//!
//! ```rust,no_run
//! # #[cfg(feature = "async-std-runtime")]
//! # {
//! use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};
//!
//! async fn main() -> Result {
//!     let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
//!     client.connect().await?;
//!
//!     let activity = ActivityBuilder::new()
//!         .state("Playing a game")
//!         .details("In the menu")
//!         .start_timestamp_now()?
//!         .large_image("game_logo")
//!         .large_text("My Awesome Game")
//!         .build();
//!
//!     client.set_activity(&activity).await?;
//! #   async_std::task::sleep(std::time::Duration::from_secs(10)).await;
//! #   client.clear_activity().await?;
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ### With smol
//!
//! Enable the `smol-runtime` feature:
//!
//! ```toml
//! [dependencies]
//! presenceforge = { version = "0.0.0", features = ["smol-runtime"] }
//! smol = "2"
//! ```
//!
//! ```rust,no_run
//! # #[cfg(feature = "smol-runtime")]
//! # {
//! use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};
//!
//! fn main() -> Result {
//!     smol::block_on(async {
//!         let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
//!         client.connect().await?;
//!
//!         let activity = ActivityBuilder::new()
//!             .state("Playing a game")
//!             .details("In the menu")
//!             .start_timestamp_now()?
//!             .large_image("game_logo")
//!             .large_text("My Awesome Game")
//!             .build();
//!
//!         client.set_activity(&activity).await?;
//!         smol::Timer::after(std::time::Duration::from_secs(10)).await;
//!         client.clear_activity().await?;
//!         Ok(())
//!     })
//! }
//! # }
//! ```
//!
//! ## Runtime-Specific APIs (Advanced)
//!
//! For advanced use cases, you can still import runtime-specific clients directly:
//!
//! ```rust,no_run
//! // Tokio-specific client
//! # #[cfg(feature = "tokio-runtime")]
//! use presenceforge::async_io::tokio::TokioDiscordIpcClient;
//!
//! // async-std-specific client
//! # #[cfg(feature = "async-std-runtime")]
//! use presenceforge::async_io::async_std::AsyncStdDiscordIpcClient;
//!
//! // smol-specific client
//! # #[cfg(feature = "smol-runtime")]
//! use presenceforge::async_io::smol::SmolDiscordIpcClient;
//! ```

pub mod activity;
pub mod async_io;
pub mod client;
pub mod error;
pub mod ipc;
pub mod macros;
pub mod retry;
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

// Unified async API - automatically selects the correct runtime based on feature flags
#[cfg(feature = "tokio-runtime")]
pub use async_io::tokio::TokioDiscordIpcClient as AsyncDiscordIpcClient;

#[cfg(all(feature = "async-std-runtime", not(feature = "tokio-runtime")))]
pub use async_io::async_std::AsyncStdDiscordIpcClient as AsyncDiscordIpcClient;

#[cfg(all(
    feature = "smol-runtime",
    not(feature = "tokio-runtime"),
    not(feature = "async-std-runtime")
))]
pub use async_io::smol::SmolDiscordIpcClient as AsyncDiscordIpcClient;
