// Machine-readable output formats (JSON, CSV).

use chrono::Utc;
use serde::Serialize;
use crate::downloader::DownloadResult;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Human,
    Json,
    JsonCompact,
    Csv,
}

impl OutputFormat {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "json-compact" | "compact" => OutputFormat::JsonCompact,
            "csv" => OutputFormat::Csv,
            _ => OutputFormat::Human,
        }
    }
}

#[derive(Serialize)]
struct SpeedInfo {
    mbps: f64,
    mb_s: f64,
}

#[derive(Serialize)]
struct ServerInfo {
    name: String,
    url: String,
}

#[derive(Serialize)]
struct JsonOutput {
    timestamp: String,
    server: ServerInfo,
    results: JsonResults,
}

#[derive(Serialize)]
struct JsonResults {
    status_code: u16,
    bytes_downloaded: u64,
    total_time: f64,
    connect_time: f64,
    ttfb: f64,
    speed: SpeedInfo,
}

pub fn print_json(
    result: &DownloadResult,
    server_name: &str,
    server_url: &str,
    compact: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mbps = (result.bytes_downloaded as f64 * 8.0 / result.total_time) / 1_000_000.0;
    let mb_s = (result.bytes_downloaded as f64 / result.total_time) / 1_000_000.0;

    let output = JsonOutput {
        timestamp: Utc::now().to_rfc3339(),
        server: ServerInfo {
            name: server_name.to_string(),
            url: server_url.to_string(),
        },
        results: JsonResults {
            status_code: result.status_code,
            bytes_downloaded: result.bytes_downloaded,
            total_time: result.total_time,
            connect_time: result.connect_time,
            ttfb: result.ttfb,
            speed: SpeedInfo { mbps, mb_s },
        },
    };

    if compact {
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&output)?);
    }

    Ok(())
}

pub fn print_csv(
    result: &DownloadResult,
    server_name: &str,
    server_url: &str,
    include_header: bool,
) {
    let mbps = (result.bytes_downloaded as f64 * 8.0 / result.total_time) / 1_000_000.0;
    let timestamp = Utc::now().to_rfc3339();

    if include_header {
        println!("timestamp,server_name,server_url,bytes_downloaded,total_time,connect_time,ttfb,speed_mbps,status_code");
    }

    println!(
        "{},{},{},{},{:.3},{:.3},{:.3},{:.2},{}",
        timestamp,
        escape_csv(server_name),
        escape_csv(server_url),
        result.bytes_downloaded,
        result.total_time,
        result.connect_time,
        result.ttfb,
        mbps,
        result.status_code
    );
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
