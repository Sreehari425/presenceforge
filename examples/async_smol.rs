use clap::Parser;
use presenceforge::async_io::smol::client::new_discord_ipc_client;
use presenceforge::{ActivityBuilder, Result};
use std::time::Duration;

/// Discord Rich Presence Smol Example
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
            eprintln!("  - Command line: cargo run --example async_smol --features smol-runtime -- --client-id YOUR_ID");
            eprintln!("  - Environment: DISCORD_CLIENT_ID=YOUR_ID cargo run --example async_smol --features smol-runtime");
            eprintln!("  - .env file: Create .env from .env.example and set DISCORD_CLIENT_ID");
            std::process::exit(1);
        });

    smol::block_on(async {
        let mut client = new_discord_ipc_client(&client_id).await?;

        // Perform handshake
        client.connect().await?;

        // Create activity using the builder pattern
        let activity = ActivityBuilder::new()
            .state("Playing a game")
            .details("In the menu")
            .start_timestamp_now()
            .large_image("car")
            .large_text("This is a large image")
            .button(" View Car", "https://google.com")
            .button(" Documentation", "https://docs.rs/presenceforge")
            .build();

        println!(
            "Activity payload: {}",
            serde_json::to_string_pretty(&activity).unwrap()
        );

        // Set the activity
        println!("Setting Discord Rich Presence activity...");
        match client.set_activity(&activity).await {
            Ok(_) => println!("Successfully set activity!"),
            Err(e) => println!("Failed to set activity: {:?}", e),
        }

        // Keep activity for some time
        println!("Sleeping for 10 seconds to maintain presence...");
        smol::Timer::after(Duration::from_secs(10)).await;

        // Clear the activity
        client.clear_activity().await?;

        // Connection is automatically closed when client is dropped
        Ok(())
    })
}
