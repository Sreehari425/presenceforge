use presenceforge::{
    Activity, ActivityAssets, ActivityButton, ActivityTimestamps, DiscordIpcClient, Result,
};
use std::time::{SystemTime, UNIX_EPOCH};
use clap::Parser;

/// Discord Rich Presence Custom Activity Example
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Discord Application Client ID
    #[arg(short, long)]
    client_id: Option<String>,
}

/// Example showing manual Activity creation without the builder pattern
fn main() -> Result {
    // Load .env file if it exists (optional)
    let _ = dotenvy::dotenv();
    
    let args = Args::parse();
    
    let client_id = args.client_id
        .or_else(|| std::env::var("DISCORD_CLIENT_ID").ok())
        .unwrap_or_else(|| {
            eprintln!("Error: DISCORD_CLIENT_ID is required!");
            eprintln!("Provide it via:");
            eprintln!("  - Command line: cargo run --example custom_activity -- --client-id YOUR_ID");
            eprintln!("  - Environment: DISCORD_CLIENT_ID=YOUR_ID cargo run --example custom_activity");
            eprintln!("  - .env file: Create .env from .env.example and set DISCORD_CLIENT_ID");
            std::process::exit(1);
        });
    
    let mut client = DiscordIpcClient::new(&client_id)?;

    println!(" Creating custom activity manually");
    client.connect()?;

    // Create activity manually for full control
    let custom_activity = Activity {
        state: Some("Custom State".to_string()),
        details: Some("Manually crafted activity".to_string()),
        timestamps: Some(ActivityTimestamps {
            start: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
            end: None,
        }),
        assets: Some(ActivityAssets {
            large_image: Some("custom_logo".to_string()),
            large_text: Some("Custom Application".to_string()),
            small_image: Some("status_online".to_string()),
            small_text: Some("Online".to_string()),
        }),
        party: None,
        secrets: None,
        buttons: Some(vec![
            ActivityButton {
                label: " Website".to_string(),
                url: "https://your-website.com".to_string(),
            },
            ActivityButton {
                label: " Discord".to_string(),
                url: "https://discord.gg/your-server".to_string(),
            },
        ]),
        instance: Some(false),
    };

    client.set_activity(&custom_activity)?;
    println!(" Custom activity set!");

    // Keep active for 20 seconds
    std::thread::sleep(std::time::Duration::from_secs(20));

    client.clear_activity()?;
    println!(" Activity cleared!");

    Ok(())
}
