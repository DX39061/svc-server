use crate::config::server::ServerConfig;
use serde::Deserialize;
use std::fs;

mod server;

#[derive(Deserialize)]
pub struct Config {
    pub server_config: ServerConfig,
}
const CONFIG_PATH: [&str; 3] = [
    "/etc/svc/config.toml",
    "~/.config/svc/config.toml",
    "./config.toml",
];

impl Config {
    pub fn load() -> Result<Config, &'static str> {
        for path in CONFIG_PATH {
            if let Ok(config_str) = fs::read_to_string(path) {
                return if let Ok(config) = toml::from_str(&config_str) {
                    Ok(config)
                } else {
                    Err("cannot parse config file")
                };
            }
        }
        Err("config file not found")
    }
}
