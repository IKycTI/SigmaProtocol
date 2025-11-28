use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    name: String,
    address: Address,
    second_server: Address,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Address {
    ip: Option<String>,
    port: Option<u32>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        let json_content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&json_content)?;
        Ok(config)
    }
    pub fn get_address(&self) -> String {
        self.address.get()
    }
}

impl Address {
    pub fn get(&self) -> String {
        format!(
            "{}:{}",
            self.ip.as_ref().unwrap_or(&String::from("localhost")),
            self.port.unwrap_or(8080)
        )
    }
}
