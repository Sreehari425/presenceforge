//! Async implementation of Discord Rich Presence client
//!
//! This module provides a runtime-agnostic async implementation of the Discord IPC client
//! that works with any async runtime through the use of abstract traits.
//!
//! ## Supported Runtimes
//!
//! - **Tokio**: Enable with the `tokio-runtime` feature
//! - **async-std**: Enable with the `async-std-runtime` feature
//!
//! ## Usage Examples
//!
//! ### With Tokio
//!
//! ```rust,no_run
//! use presenceforge::{ActivityBuilder, Result};
//! use presenceforge::async_io::tokio::client::new_discord_ipc_client;
//!
//! # #[tokio::main]
//! # async fn main() -> Result {
//! let client_id = "your_client_id";
//! let mut client = new_discord_ipc_client(client_id).await?;
//! client.connect().await?;
//!
//! let activity = ActivityBuilder::new()
//!     .state("Playing a game")
//!     .details("In the menu")
//!     .build();
//!
//! client.set_activity(&activity).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### With async-std
//!
//! ```rust,no_run
//! use presenceforge::{ActivityBuilder, Result};
//! use presenceforge::async_io::async_std::client::new_discord_ipc_client;
//!
//! # #[async_std::main]
//! # async fn main() -> Result {
//! let client_id = "your_client_id";
//! let mut client = new_discord_ipc_client(client_id).await?;
//! client.connect().await?;
//!
//! let activity = ActivityBuilder::new()
//!     .state("Playing a game")
//!     .details("In the menu")
//!     .build();
//!
//! client.set_activity(&activity).await?;
//! # Ok(())
//! # }
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
