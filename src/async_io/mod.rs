//! Async implementation of Discord Rich Presence client
//!
//! This module provides a runtime-agnostic async implementation of the Discord IPC client
//! that works with any async runtime through the use of abstract traits.
//!
//! ## Unified API (Recommended)
//!
//! The library provides a single `AsyncDiscordIpcClient` type that automatically adapts
//! to your chosen async runtime. Simply enable the appropriate feature flag:
//!
//! - `tokio-runtime` - for Tokio
//! - `async-std-runtime` - for async-std
//! - `smol-runtime` - for smol
//!
//! ### Example
//!
//! ```rust,no_run
//! # #[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime", feature = "smol-runtime"))]
//! # {
//! // Same code works with any runtime!
//! use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};
//!
//! async fn setup_presence() -> Result {
//!     let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
//!     client.connect().await?;
//!
//!     let activity = ActivityBuilder::new()
//!         .state("Playing a game")
//!         .details("In the menu")
//!         .build();
//!
//!     client.set_activity(&activity).await?;
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ## Runtime-Specific APIs (Advanced)
//!
//! For advanced use cases where you need direct access to runtime-specific features,
//! you can still import the concrete types:
//!
//! ### With Tokio
//!
//! ```rust,no_run
//! # #[cfg(feature = "tokio-runtime")]
//! use presenceforge::async_io::tokio::TokioDiscordIpcClient;
//! ```
//!
//! ### With async-std
//!
//! ```rust,no_run
//! # #[cfg(feature = "async-std-runtime")]
//! use presenceforge::async_io::async_std::AsyncStdDiscordIpcClient;
//! ```
//!
//! ### With smol
//!
//! ```rust,no_run
//! # #[cfg(feature = "smol-runtime")]
//! use presenceforge::async_io::smol::SmolDiscordIpcClient;
//! ```

mod client;
mod traits;

pub use client::AsyncDiscordIpcClient;
pub use traits::{AsyncRead, AsyncWrite};

// Runtime-specific re-exports
#[cfg(feature = "tokio-runtime")]
pub mod tokio;

#[cfg(feature = "async-std-runtime")]
pub mod async_std;

#[cfg(feature = "smol-runtime")]
pub mod smol;
