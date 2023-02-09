use serde::Deserialize;

#[derive(Deserialize)]
pub struct ServerConfig {
    pub ip: String,
    pub port: u32,
    pub token: String,
}
