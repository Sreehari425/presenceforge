use presenceforge::async_io::tokio::client::new_discord_ipc_client;
use presenceforge::{ActivityBuilder, Result};

#[tokio::main]
async fn main() -> Result {
    let client_id = "1416069067697033216";
    let mut client = new_discord_ipc_client(client_id).await?;

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
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Clear the activity
    client.clear_activity().await?;

    // Connection is automatically closed when client is dropped
    Ok(())
}
