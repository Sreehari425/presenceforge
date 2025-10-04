use clap::Parser;
use presenceforge::retry::{with_retry, RetryConfig};
use presenceforge::{ActivityBuilder, DiscordIpcClient, Result};
use std::time::Duration;

/// Discord Rich Presence Connection Retry Example
///
/// This example demonstrates various retry patterns for handling connection issues
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Discord Application Client ID
    #[arg(short, long)]
    client_id: Option<String>,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Load .env file if it exists (optional)
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    let client_id = args.client_id
        .or_else(|| std::env::var("DISCORD_CLIENT_ID").ok())
        .unwrap_or_else(|| {
            eprintln!("Error: DISCORD_CLIENT_ID is required!");
            eprintln!("Provide it via:");
            eprintln!("  - Command line: cargo run --example connection_retry -- --client-id YOUR_ID");
            eprintln!("  - Environment: DISCORD_CLIENT_ID=YOUR_ID cargo run --example connection_retry");
            eprintln!("  - .env file: Create .env from .env.example and set DISCORD_CLIENT_ID");
            std::process::exit(1);
        });

    println!("=== Discord Rich Presence Connection Retry Examples ===\n");

    // Example 1: Basic connection with retry
    println!("Example 1: Connect with automatic retry");
    println!("{}", "-".repeat(50));
    example_basic_retry(&client_id)?;

    println!("\n");

    // Example 2: Manual reconnection on error
    println!("Example 2: Manual reconnection on connection loss");
    println!("{}", "-".repeat(50));
    example_manual_reconnect(&client_id)?;

    println!("\n");

    // Example 3: Custom retry configuration
    println!("Example 3: Custom retry configuration with exponential backoff");
    println!("{}", "-".repeat(50));
    example_custom_retry(&client_id)?;

    Ok(())
}

/// Example 1: Basic connection with automatic retry
fn example_basic_retry(client_id: &str) -> Result {
    println!("Attempting to connect with automatic retry (3 attempts)...");

    // Use the retry utility to automatically retry connection
    let config = RetryConfig::with_max_attempts(3);

    let mut client = with_retry(&config, || {
        println!("  Connecting...");
        DiscordIpcClient::new(client_id)
    })?;

    println!("✓ Connected successfully!");

    // Perform handshake
    client.connect()?;
    println!("✓ Handshake completed!");

    // Set activity
    let activity = ActivityBuilder::new()
        .state("Example 1: Auto Retry")
        .details("Testing connection retry")
        .start_timestamp_now()
        .build();

    client.set_activity(&activity)?;
    println!("✓ Activity set!");

    std::thread::sleep(Duration::from_secs(2));
    client.clear_activity()?;

    Ok(())
}

/// Example 2: Manual reconnection when connection is lost
fn example_manual_reconnect(client_id: &str) -> Result {
    println!("Connecting to Discord...");

    let mut client = DiscordIpcClient::new(client_id)?;
    client.connect()?;
    println!("✓ Connected!");

    let activity = ActivityBuilder::new()
        .state("Example 2: Manual Reconnect")
        .details("Testing reconnection")
        .start_timestamp_now()
        .build();

    println!("Setting activity...");

    // Try to set activity, and reconnect if connection fails
    match client.set_activity(&activity) {
        Ok(_) => println!("✓ Activity set!"),
        Err(e) if e.is_connection_error() => {
            println!("⚠ Connection error detected: {}", e);
            println!("  Attempting to reconnect...");

            // Reconnect and retry
            client.reconnect()?;
            println!("✓ Reconnected!");

            client.set_activity(&activity)?;
            println!("✓ Activity set after reconnection!");
        }
        Err(e) => return Err(e),
    }

    std::thread::sleep(Duration::from_secs(2));
    client.clear_activity()?;

    Ok(())
}

/// Example 3: Custom retry configuration with exponential backoff
fn example_custom_retry(client_id: &str) -> Result {
    println!("Connecting with custom retry configuration:");
    println!("  - Max attempts: 5");
    println!("  - Initial delay: 500ms");
    println!("  - Max delay: 8000ms");
    println!("  - Backoff multiplier: 2.0");

    // Create a custom retry configuration
    let config = RetryConfig::new(
        5,      // max_attempts
        500,    // initial_delay_ms
        8000,   // max_delay_ms
        2.0,    // backoff_multiplier
    );

    // Show the delay progression
    println!("\nDelay progression:");
    for attempt in 0..config.max_attempts {
        let delay = config.delay_for_attempt(attempt);
        println!("  Attempt {}: {}ms delay", attempt + 1, delay.as_millis());
    }

    println!("\nConnecting...");

    let mut client = with_retry(&config, || {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        DiscordIpcClient::new(client_id)
    })?;

    println!("\n✓ Connected!");

    client.connect()?;
    println!("✓ Handshake completed!");

    let activity = ActivityBuilder::new()
        .state("Example 3: Custom Retry")
        .details("With exponential backoff")
        .start_timestamp_now()
        .build();

    client.set_activity(&activity)?;
    println!("✓ Activity set!");

    std::thread::sleep(Duration::from_secs(2));
    client.clear_activity()?;

    Ok(())
}

/// Example 4: Resilient operation with retry wrapper
#[allow(dead_code)]
fn example_resilient_operation(client_id: &str) -> Result {
    println!("Running resilient operation...");

    let config = RetryConfig::with_max_attempts(3);

    // Create a resilient connection function
    let mut connect = || {
        let mut client = DiscordIpcClient::new(client_id)?;
        client.connect()?;
        Ok(client)
    };

    // Connect with retry
    let mut client = with_retry(&config, &mut connect)?;

    // Helper function to perform operations with automatic reconnect
    let mut perform_with_retry = |op: &dyn Fn(&mut DiscordIpcClient) -> Result| {
        match op(&mut client) {
            Ok(_) => Ok(()),
            Err(e) if e.is_recoverable() => {
                println!("  Operation failed, reconnecting...");
                client.reconnect()?;
                op(&mut client)
            }
            Err(e) => Err(e),
        }
    };

    // Set activity with auto-retry
    let activity = ActivityBuilder::new()
        .state("Resilient Operation")
        .details("With auto-reconnect")
        .build();

    perform_with_retry(&|c| c.set_activity(&activity))?;
    println!("✓ Activity set with resilient operation!");

    std::thread::sleep(Duration::from_secs(2));

    perform_with_retry(&|c| c.clear_activity().map(|_| ()))?;
    println!("✓ Activity cleared!");

    Ok(())
}
