// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025-2026 Sreehari Anil and project contributors

use clap::Parser;
use presenceforge::sync::DiscordIpcClient;
use presenceforge::{ActivityBuilder, EventData, Result};
use std::time::Duration;

/// Demonstrate subscribing to and receiving Discord events.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Discord Application Client ID
    #[arg(short, long)]
    client_id: Option<String>,
}

fn main() -> Result {
    // Initialize logging so we can see what's happening
    env_logger::init();

    let _ = dotenvy::dotenv();
    let args = Args::parse();

    let client_id = args
        .client_id
        .or_else(|| std::env::var("DISCORD_CLIENT_ID").ok())
        .unwrap_or_else(|| {
            eprintln!("Error: DISCORD_CLIENT_ID is required!");
            std::process::exit(1);
        });

    let mut client = DiscordIpcClient::new(client_id)?;
    client.connect()?;
    println!("✓ Connected to Discord!");

    // Set an activity with a join secret to enable the "Join" button in Discord
    let activity = ActivityBuilder::new()
        .state("Testing Event System")
        .details("Waiting for someone to join...")
        .party_simple(1, 5)
        .join_secret("very_secret_join_token")
        .build();

    client.set_activity(&activity)?;
    println!("✓ Activity set with join secret.");

    // Subscribe to ACTIVITY_JOIN event
    // Note: Discord requires us to subscribe to receive these events
    client.subscribe("ACTIVITY_JOIN", serde_json::json!({}))?;
    println!("✓ Subscribed to ACTIVITY_JOIN event.");
    println!("✓ You can also check for events without blocking using client.poll_event().");

    // Check once without blocking
    match client.poll_event()? {
        Some(_) => println!("! Found an early event!"),
        None => println!("✓ No immediate events (as expected)."),
    }

    println!("\n--- Waiting for events (Ctrl+C to stop) ---");
    println!("If someone clicks 'Join' on your profile in Discord, you'll see an event here.");

    // Listen for events in a loop
    // In a real app, you might run this in a separate thread
    loop {
        match client.next_event() {
            Ok(EventData::ActivityJoin(event)) => {
                println!("\n RECEIVED EVENT: Someone wants to join!");
                println!("   Secret: {}", event.secret);
            }
            Ok(EventData::Error(e)) => {
                eprintln!("\n Discord Error: {} - {}", e.code, e.message);
            }
            Ok(EventData::Unknown { name, .. }) => {
                println!("\n Received unknown event: {}", name);
            }
            Ok(_) => {} // Ignore other events like READY
            Err(e) => {
                eprintln!("\n Communication Error: {}", e);
                break;
            }
        }

        // Small sleep to avoid hogging CPU in this example loop
        std::thread::sleep(Duration::from_millis(100));

        // In a real app, you might break based on some condition:
        // break;
    }

    // This code would be reached if the loop above breaks
    println!("\nUnsubscribing from events...");
    client.unsubscribe("ACTIVITY_JOIN", serde_json::json!({}))?;
    println!("✓ Unsubscribed.");

    Ok(())
}
