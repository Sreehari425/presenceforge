use presenceforge::{Activity, ActivityAssets, ActivityTimestamps, ActivityButton, DiscordIpcClient, Result};
use std::time::{SystemTime, UNIX_EPOCH};

/// Example showing manual Activity creation without the builder pattern
fn main() -> Result<()> {
    let client_id = "YOUR-CLIENT-ID"; // Replace with your Discord app client ID
    let mut client = DiscordIpcClient::new(client_id)?;

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
                    .as_secs() as i64
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