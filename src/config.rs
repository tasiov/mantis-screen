use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub rpc_endpoint: String,
    pub keypair_path: PathBuf,
    pub debug: bool,
}

impl Config {
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config: {}", e)))?;

        toml::from_str(&contents)
            .map_err(|e| Error::Config(format!("Invalid config format: {}", e)))
    }

    pub fn default() -> Self {
        Self {
            rpc_endpoint: "https://api.devnet.solana.com".to_string(),
            keypair_path: PathBuf::from("./keypair.json"),
            debug: false,
        }
    }
}
