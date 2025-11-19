// Pre-configured speed test server definitions with metadata.
// Contains test file URLs from Cloudflare, Tele2, Hetzner, and Vultr.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetadata {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
    pub lon: Option<f64>,
    #[serde(default)]
    pub file_size: Option<u64>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealth {
    pub url: String,
    #[serde(default)]
    pub last_checked: Option<DateTime<Utc>>,
    #[serde(default)]
    pub success_rate: f64,
    #[serde(default)]
    pub avg_speed_mbps: f64,
    #[serde(default)]
    pub avg_latency_ms: f64,
    #[serde(default)]
    pub failures: u32,
    #[serde(default)]
    pub total_checks: u32,
    #[serde(default)]
    pub user_rating: Option<i32>, // -1 to 5 stars, None = not rated
    #[serde(default)]
    pub user_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerList {
    pub version: String,
    pub updated: DateTime<Utc>,
    pub servers: Vec<ServerMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalServerData {
    pub health: std::collections::HashMap<String, ServerHealth>,
    pub cache_timestamp: DateTime<Utc>,
    pub remote_list: Option<ServerList>,
}

impl Default for LocalServerData {
    fn default() -> Self {
        Self {
            health: std::collections::HashMap::new(),
            cache_timestamp: Utc::now(),
            remote_list: None,
        }
    }
}

pub struct TestServer {
    pub name: &'static str,
    pub url: &'static str,
}

// Embedded fallback servers (used if remote fetch fails)
pub const SERVERS: &[TestServer] = &[
    TestServer {
        name: "Cloudflare (CDN)",
        url: "https://speed.cloudflare.com/__down?bytes=100000000",
    },
    TestServer {
        name: "Tele2 (Global)",
        url: "http://speedtest.tele2.net/100MB.zip",
    },
    TestServer {
        name: "Hetzner (Nuremberg)",
        url: "https://nbg1-speed.hetzner.com/100MB.bin",
    },
    TestServer {
        name: "Hetzner (Falkenstein)",
        url: "https://fsn1-speed.hetzner.com/100MB.bin",
    },
    TestServer {
        name: "Hetzner (Helsinki)",
        url: "https://hel1-speed.hetzner.com/100MB.bin",
    },
    TestServer {
        name: "Hetzner (Ashburn VA)",
        url: "https://ash-speed.hetzner.com/100MB.bin",
    },
    TestServer {
        name: "Hetzner (Hillsboro OR)",
        url: "https://hil-speed.hetzner.com/100MB.bin",
    },
    TestServer {
        name: "Hetzner (Singapore)",
        url: "https://sin-speed.hetzner.com/100MB.bin",
    },
    TestServer {
        name: "Vultr (New Jersey)",
        url: "https://nj-us-ping.vultr.com/vultr.com.100MB.bin",
    },
    TestServer {
        name: "Vultr (Silicon Valley)",
        url: "https://sjo-ca-us-ping.vultr.com/vultr.com.100MB.bin",
    },
    TestServer {
        name: "Vultr (Singapore)",
        url: "https://sgp-ping.vultr.com/vultr.com.100MB.bin",
    },
];

const REMOTE_SERVER_LIST_URL: &str = "https://raw.githubusercontent.com/coryzibell/speedo/main/servers.json";
const CACHE_EXPIRY_DAYS: i64 = 7;

fn get_server_data_path() -> PathBuf {
    if let Some(data_dir) = dirs::data_local_dir() {
        data_dir.join("speedo").join("servers.json")
    } else {
        PathBuf::from(".speedo_servers.json")
    }
}

pub fn load_local_server_data() -> LocalServerData {
    let path = get_server_data_path();
    if path.exists() {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(data) = serde_json::from_str::<LocalServerData>(&contents) {
                return data;
            }
        }
    }
    LocalServerData::default()
}

pub fn save_local_server_data(data: &LocalServerData) -> std::io::Result<()> {
    let path = get_server_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub async fn fetch_remote_server_list() -> Result<ServerList, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("speedo")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    
    let response = client.get(REMOTE_SERVER_LIST_URL).send().await?;
    let list = response.json::<ServerList>().await?;
    Ok(list)
}

pub fn should_update_cache(data: &LocalServerData) -> bool {
    let now = Utc::now();
    let elapsed = now.signed_duration_since(data.cache_timestamp);
    elapsed.num_days() >= CACHE_EXPIRY_DAYS
}

pub fn get_merged_server_list(data: &LocalServerData) -> Vec<ServerMetadata> {
    let mut servers = Vec::new();
    
    // Start with remote servers if available
    if let Some(ref remote_list) = data.remote_list {
        servers.extend(remote_list.servers.clone());
    } else {
        // Fallback to embedded servers
        for server in SERVERS {
            servers.push(ServerMetadata {
                name: server.name.to_string(),
                url: server.url.to_string(),
                provider: None,
                location: None,
                region: None,
                lat: None,
                lon: None,
                file_size: Some(100_000_000),
                enabled: true,
            });
        }
    }
    
    // Filter out disabled servers and apply health data
    servers.into_iter()
        .filter(|s| s.enabled)
        .collect()
}
