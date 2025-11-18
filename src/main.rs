// Application entry point and command-line argument handling.
// Routes execution to interactive mode, non-interactive mode, or URL download.

mod config;
mod downloader;
mod servers;
mod ui;
mod fonts;
mod title;

use clap::Parser;
use config::load_config;
use downloader::download_file;
use servers::SERVERS;
use ui::{show_menu, print_results, print_speed_only, print_download_header, wait_for_continue, ServerSelection};

#[derive(Parser)]
#[command(version, about = "A fast network speed test tool", long_about = None)]
struct Args {
    /// URL to download (saves file to current directory)
    #[arg(value_name = "URL")]
    url: Option<String>,
    
    /// Run in interactive mode (show menu)
    #[arg(short, long)]
    interactive: bool,
    
    /// Run in non-interactive mode (quick test)
    #[arg(short, long)]
    non_interactive: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = load_config();
    
    // If URL is provided, download it and save to current directory
    if let Some(url) = args.url {
        let filename = downloader::extract_filename(&url);
        let result = download_file(&url, Some(&filename), &config.user_agent).await?;
        
        ui::print_speed_only(
            result.status_code,
            result.total_time,
            result.bytes_downloaded,
        );
        
        if result.status_code == 200 {
            println!("Saved: {}", filename);
        }
        
        return Ok(());
    }
    
    // Determine mode: CLI flags override config
    let interactive_mode = if args.non_interactive {
        false
    } else if args.interactive {
        true
    } else {
        config.interactive
    };
    
    if interactive_mode {
        // Interactive mode - show menu and loop
        run_interactive_mode(&config).await?;
    } else {
        // Non-interactive mode - run default server once
        run_default_test(&config).await?;
    }

    Ok(())
}

async fn run_default_test(config: &crate::config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let server = &SERVERS[0];
    let result = download_file(server.url, None, &config.user_agent).await?;
    
    print_speed_only(
        result.status_code,
        result.total_time,
        result.bytes_downloaded,
    );

    Ok(())
}

async fn run_interactive_mode(config: &crate::config::Config) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let selection = match show_menu() {
            Ok(sel) => sel,
            Err(_) => {
                println!("\nExiting...");
                break;
            }
        };

        let (url, name, save_path): (String, String, Option<String>) = match selection {
            ServerSelection::Predefined(index) => {
                (
                    SERVERS[index].url.to_string(),
                    SERVERS[index].name.to_string(),
                    None,
                )
            }
            ServerSelection::Custom(url, save_path) => {
                (url, "Custom URL".to_string(), save_path)
            }
            ServerSelection::Quit => {
                println!("Exiting...");
                break;
            }
        };

        print_download_header(&name, &save_path);

        let result = download_file(&url, save_path.as_deref(), &config.user_agent).await?;

        print_results(
            result.status_code,
            result.connect_time,
            result.ttfb,
            result.total_time,
            result.bytes_downloaded,
            save_path,
        );

        println!();
        wait_for_continue().ok();
    }

    Ok(())
}
