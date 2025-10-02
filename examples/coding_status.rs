use presenceforge::{ActivityBuilder, DiscordIpcClient, Result};
use std::time::Duration;

/// Example showing developer coding status
fn main() -> Result {
    let client_id = "YOUR-CLIENT-ID"; // Replace with your Discord app client ID
    let mut client = DiscordIpcClient::new(client_id)?;

    println!(" Starting Discord Rich Presence for Codings");
    client.connect()?;

    // Coding activity
    let activity = ActivityBuilder::new()
        .state("Writing Rust code")
        .details("Building Discord RPC library")
        .start_timestamp_now()
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
        .start_timestamp_now()
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
