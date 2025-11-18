// HTTP file download with progress tracking.
// Downloads files from URLs with optional save-to-disk, reporting connection time,
// TTFB, total time, and bytes downloaded.

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

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
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("##-"),
    );

    let mut downloaded: u64 = 0;
    let mut ttfb: Option<f64> = None;
    let mut file: Option<File> = None;

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
