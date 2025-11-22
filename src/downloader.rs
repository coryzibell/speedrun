// HTTP file download with progress tracking.
// Downloads files from URLs with optional save-to-disk, reporting connection time,
// TTFB, total time, and bytes downloaded.

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::Serialize;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::config::SpeedUnit;
use bytesize::ByteSize;

fn format_speed(bytes_per_sec: f64, unit: SpeedUnit) -> String {
    match unit {
        SpeedUnit::BitsMetric => {
            let bits_per_sec = bytes_per_sec * 8.0;
            if bits_per_sec >= 1_000_000_000.0 {
                format!("{:.2} Gbps", bits_per_sec / 1_000_000_000.0)
            } else if bits_per_sec >= 1_000_000.0 {
                format!("{:.2} Mbps", bits_per_sec / 1_000_000.0)
            } else if bits_per_sec >= 1_000.0 {
                format!("{:.2} Kbps", bits_per_sec / 1_000.0)
            } else {
                format!("{:.2} bps", bits_per_sec)
            }
        }
        SpeedUnit::BitsBinary => {
            let bits_per_sec = bytes_per_sec * 8.0;
            if bits_per_sec >= 1_073_741_824.0 {
                format!("{:.2} Gibps", bits_per_sec / 1_073_741_824.0)
            } else if bits_per_sec >= 1_048_576.0 {
                format!("{:.2} Mibps", bits_per_sec / 1_048_576.0)
            } else if bits_per_sec >= 1_024.0 {
                format!("{:.2} Kibps", bits_per_sec / 1_024.0)
            } else {
                format!("{:.2} bps", bits_per_sec)
            }
        }
        SpeedUnit::BytesMetric => {
            format!("{}/s", ByteSize::b(bytes_per_sec as u64).display().si())
        }
        SpeedUnit::BytesBinary => {
            format!("{}/s", ByteSize::b(bytes_per_sec as u64))
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadResult {
    pub status_code: u16,
    pub connect_time: f64,
    pub ttfb: f64,
    pub total_time: f64,
    pub bytes_downloaded: u64,
}

pub async fn download_file(
    url: &str,
    save_path: Option<&str>,
    user_agent: &str,
    speed_unit: SpeedUnit,
) -> Result<DownloadResult, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent(user_agent)
        .build()?;

    let start = Instant::now();
    
    let response = client.get(url).send().await?;
    let connect_time = start.elapsed().as_secs_f64();
    
    let status_code = response.status().as_u16();
    let total_size = response.content_length().unwrap_or(0);
    
    let ttfb_start = Instant::now();
    let mut stream = response.bytes_stream();
    
    let pb = ProgressBar::new(total_size);
    
    // Use different template based on whether we know the file size
    let template = if total_size > 0 {
        "{bar:40.cyan/blue} {bytes}/{total_bytes} {msg} ({eta})"
    } else {
        "{spinner:.cyan} {bytes} {msg}"
    };
    
    pb.set_style(
        ProgressStyle::default_bar()
            .template(template)
            .unwrap()
            .progress_chars("##-"),
    );

    let mut downloaded: u64 = 0;
    let mut ttfb: Option<f64> = None;
    let mut file: Option<File> = None;
    let mut last_update = Instant::now();
    let mut last_downloaded = 0u64;

    if let Some(path) = save_path {
        file = Some(File::create(path).await?);
    }

    use futures_util::StreamExt;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        
        if ttfb.is_none() {
            ttfb = Some(ttfb_start.elapsed().as_secs_f64());
        }
        
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
        
        // Update speed message every 100ms
        let now = Instant::now();
        if now.duration_since(last_update).as_millis() >= 100 {
            let elapsed = now.duration_since(last_update).as_secs_f64();
            let bytes_diff = downloaded - last_downloaded;
            let speed = bytes_diff as f64 / elapsed;
            pb.set_message(format_speed(speed, speed_unit));
            last_update = now;
            last_downloaded = downloaded;
        }
        
        if let Some(ref mut f) = file {
            f.write_all(&chunk).await?;
        }
    }

    pb.finish_and_clear();

    let total_time = start.elapsed().as_secs_f64();

    Ok(DownloadResult {
        status_code,
        connect_time,
        ttfb: ttfb.unwrap_or(connect_time),
        total_time,
        bytes_downloaded: downloaded,
    })
}

pub fn extract_filename(url: &str) -> String {
    url.split('?')
        .next()
        .and_then(|s| s.split('/').last())
        .filter(|s| !s.is_empty())
        .unwrap_or("speedtest_file.dat")
        .to_string()
}
