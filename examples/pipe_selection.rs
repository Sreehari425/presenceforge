// Example demonstrating pipe selection and discovery features

use presenceforge::{ActivityBuilder, DiscordIpcClient, IpcConnection, PipeConfig};
use clap::Parser;

/// Discord IPC Pipe Selection Example
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Discord Application Client ID
    #[arg(short, long)]
    client_id: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if it exists (optional)
    let _ = dotenvy::dotenv();
    
    let args = Args::parse();
    
    let client_id = args.client_id
        .or_else(|| std::env::var("DISCORD_CLIENT_ID").ok())
        .unwrap_or_else(|| {
            eprintln!("Error: DISCORD_CLIENT_ID is required!");
            eprintln!("Provide it via:");
            eprintln!("  - Command line: cargo run --example pipe_selection -- --client-id YOUR_ID");
            eprintln!("  - Environment: DISCORD_CLIENT_ID=YOUR_ID cargo run --example pipe_selection");
            eprintln!("  - .env file: Create .env from .env.example and set DISCORD_CLIENT_ID");
            std::process::exit(1);
        });
    
    println!("=== Discord IPC Pipe Selection Example ===\n");

    // 1. Discover all available Discord IPC pipes
    println!("1. Discovering available Discord IPC pipes...");
    let pipes = IpcConnection::discover_pipes();

    if pipes.is_empty() {
        println!("   No Discord IPC pipes found. Is Discord running?");
        return Ok(());
    }

    println!("   Found {} pipe(s):", pipes.len());
    for pipe in &pipes {
        println!("   - Pipe {}: {}", pipe.pipe_number, pipe.path);
    }
    println!();

    // 2. Connect using auto-discovery (default behavior)
    println!("2. Connecting using auto-discovery (default)...");
    let mut client1 = DiscordIpcClient::new(&client_id)?;
    client1.connect()?;
    println!("   ✓ Connected successfully using auto-discovery");
    client1.clear_activity()?;
    drop(client1);
    println!();

    // 3. Connect to a specific pipe using discovered path
    if !pipes.is_empty() {
        let pipe_num = pipes[0].pipe_number;
        let pipe_path = pipes[0].path.clone();
        println!(
            "3. Connecting to specific pipe {} at {}...",
            pipe_num, pipe_path
        );
        let mut client2 = DiscordIpcClient::new_with_config(
            &client_id,
            Some(PipeConfig::CustomPath(pipe_path)),
        )?;
        client2.connect()?;
        println!("   ✓ Connected successfully to pipe {}", pipe_num);

        // Set an activity
        let activity = ActivityBuilder::new()
            .state("Using pipe selection")
            .details(&format!("Connected to pipe {}", pipe_num))
            .build();

        client2.set_activity(&activity)?;
        println!("   ✓ Activity set successfully");

        // Keep activity for a moment
        std::thread::sleep(std::time::Duration::from_secs(5));

        client2.clear_activity()?;
        println!("   ✓ Activity cleared");
        drop(client2);
    }
    println!();

    // 4. Connect using custom path (Unix only)
    #[cfg(unix)]
    {
        if !pipes.is_empty() {
            let custom_path = pipes[0].path.clone();
            println!("4. Connecting using custom path: {}...", custom_path);
            let mut client3 = DiscordIpcClient::new_with_config(
                "your_client_id",
                Some(PipeConfig::CustomPath(custom_path.clone())),
            )?;
            client3.connect()?;
            println!("   ✓ Connected successfully using custom path");
            client3.clear_activity()?;
            drop(client3);
            println!();
        }
    }

    // 5. Connect with timeout and specific pipe
    if !pipes.is_empty() {
        let pipe_num = pipes[0].pipe_number;
        let pipe_path = pipes[0].path.clone();
        println!(
            "5. Connecting with timeout (5000ms) to pipe {} at {}...",
            pipe_num, pipe_path
        );
        let mut client4 = DiscordIpcClient::new_with_config_and_timeout(
            "your_client_id",
            Some(PipeConfig::CustomPath(pipe_path)),
            5000,
        )?;
        client4.connect()?;
        println!("   ✓ Connected successfully with timeout");
        client4.clear_activity()?;
        drop(client4);
    }

    println!("\n=== All examples completed successfully! ===");
    Ok(())
}
