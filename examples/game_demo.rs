use presenceforge::{ActivityBuilder, DiscordIpcClient, Result};
use std::thread;
use std::time::Duration;

/// Example showing a dynamic game status that changes over time
fn main() -> Result {
    let client_id = "YOUR-CLIENT-ID"; // Replace with your Discord app client ID
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
            .start_timestamp_now()
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
