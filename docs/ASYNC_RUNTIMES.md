# Async Runtimes Guide

> ⚠️ **NOTE:** This feature is experimental/untested. Use at your own risk.

## Table of Contents

- [Overview](#overview)
- [Unified API (Recommended)](#unified-api-recommended)
- [Tokio](#tokio)
- [async-std](#async-std)
- [smol](#smol)
- [Runtime Comparison](#runtime-comparison)
- [Best Practices](#best-practices)
- [Migration from Sync](#migration-from-sync)
- [Advanced: Runtime-Specific APIs](#advanced-runtime-specific-apis)

---

## Overview

PresenceForge offers async support with a runtime-agnostic design. You can use it with:

- **Tokio** - Most popular async runtime
- **async-std** - Standard library style async runtime
- **smol** - Lightweight async runtime

**All three runtimes use the same unified API**, making it trivial to switch between them without changing your code!

### When to Use Async

**Use async when:**

- Your application is already using an async runtime
- You need non-blocking I/O operations
- You're building a server or service with many concurrent connections
- You want to update presence without blocking other tasks

**Use sync when:**

- You have a simple application with no async code
- You only update presence occasionally
- You want minimal dependencies
- You're just getting started (sync is simpler)

---

## Unified API (Recommended)

The unified API allows you to write runtime-agnostic code that works with any async runtime. Simply import `AsyncDiscordIpcClient` and enable the appropriate feature flag - the library handles the rest!

### Why Use the Unified API?

**No Code Changes** - Switch runtimes by just changing your feature flag  
 **DRY Principle** - Write once, run on any runtime  
 **Simple Imports** - Single import works everywhere  
 **Future-Proof** - Your code won't break if you switch runtimes later

### Example: Same Code, Any Runtime!

Add to your `Cargo.toml` with **one** of these feature flags:

```toml
[dependencies]
# For Tokio
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["tokio-runtime"] }

# For async-std
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["async-std-runtime"] }

# For smol
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["smol-runtime"] }
```

**This exact code works with all three runtimes:**

```rust
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};

async fn setup_presence() -> Result {
    // Create client - works with Tokio, async-std, or smol!
    let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;

    // Connect to Discord
    client.connect().await?;

    println!("Connected to Discord!");

    // Create activity
    let activity = ActivityBuilder::new()
        .state("Playing async")
        .details("Runtime-agnostic!")
        .start_timestamp_now().expect("timestamp")
        .build();

    // Set activity
    client.set_activity(&activity).await?;

    Ok(())
}
```

Then just use your runtime's entry point:

**With Tokio:**

```rust
#[tokio::main]
async fn main() -> Result {
    setup_presence().await
}
```

**With async-std:**

```rust
#[async_std::main]
async fn main() -> Result {
    setup_presence().await
}
```

**With smol:**

```rust
fn main() -> Result {
    smol::block_on(setup_presence())
}
```

---

## Tokio

Tokio is the most popular async runtime in the Rust ecosystem.

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["tokio-runtime"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Basic Usage

```rust
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};

#[tokio::main]
async fn main() -> Result {
    // Create client
    let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;

    // Connect to Discord
    client.connect().await?;

    println!("Connected to Discord!");

    // Create activity
    let activity = ActivityBuilder::new()
        .state("Playing async")
        .details("Using Tokio")
        .start_timestamp_now().expect("timestamp")
        .build();

    // Set activity
    client.set_activity(&activity).await?;

    println!("Activity set! Press Ctrl+C to exit.");

    // Keep running
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

    // Clean up
    client.clear_activity().await?;

    Ok(())
}
```

### With Tokio Tasks

Run presence updates in a background task:

```rust
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};
use tokio::time::{sleep, Duration, interval};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result {
    let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
    client.connect().await?;

    // Wrap client in Arc<Mutex> for sharing across tasks
    let client = Arc::new(Mutex::new(client));

    // Spawn presence update task
    let presence_client = Arc::clone(&client);
    let presence_task = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(15));
        let mut counter = 0;

        loop {
            interval.tick().await;
            counter += 1;

                let activity = ActivityBuilder::new()
                .state(format!("Update #{}", counter))
                .details("Tokio Background Task")
                .start_timestamp_now().expect("timestamp")
                .build();

            let mut client = presence_client.lock().await;
            if let Err(e) = client.set_activity(&activity).await {
                eprintln!("Failed to update presence: {}", e);
            }
        }
    });

    // Do other work...
    println!("Doing other async work...");
    sleep(Duration::from_secs(60)).await;

    // Cleanup
    presence_task.abort();
    let mut client = client.lock().await;
    client.clear_activity().await?;

    Ok(())
}
```

### With Tokio Channels

Communicate with presence task via channels:

```rust
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

enum PresenceCommand {
    SetActivity(String, String),
    Clear,
    Shutdown,
}

#[tokio::main]
async fn main() -> Result {
    let (tx, mut rx) = mpsc::channel::<PresenceCommand>(32);

    // Spawn presence manager task
    let presence_task = tokio::spawn(async move {
        let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
        client.connect().await?;

        while let Some(cmd) = rx.recv().await {
            match cmd {
                PresenceCommand::SetActivity(state, details) => {
                    let activity = ActivityBuilder::new()
                        .state(state)
                        .details(details)
                        .start_timestamp_now()
                        .build();

                    if let Err(e) = client.set_activity(&activity).await {
                        eprintln!("Failed to set activity: {}", e);
                    }
                }
                PresenceCommand::Clear => {
                    if let Err(e) = client.clear_activity().await {
                        eprintln!("Failed to clear activity: {}", e);
                    }
                }
                PresenceCommand::Shutdown => {
                    client.clear_activity().await.ok();
                    break;
                }
            }
        }

        Result::Ok(())
    });

    // Send commands
    tx.send(PresenceCommand::SetActivity(
        "Coding".to_string(),
        "main.rs".to_string()
    )).await.unwrap();

    sleep(Duration::from_secs(5)).await;

    tx.send(PresenceCommand::SetActivity(
        "Building".to_string(),
        "Release mode".to_string()
    )).await.unwrap();

    sleep(Duration::from_secs(5)).await;

    tx.send(PresenceCommand::Shutdown).await.unwrap();

    presence_task.await.unwrap()?;

    Ok(())
}
```

---

## async-std

async-std provides an async API similar to the standard library.

### Installation

```toml
[dependencies]
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["async-std-runtime"] }
async-std = { version = "1", features = ["attributes"] }
```

### Basic Usage

```rust
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};
use async_std::task;
use std::time::Duration;

#[async_std::main]
async fn main() -> Result {
    // Create and connect client
    let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
    client.connect().await?;

    println!("Connected to Discord!");

    // Create and set activity
    let activity = ActivityBuilder::new()
        .state("Playing async")
        .details("Using async-std")
        .start_timestamp_now()
        .build();

    client.set_activity(&activity).await?;

    println!("Activity set!");

    // Keep running
    task::sleep(Duration::from_secs(60)).await;

    // Cleanup
    client.clear_activity().await?;

    Ok(())
}
```

### With async-std Tasks

```rust
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};
use async_std::{task, channel};
use std::time::Duration;

#[async_std::main]
async fn main() -> Result {
    let (sender, receiver) = channel::bounded(10);

    // Spawn presence manager task
    let handle = task::spawn(async move {
        let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
        client.connect().await?;

        while let Ok(state) = receiver.recv().await {
                        let activity = ActivityBuilder::new()
                        .state(state)
                        .details("async-std Task")
                        .build();

            if let Err(e) = client.set_activity(&activity).await {
                eprintln!("Error setting activity: {}", e);
            }
        }

        client.clear_activity().await?;
        Result::Ok(())
    });

    // Send updates
    sender.send("Initializing".to_string()).await.ok();
    task::sleep(Duration::from_secs(2)).await;

    sender.send("Running".to_string()).await.ok();
    task::sleep(Duration::from_secs(2)).await;

    sender.send("Finishing".to_string()).await.ok();
    task::sleep(Duration::from_secs(2)).await;

    // Close channel and wait for cleanup
    drop(sender);
    handle.await?;

    Ok(())
}
```

---

## smol

smol is a small and fast async runtime.

### Installation

```toml
[dependencies]
presenceforge = { git = "https://github.com/Sreehari425/presenceforge", features = ["smol-runtime"] }
smol = "2"
```

### Basic Usage

```rust
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};
use smol::Timer;
use std::time::Duration;

fn main() -> Result {
    smol::block_on(async {
        // Create and connect client
        let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
        client.connect().await?;

        println!("Connected to Discord!");

        // Create and set activity
        let activity = ActivityBuilder::new()
            .state("Playing async")
            .details("Using smol")
            .start_timestamp_now().expect("timestamp")
            .build();

        client.set_activity(&activity).await?;

        println!("Activity set!");

        // Keep running
        Timer::after(Duration::from_secs(60)).await;

        // Cleanup
        client.clear_activity().await?;

        Ok(())
    })
}
```

### With smol Tasks

```rust
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};
use smol::{Timer, channel};
use std::time::Duration;

fn main() -> Result {
    smol::block_on(async {
        let (sender, receiver) = channel::bounded(10);

        // Spawn presence task
        let task = smol::spawn(async move {
            let mut client = AsyncDiscordIpcClient::new("your_client_id").await?;
            client.connect().await?;

            while let Ok(message) = receiver.recv().await {
                let activity = ActivityBuilder::new()
                    .state(message)
                    .details("smol Runtime")
                    .build();

                if let Err(e) = client.set_activity(&activity).await {
                    eprintln!("Failed to set activity: {}", e);
                }
            }

            client.clear_activity().await?;
            Result::Ok(())
        });

        // Send some updates
        for i in 1..=5 {
            sender.send(format!("Update {}", i)).await.ok();
            Timer::after(Duration::from_secs(2)).await;
        }

        // Cleanup
        drop(sender);
        task.await?;

        Ok(())
    })
}
```

---

## Best Practices

### 1. Choose One Runtime

Don't mix async runtimes in the same project:

**Don't:**

```toml
[dependencies]
presenceforge = { features = ["tokio-runtime", "smol-runtime"] }
```

**Do:**

```toml
[dependencies]
presenceforge = { features = ["tokio-runtime"] }
tokio = { version = "1", features = ["full"] }
```

---

### 2. Handle Errors Properly

Always handle async errors:

```rust
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};

#[tokio::main]
async fn main() -> Result {
    let mut client = AsyncDiscordIpcClient::new("client_id").await
        .map_err(|e| {
            eprintln!("Failed to create client: {}", e);
            e
        })?;

    client.connect().await
        .map_err(|e| {
            eprintln!("Failed to connect: {}", e);
            e
        })?;

    // ... rest of code

    Ok(())
}
```

---

### 3. Use Timeouts

Prevent hanging on slow operations:

```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(5),
    client.connect()
).await;

match result {
    Ok(Ok(_)) => println!("Connected!"),
    Ok(Err(e)) => eprintln!("Connection failed: {}", e),
    Err(_) => eprintln!("Connection timed out"),
}
```

---

### 4. Share Clients Safely

Use Arc<Mutex<>> for sharing:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

let client = Arc::new(Mutex::new(client));

// Clone for another task
let client_clone = Arc::clone(&client);
tokio::spawn(async move {
    let mut c = client_clone.lock().await;
    c.set_activity(&activity).await.ok();
});
```

---

### 5. Clean Up on Shutdown

Use signal handlers:

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result {
    let mut client = AsyncDiscordIpcClient::new("client_id").await?;
    client.connect().await?;

    // Set up Ctrl+C handler
    let client_for_cleanup = Arc::new(Mutex::new(client));
    let client_clone = Arc::clone(&client_for_cleanup);

    tokio::spawn(async move {
        signal::ctrl_c().await.ok();
        println!("\nShutting down...");
        let mut c = client_clone.lock().await;
        c.clear_activity().await.ok();
        std::process::exit(0);
    });

    // Main loop
    loop {
        // Update presence...
        tokio::time::sleep(Duration::from_secs(15)).await;
    }
}
```

---

## Migration from Sync

### Sync Code

```rust
use presenceforge::ActivityBuilder;
use presenceforge::sync::DiscordIpcClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DiscordIpcClient::new("client_id")?;
    client.connect()?;

    let activity = ActivityBuilder::new()
        .state("Hello")
        .build();

    client.set_activity(&activity)?;

    std::thread::sleep(std::time::Duration::from_secs(10));

    client.clear_activity()?;
    Ok(())
}
```

### Async Code (Tokio)

```rust
use presenceforge::{AsyncDiscordIpcClient, ActivityBuilder, Result};

#[tokio::main]
async fn main() -> Result {
    let mut client = AsyncDiscordIpcClient::new("client_id").await?;
    client.connect().await?;

    let activity = ActivityBuilder::new()
        .state("Hello")
        .build();

    client.set_activity(&activity).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    client.clear_activity().await?;
    Ok(())
}
```

### Key Changes

1. Add `.await` to all async operations
2. Change `DiscordIpcClient::new()` to `AsyncDiscordIpcClient::new().await`
3. Use `#[tokio::main]` (or equivalent) instead of regular `main`
4. Change `std::thread::sleep` to runtime sleep
5. Change error type from `Box<dyn std::error::Error>` to `Result`

---

## Advanced: Runtime-Specific APIs

For advanced use cases where you need direct access to runtime-specific client types (e.g., for type annotations or specialized functionality), you can still import them explicitly:

### Tokio-Specific Client

```rust
use presenceforge::async_io::tokio::TokioDiscordIpcClient;

async fn my_function(client: &mut TokioDiscordIpcClient) {
    // Function that explicitly requires Tokio client
}
```

### async-std-Specific Client

```rust
use presenceforge::async_io::async_std::AsyncStdDiscordIpcClient;

async fn my_function(client: &mut AsyncStdDiscordIpcClient) {
    // Function that explicitly requires async-std client
}
```

### smol-Specific Client

```rust
use presenceforge::async_io::smol::SmolDiscordIpcClient;

async fn my_function(client: &mut SmolDiscordIpcClient) {
    // Function that explicitly requires smol client
}
```

### When to Use Runtime-Specific Types

Use the unified `AsyncDiscordIpcClient` **unless** you need to:

- Write generic functions with explicit type constraints
- Access runtime-specific connection details (advanced)
- Build libraries that expose the client type in public APIs

**Recommendation:** In 99% of cases, use the unified API. It's simpler, more flexible, and follows Rust best practices.

---

## See Also

- [API Reference](API_REFERENCE.md) - Full async client API
- [Getting Started](GETTING_STARTED.md) - Basic synchronous usage
- [Error Handling](ERROR_HANDLING.md) - Error handling in async contexts
- [Examples](../examples/) - More async examples
