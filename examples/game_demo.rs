use clap::Parser;
use presenceforge::sync::DiscordIpcClient;
use presenceforge::{ActivityBuilder, Result};
use std::thread;
use std::time::Duration;
/// Discord Rich Presence Game Demo Example
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Discord Application Client ID
    #[arg(short, long)]
    client_id: Option<String>,
}

/// Example showing a dynamic game status that changes over time
fn main() -> Result {
    // Load .env file if it exists (optional)
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    let client_id = args
        .client_id
        .or_else(|| std::env::var("DISCORD_CLIENT_ID").ok())
        .unwrap_or_else(|| {
            eprintln!("Error: DISCORD_CLIENT_ID is required!");
            eprintln!("Provide it via:");
            eprintln!("  - Command line: cargo run --example game_demo -- --client-id YOUR_ID");
            eprintln!("  - Environment: DISCORD_CLIENT_ID=YOUR_ID cargo run --example game_demo");
            eprintln!("  - .env file: Create .env from .env.example and set DISCORD_CLIENT_ID");
            std::process::exit(1);
        });

    let mut client = DiscordIpcClient::new(client_id)?;

    println!("🎮 Starting Discord Rich Presence for Game Demo...");
    client.connect()?;
    println!(" Connected to Discord!");

    // Game states to cycle through
    let game_states = vec![
        (" Main Menu", "Selecting character", "menu_bg", "Main Menu"),
        (
            " Forest Level",
            "Fighting goblins",
            "forest_map",
            "Enchanted Forest",
        ),
        (" Castle", "Boss battle", "castle_map", "Dark Castle"),
        (" Victory Screen", "Quest completed!", "victory", "Victory!"),
    ];

    for (i, (state, details, image_key, image_text)) in game_states.iter().enumerate() {
        println!("\n Game State {}: {}", i + 1, state);

        let activity = ActivityBuilder::new()
            .state(*state)
            .details(*details)
            .start_timestamp_now()?
            .large_image(*image_key) // You'd need to upload these to Discord
            .large_text(*image_text)
            .small_image("player_avatar")
            .small_text("Level 25 Warrior")
            .button(" Play Now", "https://your-game.com")
            .button(" Leaderboard", "https://your-game.com/leaderboard")
            .build();

        client.set_activity(&activity)?;

        // Stay in this state for 8 seconds
        thread::sleep(Duration::from_secs(8));
    }

    // Clear activity when game ends
    println!("\n Game ended, clearing activity...");
    client.clear_activity()?;
    println!(" Activity cleared!");

    Ok(())
}
