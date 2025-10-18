use clap::Parser;
#[cfg(feature = "tokio-runtime")]
use presenceforge::{ActivityBuilder, AsyncDiscordIpcClient, Result};
#[cfg(feature = "tokio-runtime")]
use std::time::{SystemTime, UNIX_EPOCH};

/// Discord Rich Presence - Update Activity Without Resetting Timer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Discord Application Client ID
    #[arg(short, long)]
    client_id: Option<String>,
}
#[cfg(feature = "tokio-runtime")]
#[tokio::main]
async fn main() -> Result {
    // Load .env file if it exists (optional)
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    let client_id = args
        .client_id
        .or_else(|| std::env::var("DISCORD_CLIENT_ID").ok())
        .unwrap_or_else(|| {
            eprintln!("Error: DISCORD_CLIENT_ID is required!");
            eprintln!("Provide it via:");
            eprintln!("  - Command line: cargo run --example update_activity_tokio --features tokio-runtime -- --client-id YOUR_ID");
            eprintln!("  - Environment: DISCORD_CLIENT_ID=YOUR_ID cargo run --example update_activity_tokio --features tokio-runtime");
            eprintln!("  - .env file: Create .env from .env.example and set DISCORD_CLIENT_ID");
            std::process::exit(1);
        });

    let mut client = AsyncDiscordIpcClient::new(&client_id).await?;

    // Perform handshake
    client.connect().await?;
    println!("✓ Connected to Discord!");

    // Get the initial timestamp - this will stay consistent across all updates
    let start_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    println!("\n=== Demonstrating Activity Updates Without Timer Reset ===\n");
    println!("The timer will remain consistent throughout all state changes!\n");

    // Initial activity - Main Menu
    println!(" State 1: Main Menu");
    let activity = ActivityBuilder::new()
        .state("Browsing menus")
        .details("In main menu")
        .start_timestamp(start_timestamp)
        .large_image("game")
        .large_text("Game Icon")
        .small_image("idle")
        .small_text("Idle")
        .button(" Play Game", "https://example.com")
        .build();

    client.set_activity(&activity).await?;
    println!("✓ Activity set: In main menu");
    println!("   Timer started at: {}", start_timestamp);

    // Wait 5 seconds
    println!("\n Waiting 5 seconds...\n");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Update to In Game - SAME TIMESTAMP
    println!(" State 2: In Game");
    let activity = ActivityBuilder::new()
        .state("Playing Solo")
        .details("In game - Level 5")
        .start_timestamp(start_timestamp) // SAME timestamp = timer continues!
        .large_image("game")
        .large_text("Game Icon")
        .small_image("playing")
        .small_text("Playing")
        .button("🎮 Play Game", "https://example.com")
        .build();

    client.set_activity(&activity).await?;
    println!("✓ Activity updated: In game");
    println!("   Timer continues from: {}", start_timestamp);
    println!("   Notice: The elapsed time on Discord continues!");

    // Wait 5 seconds
    println!("\n Waiting 5 seconds...\n");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Update to Multiplayer - SAME TIMESTAMP
    println!(" State 3: Multiplayer");
    let activity = ActivityBuilder::new()
        .state("In Multiplayer Match")
        .details("Team Deathmatch - 2/8 players")
        .start_timestamp(start_timestamp) // SAME timestamp = timer continues!
        .large_image("game")
        .large_text("Game Icon")
        .small_image("multiplayer")
        .small_text("Online")
        .button(" Join Game", "https://example.com")
        .build();

    client.set_activity(&activity).await?;
    println!("✓ Activity updated: Multiplayer");
    println!("   Timer continues from: {}", start_timestamp);
    println!("   Notice: The elapsed time keeps going!");

    // Wait 5 seconds
    println!("\n Waiting 5 seconds...\n");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Update to Loading - SAME TIMESTAMP
    println!(" State 4: Loading");
    let activity = ActivityBuilder::new()
        .state("Loading next map...")
        .details("Please wait")
        .start_timestamp(start_timestamp) // SAME timestamp = timer continues!
        .large_image("game")
        .large_text("Game Icon")
        .small_image("loading")
        .small_text("Loading")
        .button("🎮 Play Game", "https://example.com")
        .build();

    client.set_activity(&activity).await?;
    println!("✓ Activity updated: Loading");
    println!("   Timer continues from: {}", start_timestamp);

    // Wait 5 seconds
    println!("\n Waiting 5 seconds...\n");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Back to Main Menu - SAME TIMESTAMP
    println!(" State 5: Back to Main Menu");
    let activity = ActivityBuilder::new()
        .state("Browsing menus")
        .details("In main menu")
        .start_timestamp(start_timestamp) // SAME timestamp = timer continues!
        .large_image("game")
        .large_text("Game Icon")
        .small_image("idle")
        .small_text("Idle")
        .button(" Play Game", "https://example.com")
        .build();

    client.set_activity(&activity).await?;
    println!("✓ Activity updated: Back to main menu");
    println!("   Timer continues from: {}", start_timestamp);
    println!("   Total elapsed time: ~20 seconds (5s × 4 intervals)");

    // Keep showing for a bit longer
    println!("\n Keeping presence active for 10 more seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    println!("\n=== Summary ===");
    println!("✓ Changed activity 5 times");
    println!("✓ Timer remained consistent throughout all changes");
    println!("✓ Total session time: ~30 seconds");
    println!("\nKey takeaway: By using the same start_timestamp across all");
    println!("activity updates, the elapsed time continues without resetting!");

    // Clear the activity
    println!("\n Clearing activity...");
    client.clear_activity().await?;
    println!("✓ Activity cleared!");

    // Connection is automatically closed when client is dropped
    Ok(())
}
#[cfg(not(feature = "tokio-runtime"))]
fn main() {
    eprintln!("This example requires the `tokio-runtime` feature.");
}
