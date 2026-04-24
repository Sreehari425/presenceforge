use clap::Parser;
use presenceforge::sync::DiscordIpcClient;
use presenceforge::Result;

/// Fetch user info from Discord READY payload.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Discord Application Client ID
    #[arg(short, long)]
    client_id: Option<String>,
}

fn main() -> Result {
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    let client_id = args
        .client_id
        .or_else(|| std::env::var("DISCORD_CLIENT_ID").ok())
        .unwrap_or_else(|| {
            eprintln!("Error: DISCORD_CLIENT_ID is required!");
            eprintln!("Provide it via:");
            eprintln!("  - Command line: cargo run --example ready_event -- --client-id YOUR_ID");
            eprintln!("  - Environment: DISCORD_CLIENT_ID=YOUR_ID cargo run --example ready_event");
            std::process::exit(1);
        });

    let mut client = DiscordIpcClient::new(client_id)?;

    // Performs handshake and returns typed READY data when available.
    let ready = client.connect_with_ready()?;

    match ready.and_then(|r| r.user) {
        Some(user) => {
            if let Some(username) = user.username {
                println!("[Discord] Authenticated as {username}");
            }
            if let Some(id) = user.id {
                println!("[Discord] User ID: {id}");
            }
        }
        None => {
            eprintln!("READY event payload did not include user info");
        }
    }

    client.close();
    Ok(())
}
