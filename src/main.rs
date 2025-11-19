// Application entry point and command-line argument handling.
// Routes execution to interactive mode, non-interactive mode, or URL download.

mod config;
mod downloader;
mod output;
mod servers;
mod ui;

use clap::Parser;
use config::{load_config, SpeedUnit};
use downloader::download_file;
use output::OutputFormat;
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
    
    /// Speed unit format: bits-metric, bits-binary, bytes-metric, bytes-binary
    #[arg(short, long, value_name = "UNIT")]
    speed_unit: Option<String>,
    
    /// Output format: json, csv, or human (default)
    #[arg(long, value_name = "FORMAT")]
    format: Option<String>,
    
    /// Use compact JSON output (no pretty printing)
    #[arg(long)]
    compact: bool,
    
    /// Output JSON format (shorthand for --format json)
    #[arg(long)]
    json: bool,
    
    /// Update remote server list
    #[arg(long)]
    update_servers: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = load_config();
    
    // Handle --update-servers command
    if args.update_servers {
        return update_server_list().await;
    }
    
    // Auto-update server list if cache is stale
    let mut server_data = servers::load_local_server_data();
    if servers::should_update_cache(&server_data) {
        if let Ok(remote_list) = servers::fetch_remote_server_list().await {
            server_data.remote_list = Some(remote_list);
            server_data.cache_timestamp = chrono::Utc::now();
            servers::save_local_server_data(&server_data).ok();
        }
    }
    
    // Determine speed unit: CLI flag overrides config
    let speed_unit_str = args.speed_unit.as_ref().unwrap_or(&config.speed_unit);
    let speed_unit = SpeedUnit::from_string(speed_unit_str);
    
    // Determine output format
    let output_format = if args.json {
        if args.compact {
            OutputFormat::JsonCompact
        } else {
            OutputFormat::Json
        }
    } else if let Some(ref format_str) = args.format {
        OutputFormat::from_string(format_str)
    } else {
        OutputFormat::Human
    };
    
    // If URL is provided, download it and save to current directory
    if let Some(url) = args.url {
        let filename = downloader::extract_filename(&url);
        let result = download_file(&url, Some(&filename), &config.user_agent, speed_unit).await?;
        
        match output_format {
            OutputFormat::Json => {
                output::print_json(&result, "Custom URL", &url, false)?;
            }
            OutputFormat::JsonCompact => {
                output::print_json(&result, "Custom URL", &url, true)?;
            }
            OutputFormat::Csv => {
                output::print_csv(&result, "Custom URL", &url, true);
            }
            OutputFormat::Human => {
                ui::print_speed_only(
                    result.status_code,
                    result.total_time,
                    result.bytes_downloaded,
                );
                
                if result.status_code == 200 {
                    println!("Saved: {}", filename);
                }
            }
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
        run_interactive_mode(&config, speed_unit, output_format).await?;
    } else {
        // Non-interactive mode - run default server once
        run_default_test(&config, speed_unit, output_format).await?;
    }

    Ok(())
}

async fn run_default_test(config: &crate::config::Config, speed_unit: SpeedUnit, output_format: OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    let server = &SERVERS[0];
    let result = download_file(server.url, None, &config.user_agent, speed_unit).await?;
    
    match output_format {
        OutputFormat::Json => {
            output::print_json(&result, server.name, server.url, false)?;
        }
        OutputFormat::JsonCompact => {
            output::print_json(&result, server.name, server.url, true)?;
        }
        OutputFormat::Csv => {
            output::print_csv(&result, server.name, server.url, true);
        }
        OutputFormat::Human => {
            print_speed_only(
                result.status_code,
                result.total_time,
                result.bytes_downloaded,
            );
        }
    }

    Ok(())
}

async fn run_interactive_mode(config: &crate::config::Config, speed_unit: SpeedUnit, output_format: OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
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

        let result = download_file(&url, save_path.as_deref(), &config.user_agent, speed_unit).await?;

        match output_format {
            OutputFormat::Json => {
                output::print_json(&result, &name, &url, false)?;
            }
            OutputFormat::JsonCompact => {
                output::print_json(&result, &name, &url, true)?;
            }
            OutputFormat::Csv => {
                output::print_csv(&result, &name, &url, true);
            }
            OutputFormat::Human => {
                print_results(
                    result.status_code,
                    result.connect_time,
                    result.ttfb,
                    result.total_time,
                    result.bytes_downloaded,
                    save_path,
                );
            }
        }

        println!();
        wait_for_continue().ok();
    }

    Ok(())
}

async fn update_server_list() -> Result<(), Box<dyn std::error::Error>> {
    use colored::*;
    
    println!("{}", "Fetching remote server list...".yellow());
    
    match servers::fetch_remote_server_list().await {
        Ok(remote_list) => {
            let count = remote_list.servers.len();
            println!("{}", format!("✓ Downloaded {} servers (version {})", count, remote_list.version).green());
            
            let mut server_data = servers::load_local_server_data();
            server_data.remote_list = Some(remote_list);
            server_data.cache_timestamp = chrono::Utc::now();
            
            if let Err(e) = servers::save_local_server_data(&server_data) {
                println!("{}", format!("Warning: Failed to save server list: {}", e).red());
            } else {
                println!("{}", "✓ Server list cached successfully".green());
            }
            
            Ok(())
        }
        Err(e) => {
            println!("{}", format!("✗ Failed to fetch server list: {}", e).red());
            println!("{}", "Using embedded fallback servers".yellow());
            Err(e)
        }
    }
}
