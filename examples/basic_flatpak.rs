// Basic example for connecting to Flatpak Discord using custom path configuration
//
// This is a simplified version of basic.rs that explicitly connects to Flatpak Discord
// by discovering and selecting the Flatpak Discord pipe path.
// If Flatpak Discord is not active, it will fallback to standard Discord.

use clap::Parser;
use presenceforge::sync::DiscordIpcClient;
use presenceforge::{ActivityBuilder, IpcConnection, PipeConfig, Result};
use std::time::Duration;

/// Discord Rich Presence Flatpak Example
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
            eprintln!("  - Command line: cargo run --example basic_flatpak -- --client-id YOUR_ID");
            eprintln!(
                "  - Environment: DISCORD_CLIENT_ID=YOUR_ID cargo run --example basic_flatpak"
            );
            eprintln!("  - .env file: Create .env from .env.example and set DISCORD_CLIENT_ID");
            std::process::exit(1);
        });

    // Discover all available Discord pipes
    let pipes = IpcConnection::discover_pipes();

    if pipes.is_empty() {
        eprintln!("No Discord pipes found. Is Discord running?");
        return Ok(());
    }

    // Try to find and connect to Flatpak Discord first
    let flatpak_pipe = pipes
        .iter()
        .find(|p| p.path.contains("app/com.discordapp.Discord"));

    let (selected_pipe, is_flatpak) = if let Some(pipe) = flatpak_pipe {
        println!("Found Flatpak Discord at: {}", pipe.path);
        println!("Attempting to connect to Flatpak Discord...");
        (pipe, true)
    } else {
        println!(
            "Flatpak Discord not found, using standard Discord at: {}",
            pipes[0].path
        );
        (&pipes[0], false)
    };

    let config = Some(PipeConfig::CustomPath(selected_pipe.path.clone()));

    let mut client = DiscordIpcClient::new_with_config(&client_id, config.clone())?;

    // Try to perform handshake
    match client.connect() {
        Ok(_) => {
            println!("✓ Connected successfully!");
        }
        Err(e) => {
            if is_flatpak {
                println!("Failed to connect to Flatpak pipe: {}", e);
                println!("Trying standard Discord instead...");

                // Fallback to standard Discord
                let standard_pipe = pipes
                    .iter()
                    .find(|p| !p.path.contains("app/com.discordapp.Discord"))
                    .expect("No standard Discord pipe found");

                println!("Connecting to standard Discord at: {}", standard_pipe.path);
                let fallback_config = Some(PipeConfig::CustomPath(standard_pipe.path.clone()));
                client = DiscordIpcClient::new_with_config(client_id, fallback_config)?;
                client.connect()?;
                println!("✓ Connected to standard Discord successfully!");
            } else {
                return Err(e);
            }
        }
    }

    // Create activity using the builder pattern
    let activity = ActivityBuilder::new()
        .state("Running on Flatpak")
        .details("Using custom pipe configuration")
        .start_timestamp_now()?
        .large_image("car")
        .large_text("This is a large image")
        .button("View Car", "https://google.com")
        .button("Documentation", "https://docs.rs/presenceforge")
        .build();

    // Set the activity
    client.set_activity(&activity)?;
    println!("✓ Activity set!");

    // Keep activity for some time
    println!("Keeping activity visible for 10 seconds...");
    std::thread::sleep(Duration::from_secs(10));

    // Clear the activity
    client.clear_activity()?;
    println!("✓ Activity cleared!");

    // Connection is automatically closed when client is dropped
    Ok(())
}
