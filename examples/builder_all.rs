// Comprehensive example demonstrating ALL ActivityBuilder options
//
// This example shows every available builder method with explanations
// of what each field does and how it appears in Discord.

use presenceforge::{ActivityBuilder, DiscordIpcClient, Result};
use std::time::Duration;
use clap::Parser;

/// Discord Rich Presence Complete Builder Example
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Discord Application Client ID
    #[arg(short, long)]
    client_id: Option<String>,
}

fn main() -> Result {
    // Load .env file if it exists (optional)
    let _ = dotenvy::dotenv();
    
    let args = Args::parse();
    
    let client_id = args.client_id
        .or_else(|| std::env::var("DISCORD_CLIENT_ID").ok())
        .unwrap_or_else(|| {
            eprintln!("Error: DISCORD_CLIENT_ID is required!");
            eprintln!("Provide it via:");
            eprintln!("  - Command line: cargo run --example builder_all -- --client-id YOUR_ID");
            eprintln!("  - Environment: DISCORD_CLIENT_ID=YOUR_ID cargo run --example builder_all");
            eprintln!("  - .env file: Create .env from .env.example and set DISCORD_CLIENT_ID");
            std::process::exit(1);
        });
    
    println!("=== Complete ActivityBuilder Reference Example ===\n");

    let mut client = DiscordIpcClient::new(&client_id)?;

    // Perform handshake
    println!("Connecting to Discord...");
    client.connect()?;
    println!("✓ Connected!\n");

    // Create an activity with ALL possible fields
    println!("Setting activity with all available options...\n");

    let activity = ActivityBuilder::new()
        // ═══════════════════════════════════════════════════════════
        // BASIC TEXT FIELDS
        // ═══════════════════════════════════════════════════════════
        // State: First line of text (smaller text)
        // Example: "In a Match" or "Editing main.rs"
        .state("Playing a custom game")
        // Details: Second line of text (larger text above state)
        // Example: "Competitive - Rank 50" or "Workspace: my-project"
        .details("Custom game mode with friends")
        // ═══════════════════════════════════════════════════════════
        // TIMESTAMPS (Shows elapsed/remaining time)
        // ═══════════════════════════════════════════════════════════
        // Start timestamp: Shows "elapsed" time (e.g., "00:15 elapsed")
        // Use .start_timestamp_now() for current time
        .start_timestamp_now()
        // End timestamp: Shows "remaining" time (e.g., "02:30 left")
        // Note: If you set both start and end, Discord shows remaining time
        // Uncomment to try:
        // .end_timestamp(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 300)
        // ═══════════════════════════════════════════════════════════
        // IMAGES (Large and Small)
        // ═══════════════════════════════════════════════════════════
        // Large Image: Main big image shown on the left
        // This should be an asset key uploaded to your Discord app
        // Go to: Discord Developer Portal > Your App > Rich Presence > Art Assets
        .large_image("car")
        // Large Image Text: Tooltip shown when hovering over large image
        .large_text("This is the large image - shows on hover!")
        // Small Image: Smaller circular image shown in bottom-right of large image
        // Also needs to be uploaded as an asset
        .small_image("rust_logo")
        // Small Image Text: Tooltip shown when hovering over small image
        .small_text("Built with Rust 🦀")
        // ═══════════════════════════════════════════════════════════
        // PARTY (Multiplayer/Group information)
        // ═══════════════════════════════════════════════════════════
        // Party: Shows "X of Y" (e.g., "2 of 4" for a party)
        // Useful for multiplayer games showing current players
        // Parameters: party_id, current_size, max_size
        .party("party-12345", 2, 4)
        // ═══════════════════════════════════════════════════════════
        // BUTTONS (Clickable buttons - max 2)
        // ═══════════════════════════════════════════════════════════
        // Button 1: First clickable button
        // Parameters: label (button text), url (where it goes)
        .button(" View Game", "https://example.com/game")
        // Button 2: Second clickable button
        // Note: Discord only allows up to 2 buttons
        .button(" Documentation", "https://docs.rs/presenceforge")
        // ═══════════════════════════════════════════════════════════
        // SECRETS (For "Ask to Join" and spectate features)
        // ═══════════════════════════════════════════════════════════
        // Join Secret: Used for "Ask to Join" button
        // Your game uses this to let players join via Discord
        .join_secret("join_secret_12345")
        // Spectate Secret: Used for "Spectate" button
        // Allows others to spectate your game
        .spectate_secret("spectate_secret_67890")
        // Match Secret: Unique identifier for the current match/game
        .match_secret("match_secret_abcde")
        // ═══════════════════════════════════════════════════════════
        // INSTANCE (Boolean flag)
        // ═══════════════════════════════════════════════════════════
        // Instance: Whether this is an instanced context (like a match)
        // Set to true for unique game instances, false for general activities
        .instance(true)
        // Build the activity
        .build();

    // Set the activity
    client.set_activity(&activity)?;
    println!("✓ Activity set successfully!");
    println!("\n📱 Check your Discord profile to see the activity!");
    println!("   You should see:");
    println!("   • Details: 'Custom game mode with friends'");
    println!("   • State: 'Playing a custom game'");
    println!("   • Large image with tooltip");
    println!("   • Small image (Rust logo) in corner");
    println!("   • Party info: '2 of 4'");
    println!("   • Two clickable buttons");
    println!("   • Elapsed time counter");

    // Keep activity visible for 30 seconds
    println!("\nKeeping activity visible for 30 seconds...");
    for i in 1..=30 {
        print!("\r   {} seconds remaining... ", 31 - i);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        std::thread::sleep(Duration::from_secs(1));
    }
    println!("\r   ✓ Time's up!                    ");

    // Clear the activity
    println!("\nClearing activity...");
    client.clear_activity()?;
    println!("✓ Activity cleared!");

    println!("\n=== Example completed! ===\n");

    // ═══════════════════════════════════════════════════════════
    // VISUAL REFERENCE
    // ═══════════════════════════════════════════════════════════
    println!("📊 Visual Layout Reference:");
    println!("┌─────────────────────────────────────────┐");
    println!("│  [Large Image]    DETAILS               │");
    println!("│    [Small]        State                 │");
    println!("│                   Party: 2 of 4         │");
    println!("│                   ⏱ 00:15 elapsed       │");
    println!("│                                         │");
    println!("│  [Button 1]       [Button 2]            │");
    println!("└─────────────────────────────────────────┘");
    println!();
    println!("💡 Tips:");
    println!("   • Large/Small images must be uploaded as assets in Discord Developer Portal");
    println!("   • Asset keys are case-sensitive");
    println!("   • Maximum 2 buttons allowed");
    println!("   • Timestamps are Unix timestamps (seconds since epoch)");
    println!("   • Party size shows as 'X of Y' in Discord");
    println!("   • Secrets enable 'Ask to Join' and 'Spectate' features");
    println!("   • Hover tooltips work on large_text and small_text");

    Ok(())
}
