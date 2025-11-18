// Interactive menu and result display functions.
// Handles server selection menu, download status messages, and formatted output.

use colored::*;
use inquire::{Select, Text, Confirm};

pub enum ServerSelection {
    Predefined(usize),
    Custom(String, Option<String>),
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

pub fn show_menu() -> Result<ServerSelection, Box<dyn std::error::Error>> {
    print!("\x1B[2J\x1B[1;1H");
    crate::title::print_title();

    let options = vec![
        "Cloudflare CDN - https://speed.cloudflare.com/__down?bytes=100000000",
        "Tele2 Global - http://speedtest.tele2.net/100MB.zip",
        "─────────────────────────────────────────────────────────────────────",
        "Hetzner Nuremberg - https://nbg1-speed.hetzner.com/100MB.bin",
        "Hetzner Falkenstein - https://fsn1-speed.hetzner.com/100MB.bin",
        "Hetzner Helsinki - https://hel1-speed.hetzner.com/100MB.bin",
        "Hetzner Ashburn - https://ash-speed.hetzner.com/100MB.bin",
        "Hetzner Hillsboro - https://hil-speed.hetzner.com/100MB.bin",
        "Hetzner Singapore - https://sin-speed.hetzner.com/100MB.bin",
        "─────────────────────────────────────────────────────────────────────",
        "Vultr New Jersey - https://nj-us-ping.vultr.com/vultr.com.100MB.bin",
        "Vultr Silicon Valley - https://sjo-ca-us-ping.vultr.com/vultr.com.100MB.bin",
        "Vultr Singapore - https://sgp-ping.vultr.com/vultr.com.100MB.bin",
        "─────────────────────────────────────────────────────────────────────",
        "Custom URL",
        "Quit",
    ];

    let selection = Select::new("Select a file:", options)
        .with_page_size(20)
        .prompt()?;

    match selection {
        "Cloudflare CDN - https://speed.cloudflare.com/__down?bytes=100000000" => Ok(ServerSelection::Predefined(0)),
        "Tele2 Global - http://speedtest.tele2.net/100MB.zip" => Ok(ServerSelection::Predefined(1)),
        "Hetzner Nuremberg - https://nbg1-speed.hetzner.com/100MB.bin" => Ok(ServerSelection::Predefined(2)),
        "Hetzner Falkenstein - https://fsn1-speed.hetzner.com/100MB.bin" => Ok(ServerSelection::Predefined(3)),
        "Hetzner Helsinki - https://hel1-speed.hetzner.com/100MB.bin" => Ok(ServerSelection::Predefined(4)),
        "Hetzner Ashburn - https://ash-speed.hetzner.com/100MB.bin" => Ok(ServerSelection::Predefined(5)),
        "Hetzner Hillsboro - https://hil-speed.hetzner.com/100MB.bin" => Ok(ServerSelection::Predefined(6)),
        "Hetzner Singapore - https://sin-speed.hetzner.com/100MB.bin" => Ok(ServerSelection::Predefined(7)),
        "Vultr New Jersey - https://nj-us-ping.vultr.com/vultr.com.100MB.bin" => Ok(ServerSelection::Predefined(8)),
        "Vultr Silicon Valley - https://sjo-ca-us-ping.vultr.com/vultr.com.100MB.bin" => Ok(ServerSelection::Predefined(9)),
        "Vultr Singapore - https://sgp-ping.vultr.com/vultr.com.100MB.bin" => Ok(ServerSelection::Predefined(10)),
        "Custom URL" => {
            let url = Text::new("Enter URL:")
                .with_help_message("URL to download from (http/https)")
                .prompt()?;

            let mut url = url.trim().to_string();
            if !url.starts_with("http") {
                url = format!("http://{}", url);
            }

            let want_save = Confirm::new("Save file to current folder?")
                .with_default(false)
                .prompt()?;

            let save_path = if want_save {
                let suggested = crate::downloader::extract_filename(&url);
                let filename = Text::new("Enter filename:")
                    .with_default(&suggested)
                    .prompt()?;
                Some(filename)
            } else {
                None
            };

            Ok(ServerSelection::Custom(url, save_path))
        }
        "Quit" => Ok(ServerSelection::Quit),
        _ => Ok(ServerSelection::Predefined(0)),
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

    if status_code == 200 {
        println!("{:.2} MB/s  ({:.2} Mbps)", mbs, mbps);
    } else {
        println!("{:.2} MB/s  ({:.2} Mbps) - (Error: status {})", mbs, mbps, status_code);
    }
}
