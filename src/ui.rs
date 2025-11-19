// Interactive menu and result display functions.
// Handles server selection menu, download status messages, and formatted output.

use colored::*;
use inquire::{Select, Text};
use bytesize::ByteSize;
use crate::servers::{ServerMetadata, LocalServerData};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

static CACHED_TITLE: OnceLock<String> = OnceLock::new();

fn get_title() -> &'static str {
    CACHED_TITLE.get_or_init(|| {
        playbill::generate_title("speedo", Some(env!("CARGO_PKG_VERSION")))
    })
}

pub enum ServerSelection {
    Server(ServerMetadata),
    #[allow(dead_code)]
    Custom(String, Option<String>),
    Quit,
}

pub enum BrowseMode {
    All,
    ByRegion,
    ByProvider,
    Search,
    Quit,
}

pub fn print_download_header(name: &str, save_path: &Option<String>) {
    println!();
    println!("{}", format!("Testing against {}...", name).yellow());
    if let Some(ref path) = save_path {
        println!("{}", format!("(Saving to: {})", path).bright_black());
    } else {
        println!("{}", "(Discarding Data)".bright_black());
    }
    println!("Please wait...");
    println!();
}

pub fn wait_for_continue() -> Result<(), Box<dyn std::error::Error>> {
    inquire::Text::new("")
        .with_placeholder("Press Enter to continue...")
        .prompt()?;
    Ok(())
}

fn get_browse_mode() -> Result<BrowseMode, Box<dyn std::error::Error>> {
    print!("\x1B[2J\x1B[1;1H");
    print!("{}", get_title());
    
    let options = vec![
        "🌍  Browse all servers",
        "🗺️  Browse by region",
        "🏢  Browse by provider",
        "🔍  Search servers",
        "📍  Quit",
    ];
    
    let selection = Select::new("How would you like to browse servers?", options)
        .prompt()?;
    
    match selection {
        "🌍  Browse all servers" => Ok(BrowseMode::All),
        "🗺️  Browse by region" => Ok(BrowseMode::ByRegion),
        "🏢  Browse by provider" => Ok(BrowseMode::ByProvider),
        "🔍  Search servers" => Ok(BrowseMode::Search),
        "📍  Quit" => Ok(BrowseMode::Quit),
        _ => Ok(BrowseMode::All),
    }
}

fn group_servers_by_region(servers: &[ServerMetadata]) -> HashMap<String, Vec<ServerMetadata>> {
    let mut map: HashMap<String, Vec<ServerMetadata>> = HashMap::new();
    
    for server in servers {
        let region = server.region.clone().unwrap_or_else(|| "Other".to_string());
        map.entry(region).or_insert_with(Vec::new).push(server.clone());
    }
    
    map
}

fn group_servers_by_provider(servers: &[ServerMetadata]) -> HashMap<String, Vec<ServerMetadata>> {
    let mut map: HashMap<String, Vec<ServerMetadata>> = HashMap::new();
    
    for server in servers {
        let provider = server.provider.clone().unwrap_or_else(|| "Other".to_string());
        map.entry(provider).or_insert_with(Vec::new).push(server.clone());
    }
    
    map
}

fn select_from_list(servers: &[ServerMetadata], health_data: &LocalServerData) -> Result<ServerSelection, Box<dyn std::error::Error>> {
    let mut display_names: Vec<String> = servers.iter().map(|s| {
        let health = health_data.health.get(&s.url);
        let speed_info = if let Some(h) = health {
            if h.avg_speed_mbps > 0.0 {
                format!(" ({:.1} MB/s avg)", h.avg_speed_mbps / 8.0)
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        format!("{} - {}{}", 
            s.name,
            s.location.as_ref().unwrap_or(&"Unknown".to_string()),
            speed_info
        )
    }).collect();
    
    display_names.push("← Back".to_string());
    
    let selection = Select::new("Select a server:", display_names)
        .with_page_size(20)
        .prompt()?;
    
    if selection == "← Back" {
        return show_menu();
    }
    
    // Find the server by matching the beginning of the display name
    let idx = servers.iter().position(|s| {
        selection.starts_with(&format!("{} - {}", s.name, s.location.as_ref().unwrap_or(&"Unknown".to_string())))
    }).unwrap_or(0);
    
    Ok(ServerSelection::Server(servers[idx].clone()))
}

fn browse_by_region(servers: &[ServerMetadata], health_data: &LocalServerData) -> Result<ServerSelection, Box<dyn std::error::Error>> {
    let grouped = group_servers_by_region(servers);
    
    let mut regions: Vec<String> = grouped.keys()
        .map(|r| {
            let count = grouped.get(r).map(|v| v.len()).unwrap_or(0);
            format!("{} ({} servers)", r, count)
        })
        .collect();
    regions.sort();
    regions.push("← Back".to_string());
    
    let selection = Select::new("Select a region:", regions)
        .prompt()?;
    
    if selection == "← Back" {
        return show_menu();
    }
    
    let region = selection.split(" (").next().unwrap_or("").to_string();
    let region_servers = grouped.get(&region).unwrap();
    
    select_from_list(region_servers, health_data)
}

fn browse_by_provider(servers: &[ServerMetadata], health_data: &LocalServerData) -> Result<ServerSelection, Box<dyn std::error::Error>> {
    let grouped = group_servers_by_provider(servers);
    
    let mut providers: Vec<String> = grouped.keys()
        .map(|p| {
            let count = grouped.get(p).map(|v| v.len()).unwrap_or(0);
            let regions: HashSet<String> = grouped.get(p).unwrap()
                .iter()
                .filter_map(|s| s.region.clone())
                .collect();
            
            if regions.is_empty() {
                format!("{} ({} servers)", p, count)
            } else if regions.len() == 1 {
                format!("{} ({} servers - {})", p, count, regions.iter().next().unwrap())
            } else {
                format!("{} ({} servers - {} regions)", p, count, regions.len())
            }
        })
        .collect();
    providers.sort();
    providers.push("← Back".to_string());
    
    let selection = Select::new("Select a provider:", providers)
        .prompt()?;
    
    if selection == "← Back" {
        return show_menu();
    }
    
    let provider = selection.split(" (").next().unwrap_or("").to_string();
    let provider_servers = grouped.get(&provider).unwrap();
    
    select_from_list(provider_servers, health_data)
}

fn browse_all(servers: &[ServerMetadata], health_data: &LocalServerData) -> Result<ServerSelection, Box<dyn std::error::Error>> {
    select_from_list(servers, health_data)
}

fn search_servers(servers: &[ServerMetadata], health_data: &LocalServerData) -> Result<ServerSelection, Box<dyn std::error::Error>> {
    let search_term = Text::new("Search servers:")
        .with_placeholder("Enter location, provider, or server name...")
        .prompt()?;
    
    let search_lower = search_term.to_lowercase();
    let filtered: Vec<ServerMetadata> = servers.iter()
        .filter(|s| {
            s.name.to_lowercase().contains(&search_lower) ||
            s.location.as_ref().map(|l| l.to_lowercase().contains(&search_lower)).unwrap_or(false) ||
            s.provider.as_ref().map(|p| p.to_lowercase().contains(&search_lower)).unwrap_or(false) ||
            s.region.as_ref().map(|r| r.to_lowercase().contains(&search_lower)).unwrap_or(false)
        })
        .cloned()
        .collect();
    
    if filtered.is_empty() {
        println!("{}", format!("No servers found matching '{}'", search_term).yellow());
        wait_for_continue()?;
        return show_menu();
    }
    
    println!("{}", format!("Found {} servers matching '{}'", filtered.len(), search_term).green());
    select_from_list(&filtered, health_data)
}

pub fn show_menu() -> Result<ServerSelection, Box<dyn std::error::Error>> {
    // Load server data
    let server_data = crate::servers::load_local_server_data();
    let servers = crate::servers::get_merged_server_list(&server_data);
    
    // Get browse mode
    let mode = get_browse_mode()?;
    
    match mode {
        BrowseMode::All => browse_all(&servers, &server_data),
        BrowseMode::ByRegion => browse_by_region(&servers, &server_data),
        BrowseMode::ByProvider => browse_by_provider(&servers, &server_data),
        BrowseMode::Search => search_servers(&servers, &server_data),
        BrowseMode::Quit => Ok(ServerSelection::Quit),
    }
}

pub fn print_results(
    status_code: u16,
    connect_time: f64,
    ttfb: f64,
    total_time: f64,
    bytes_downloaded: u64,
    save_path: Option<String>,
) {
    let size_mb = bytes_downloaded as f64 / 1_048_576.0;
    let mbs = (bytes_downloaded as f64 / total_time) / 1_048_576.0;
    let mbps = (bytes_downloaded as f64 * 8.0 / total_time) / 1_000_000.0;
    
    let size_str = ByteSize::b(bytes_downloaded).to_string_as(true);
    
    let time_str = if total_time >= 60.0 {
        format!("{:.0}m {:.1}s", total_time / 60.0, total_time % 60.0)
    } else {
        format!("{:.2}s", total_time)
    };
    
    println!("{} {} in {}", "Downloaded".green(), size_str, time_str);

    println!();
    if status_code == 200 {
        println!("Status:  {}", format!("{} (OK)", status_code).green());
    } else {
        println!(
            "Status:  {}",
            format!("{} (Error/Redirect)", status_code).red()
        );
    }

    println!("Connect: {:.3}s", connect_time);
    println!("TTFB:    {:.3}s", ttfb);
    println!("Total:   {:.3}s", total_time);
    println!("----------------");
    println!("Size:    {:.2} MB", size_mb);

    if size_mb < 10.0 {
        println!(
            "{}",
            "WARNING: File is very small (<10MB). Speed result may be inaccurate.".magenta()
        );
    }
    println!("----------------");

    if status_code == 200 {
        println!(
            "Speed:   {}",
            format!("{:.2} MB/s  ({:.2} Mbps)", mbs, mbps).green()
        );
        if let Some(path) = save_path {
            println!();
            println!("{}", format!("File saved successfully: {}", path).cyan());
        }
    } else {
        println!(
            "Speed:   {}",
            format!("{:.2} MB/s  ({:.2} Mbps) - (Invalid due to Error)", mbs, mbps)
                .bright_black()
        );
    }
}

pub fn print_speed_only(
    status_code: u16,
    total_time: f64,
    bytes_downloaded: u64,
) {
    let mbs = (bytes_downloaded as f64 / total_time) / 1_048_576.0;
    let mbps = (bytes_downloaded as f64 * 8.0 / total_time) / 1_000_000.0;
    
    let size_str = ByteSize::b(bytes_downloaded).to_string_as(true);
    
    let time_str = if total_time >= 60.0 {
        format!("{:.0}m {:.1}s", total_time / 60.0, total_time % 60.0)
    } else {
        format!("{:.2}s", total_time)
    };
    
    print!("{} {} in {} - ", "Downloaded".green(), size_str, time_str);

    if status_code == 200 {
        println!("{:.2} MB/s  ({:.2} Mbps)", mbs, mbps);
    } else {
        println!("{:.2} MB/s  ({:.2} Mbps) - (Error: status {})", mbs, mbps, status_code);
    }
}
