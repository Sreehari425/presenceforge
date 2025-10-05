use clap::Parser;
use presenceforge::{ActivityBuilder, DiscordIpcClient, Result};
use std::time::Duration;

/// Discord Rich Presence Basic Example
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

    let client_id = args
        .client_id
        .or_else(|| std::env::var("DISCORD_CLIENT_ID").ok())
        .unwrap_or_else(|| {
            eprintln!("Error: DISCORD_CLIENT_ID is required!");
            eprintln!("Provide it via:");
            eprintln!("  - Command line: cargo run --example basic -- --client-id YOUR_ID");
            eprintln!("  - Environment: DISCORD_CLIENT_ID=YOUR_ID cargo run --example basic");
            eprintln!("  - .env file: Create .env from .env.example and set DISCORD_CLIENT_ID");
            std::process::exit(1);
        });

    let mut client = DiscordIpcClient::new(&client_id)?;

    // Perform handshake
    client.connect()?;

    // Create activity using the builder pattern
    let activity = ActivityBuilder::new()
        .state("Playing a game")
        .details("In the menu")
        .start_timestamp_now()?
        .large_image("car")
        .large_text("This is a large image")
        .button(" View Car", "https://google.com") // anylink you can put i ran out of ideas
        .button(" Documentation", "https://docs.rs/presenceforge")
        .build();

    // Set the activity
    client.set_activity(&activity)?;

    // Keep activity for some time
    std::thread::sleep(Duration::from_secs(30));

    // Clear the activity
    client.clear_activity()?;

    // Connection is automatically closed when client is dropped
    Ok(())
}
