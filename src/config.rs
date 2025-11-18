// Configuration file loading and management.
// Handles reading TOML config from ./speedrun.toml, ./.speedrun.toml, or ~/.speedrun.toml.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    #[serde(default)]
    pub custom_servers: Vec<CustomServer>,
    #[serde(default)]
    pub interactive: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CustomServer {
    pub name: String,
    pub url: String,
}

fn default_user_agent() -> String {
    "Mozilla/5.0".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_agent: default_user_agent(),
            custom_servers: Vec::new(),
            interactive: false,
        }
    }
}

fn get_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("speedrun.toml"));
        paths.push(cwd.join(".speedrun.toml"));
    }
    
    if let Some(home_dir) = dirs::home_dir() {
        paths.push(home_dir.join(".speedrun.toml"));
    }
    
    paths
}

pub fn load_config() -> Config {
    use colored::*;
    
    for path in get_config_paths() {
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<Config>(&contents) {
                    println!("{}", format!("Loaded config from: {}", path.display()).bright_black());
                    return config;
                }
            }
        }
    }
    Config::default()
}
