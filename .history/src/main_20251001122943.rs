use presenceforge::{DiscordIpcClient, ActivityBuilder, Result};
use std::time::Duration;

fn main() -> Result<()> {
    let client_id = "1416069067697033216";
    let mut client = DiscordIpcClient::new(client_id)?;

    // Perform handshake
    client.connect()?;

    // Create activity using the builder pattern
    let activity = ActivityBuilder::new()
        .state("Playing a game")
        .details("In the menu")
        .start_timestamp_now()
        .large_image("your_image_key")
        .large_text("This is a large image")
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
