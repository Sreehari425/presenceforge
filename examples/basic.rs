use presenceforge::{ActivityBuilder, DiscordIpcClient, Result};
use std::time::Duration;

fn main() -> Result {
    let client_id = "YOUR-CLIENT-ID";
    let mut client = DiscordIpcClient::new(client_id)?;

    // Perform handshake
    client.connect()?;

    // Create activity using the builder pattern
    let activity = ActivityBuilder::new()
        .state("Playing a game")
        .details("In the menu")
        .start_timestamp_now()
        .large_image("car")
        .large_text("This is a large image")
        .button(" View Car", "https://google.com") // anylink you can put i ran out of ideas
        .button(" Documentation", "https://docs.rs/presenceforge")
        .build();

    // Set the activity
    client.set_activity(&activity)?;

    // Keep activity for some time
    std::thread::sleep(Duration::from_secs(10));

    // Clear the activity
    client.clear_activity()?;

    // Connection is automatically closed when client is dropped
    Ok(())
}
