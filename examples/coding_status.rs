use clap::Parser;
use presenceforge::sync::DiscordIpcClient;
use presenceforge::{ActivityBuilder, Result};
use std::time::Duration;

/// Discord Rich Presence Coding Status Example
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Discord Application Client ID
    #[arg(short, long)]
    client_id: Option<String>,
}

/// Example showing developer coding status
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
            eprintln!("  - Command line: cargo run --example coding_status -- --client-id YOUR_ID");
            eprintln!(
                "  - Environment: DISCORD_CLIENT_ID=YOUR_ID cargo run --example coding_status"
            );
            eprintln!("  - .env file: Create .env from .env.example and set DISCORD_CLIENT_ID");
            std::process::exit(1);
        });

    let mut client = DiscordIpcClient::new(client_id)?;

    println!(" Starting Discord Rich Presence for Codings");
    client.connect()?;

    // Coding activity
    let activity = ActivityBuilder::new()
        .state("Writing Rust code")
        .details("Building Discord RPC library")
        .start_timestamp_now()?
        .large_image("rust_logo") // Upload Rust logo to Discord
        .large_text("Rust Programming")
        .small_image("vscode") // Upload VS Code icon to Discord
        .small_text("VS Code")
        .button(
            "View on GitHub",
            "https://github.com/your-username/presenceforge",
        )
        .button(" Rust Docs", "https://doc.rust-lang.org")
        .build();

    client.set_activity(&activity)?;
    println!(" Coding status set! Others can see you're programming in Rust.");

    // Keep the status active for 30 seconds
    // OR if you have any custom logic to track time put it here
    println!("  Keeping status for 30 seconds...");
    std::thread::sleep(Duration::from_secs(30));

    // Update to debugging status
    let debugging_activity = ActivityBuilder::new()
        .state("Debugging")
        .details("Fixing async issues")
        .start_timestamp_now()?
        .large_image("rust_logo")
        .large_text("Rust Programming")
        .small_image("debug_icon")
        .small_text("Debugging Mode")
        .button(
            " View on GitHub",
            "https://github.com/your-username/presenceforge",
        )
        .build();

    client.set_activity(&debugging_activity)?;
    println!(" Updated to debugging status!");

    std::thread::sleep(Duration::from_secs(15));

    client.clear_activity()?;
    println!(" Cleared coding status!");

    Ok(())
}
