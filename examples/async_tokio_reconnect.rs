use clap::Parser;
use presenceforge::retry::{with_retry_async, RetryConfig};
use presenceforge::{ActivityBuilder, AsyncDiscordIpcClient, Result};
use tokio::time::{sleep, Duration};

/// Discord Rich Presence Async Tokio Reconnection Example
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Discord Application Client ID
    #[arg(short, long)]
    client_id: Option<String>,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Load .env file if it exists (optional)
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    let client_id = args.client_id
        .or_else(|| std::env::var("DISCORD_CLIENT_ID").ok())
        .unwrap_or_else(|| {
            eprintln!("Error: DISCORD_CLIENT_ID is required!");
            eprintln!("Provide it via:");
            eprintln!("  - Command line: cargo run --example async_tokio_reconnect --features tokio-runtime -- --client-id YOUR_ID");
            eprintln!("  - Environment: DISCORD_CLIENT_ID=YOUR_ID cargo run --example async_tokio_reconnect --features tokio-runtime");
            eprintln!("  - .env file: Create .env from .env.example and set DISCORD_CLIENT_ID");
            std::process::exit(1);
        });

    println!("=== Discord Rich Presence Async Tokio Reconnection Examples ===\n");

    // Example 1: Basic connection with manual reconnect
    println!("Example 1: Manual reconnection on connection loss");
    println!("{}", "-".repeat(50));
    example_manual_reconnect(&client_id).await?;

    println!("\n");

    // Example 2: Connection with automatic retry
    println!("Example 2: Connection with automatic retry");
    println!("{}", "-".repeat(50));
    example_auto_retry(&client_id).await?;

    println!("\n");

    // Example 3: Resilient presence loop
    println!("Example 3: Resilient presence maintenance loop");
    println!("{}", "-".repeat(50));
    example_resilient_loop(&client_id).await?;

    Ok(())
}

/// Example 1: Manual reconnection when connection is lost
async fn example_manual_reconnect(client_id: &str) -> Result {
    println!("Connecting to Discord...");

    let mut client = AsyncDiscordIpcClient::new(client_id).await?;
    client.connect().await?;
    println!("✓ Connected!");

    let activity = ActivityBuilder::new()
        .state("Example 1: Manual Reconnect")
        .details("Testing async reconnection")
        .start_timestamp_now()
        .build();

    println!("Setting activity...");

    // Try to set activity, and reconnect if connection fails
    match client.set_activity(&activity).await {
        Ok(_) => println!("✓ Activity set!"),
        Err(e) if e.is_connection_error() => {
            println!("⚠ Connection error detected: {}", e);
            println!("  Attempting to reconnect...");

            // Reconnect and retry
            client.reconnect().await?;
            println!("✓ Reconnected!");

            client.set_activity(&activity).await?;
            println!("✓ Activity set after reconnection!");
        }
        Err(e) => return Err(e),
    }

    sleep(Duration::from_secs(2)).await;
    client.clear_activity().await?;
    println!("✓ Activity cleared!");

    Ok(())
}

/// Example 2: Connection with automatic retry
async fn example_auto_retry(client_id: &str) -> Result {
    println!("Connecting with automatic retry (5 attempts)...");

    // Use the retry utility to automatically retry connection
    let config = RetryConfig::with_max_attempts(5);

    let mut client = with_retry_async(&config, || {
        Box::pin(async {
            println!("  Attempting to connect...");
            AsyncDiscordIpcClient::new(client_id).await
        })
    })
    .await?;

    println!("✓ Connected successfully!");

    // Perform handshake
    client.connect().await?;
    println!("✓ Handshake completed!");

    // Set activity
    let activity = ActivityBuilder::new()
        .state("Example 2: Auto Retry")
        .details("With exponential backoff")
        .start_timestamp_now()
        .build();

    client.set_activity(&activity).await?;
    println!("✓ Activity set!");

    sleep(Duration::from_secs(2)).await;
    client.clear_activity().await?;
    println!("✓ Activity cleared!");

    Ok(())
}

/// Example 3: Resilient presence maintenance loop with automatic reconnection
async fn example_resilient_loop(client_id: &str) -> Result {
    println!("Starting resilient presence maintenance loop...");

    // Connect with retry
    let config = RetryConfig::with_max_attempts(3);
    let mut client =
        with_retry_async(&config, || Box::pin(AsyncDiscordIpcClient::new(client_id))).await?;

    client.connect().await?;
    println!("✓ Connected!");

    let activity = ActivityBuilder::new()
        .state("Example 3: Resilient Loop")
        .details("Auto-reconnecting")
        .start_timestamp_now()
        .build();

    println!("Maintaining presence (will update 3 times)...");

    // Maintain presence with automatic reconnection
    for i in 1..=3 {
        match client.set_activity(&activity).await {
            Ok(_) => {
                println!("  ✓ Update #{} successful", i);
            }
            Err(e) if e.is_connection_error() => {
                println!("  ⚠ Connection lost on update #{}, reconnecting...", i);

                // Try to reconnect
                match client.reconnect().await {
                    Ok(_) => {
                        println!("  ✓ Reconnected successfully!");
                        // Retry the update
                        client.set_activity(&activity).await?;
                        println!("  ✓ Update #{} successful after reconnection", i);
                    }
                    Err(reconnect_err) => {
                        println!("  ✗ Reconnection failed: {}", reconnect_err);
                        println!("  💡 Discord may have been closed");
                        return Err(reconnect_err);
                    }
                }
            }
            Err(e) => {
                println!("  ✗ Unexpected error: {}", e);
                return Err(e);
            }
        }

        if i < 3 {
            sleep(Duration::from_secs(2)).await;
        }
    }

    client.clear_activity().await?;
    println!("✓ Activity cleared!");
    println!("✓ Resilient loop completed successfully!");

    Ok(())
}
