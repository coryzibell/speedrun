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

pub enum ServerOption {
    Server(ServerMetadata, Option<String>, Color), // server, health info, and color
    Back,
}

// Define a palette of visually distinct colors
const COLOR_PALETTE: &[Color] = &[
    Color::BrightCyan,
    Color::BrightBlue,
    Color::BrightGreen,
    Color::BrightMagenta,
    Color::BrightYellow,
    Color::BrightRed,
    Color::Cyan,
    Color::Blue,
    Color::Green,
    Color::Magenta,
    Color::Yellow,
    Color::Red,
];

fn build_provider_color_map(servers: &[ServerMetadata]) -> HashMap<String, Color> {
    // Extract and sort all unique providers
    let mut providers: Vec<String> = servers
        .iter()
        .filter_map(|s| s.provider.clone())
        .collect();
    providers.sort();
    providers.dedup();
    
    // Build map from provider to color
    providers
        .into_iter()
        .enumerate()
        .map(|(i, provider)| (provider, COLOR_PALETTE[i % COLOR_PALETTE.len()]))
        .collect()
}

fn get_provider_color(provider: &Option<String>, color_map: &HashMap<String, Color>) -> Color {
    provider
        .as_ref()
        .and_then(|p| color_map.get(p))
        .copied()
        .unwrap_or(Color::White)
}

impl std::fmt::Display for ServerOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerOption::Server(server, health_info, color) => {
                let base = format!("{} - {}", 
                    server.name,
                    server.location.as_ref().unwrap_or(&"Unknown".to_string())
                );
                let colored_base = base.color(*color);
                
                if let Some(health) = health_info {
                    write!(f, "{}{}", colored_base, health)
                } else {
                    write!(f, "{}", colored_base)
                }
            }
            ServerOption::Back => write!(f, "← Back"),
        }
    }
}

pub enum RegionOption {
    Region(String, usize), // region name and server count
    Back,
}

impl std::fmt::Display for RegionOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegionOption::Region(name, count) => write!(f, "{} ({} servers)", name, count),
            RegionOption::Back => write!(f, "← Back"),
        }
    }
}

pub enum ProviderOption {
    Provider(String, String), // provider name and display info
    Back,
}

impl std::fmt::Display for ProviderOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderOption::Provider(_, display) => write!(f, "{}", display),
            ProviderOption::Back => write!(f, "← Back"),
        }
    }
}

pub enum MenuOption {
    GlobalServer(ServerMetadata),
    BrowseAll(usize), // carries server count
    BrowseByRegion,
    BrowseByProvider,
    Search,
    Quit,
}

impl std::fmt::Display for MenuOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuOption::GlobalServer(server) => {
                write!(f, "🌐  {} - {}", 
                    server.name,
                    server.location.as_ref().unwrap_or(&"Global CDN".to_string())
                )
            }
            MenuOption::BrowseAll(count) => write!(f, "🌍  Browse all servers ({} servers)", count),
            MenuOption::BrowseByRegion => write!(f, "🗺️  Browse by region"),
            MenuOption::BrowseByProvider => write!(f, "🏢  Browse by provider"),
            MenuOption::Search => write!(f, "🔍  Search servers"),
            MenuOption::Quit => write!(f, "📍  Quit"),
        }
    }
}

pub enum MenuSelection {
    Server(ServerMetadata),
    BrowseAll,
    BrowseByRegion,
    BrowseByProvider,
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

fn get_main_menu_selection(servers: &[ServerMetadata]) -> Result<MenuSelection, Box<dyn std::error::Error>> {
    print!("\x1B[2J\x1B[1;1H");
    print!("{}", get_title());
    
    // Separate global servers from others
    let global_servers: Vec<ServerMetadata> = servers.iter()
        .filter(|s| s.region.as_ref().map(|r| r == "Global").unwrap_or(false))
        .cloned()
        .collect();
    
    let mut options: Vec<MenuOption> = Vec::new();
    
    // Add global servers first
    for server in global_servers {
        options.push(MenuOption::GlobalServer(server));
    }
    
    // Add browsing options
    options.push(MenuOption::BrowseAll(servers.len()));
    options.push(MenuOption::BrowseByRegion);
    options.push(MenuOption::BrowseByProvider);
    options.push(MenuOption::Search);
    options.push(MenuOption::Quit);
    
    let selection = Select::new("Select a server or browse:", options)
        .prompt()?;
    
    // Convert MenuOption to MenuSelection
    match selection {
        MenuOption::GlobalServer(server) => Ok(MenuSelection::Server(server)),
        MenuOption::BrowseAll(_) => Ok(MenuSelection::BrowseAll),
        MenuOption::BrowseByRegion => Ok(MenuSelection::BrowseByRegion),
        MenuOption::BrowseByProvider => Ok(MenuSelection::BrowseByProvider),
        MenuOption::Search => Ok(MenuSelection::Search),
        MenuOption::Quit => Ok(MenuSelection::Quit),
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
    // Build color map once for all servers
    let color_map = build_provider_color_map(servers);
    
    let mut options: Vec<ServerOption> = servers.iter().map(|s| {
        let health = health_data.health.get(&s.url);
        let speed_info = if let Some(h) = health {
            if h.avg_speed_mbps > 0.0 {
                Some(format!(" ({:.1} MB/s avg)", h.avg_speed_mbps / 8.0))
            } else {
                None
            }
        } else {
            None
        };
        
        let color = get_provider_color(&s.provider, &color_map);
        ServerOption::Server(s.clone(), speed_info, color)
    }).collect();
    
    options.push(ServerOption::Back);
    
    let selection = Select::new("Select a server:", options)
        .with_page_size(20)
        .prompt()?;
    
    match selection {
        ServerOption::Server(server, _, _) => Ok(ServerSelection::Server(server)),
        ServerOption::Back => show_menu(),
    }
}

fn browse_by_region(servers: &[ServerMetadata], health_data: &LocalServerData) -> Result<ServerSelection, Box<dyn std::error::Error>> {
    let grouped = group_servers_by_region(servers);
    
    let mut options: Vec<RegionOption> = grouped.iter()
        .map(|(region, servers)| RegionOption::Region(region.clone(), servers.len()))
        .collect();
    options.sort_by(|a, b| {
        match (a, b) {
            (RegionOption::Region(name_a, _), RegionOption::Region(name_b, _)) => name_a.cmp(name_b),
            _ => std::cmp::Ordering::Equal,
        }
    });
    options.push(RegionOption::Back);
    
    let selection = Select::new("Select a region:", options)
        .prompt()?;
    
    match selection {
        RegionOption::Region(region, _) => {
            let region_servers = grouped.get(&region).unwrap();
            select_from_list(region_servers, health_data)
        }
        RegionOption::Back => show_menu(),
    }
}

fn browse_by_provider(servers: &[ServerMetadata], health_data: &LocalServerData) -> Result<ServerSelection, Box<dyn std::error::Error>> {
    let grouped = group_servers_by_provider(servers);
    
    let mut options: Vec<ProviderOption> = grouped.iter()
        .map(|(provider, servers)| {
            let count = servers.len();
            let regions: HashSet<String> = servers.iter()
                .filter_map(|s| s.region.clone())
                .collect();
            
            let display = if regions.is_empty() {
                format!("{} ({} servers)", provider, count)
            } else if regions.len() == 1 {
                format!("{} ({} servers - {})", provider, count, regions.iter().next().unwrap())
            } else {
                format!("{} ({} servers - {} regions)", provider, count, regions.len())
            };
            
            ProviderOption::Provider(provider.clone(), display)
        })
        .collect();
    options.sort_by(|a, b| {
        match (a, b) {
            (ProviderOption::Provider(name_a, _), ProviderOption::Provider(name_b, _)) => name_a.cmp(name_b),
            _ => std::cmp::Ordering::Equal,
        }
    });
    options.push(ProviderOption::Back);
    
    let selection = Select::new("Select a provider:", options)
        .prompt()?;
    
    match selection {
        ProviderOption::Provider(provider, _) => {
            let provider_servers = grouped.get(&provider).unwrap();
            select_from_list(provider_servers, health_data)
        }
        ProviderOption::Back => show_menu(),
    }
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
    
    // Get main menu selection
    let selection = get_main_menu_selection(&servers)?;
    
    match selection {
        MenuSelection::Server(server) => Ok(ServerSelection::Server(server)),
        MenuSelection::BrowseAll => browse_all(&servers, &server_data),
        MenuSelection::BrowseByRegion => browse_by_region(&servers, &server_data),
        MenuSelection::BrowseByProvider => browse_by_provider(&servers, &server_data),
        MenuSelection::Search => search_servers(&servers, &server_data),
        MenuSelection::Quit => Ok(ServerSelection::Quit),
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
