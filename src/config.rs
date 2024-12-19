use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub rpc_endpoint: String,
    pub api_key: String,
    pub keypair_path: PathBuf,
}

impl Config {
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config: {}", e)))?;

        let v = toml::from_str(&contents)
            .map_err(|e| Error::Config(format!("Invalid config format: {}", e)))?;

        Ok(v)
    }

    pub fn default() -> Self {
        Self {
            rpc_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            api_key: "".to_string(),
            keypair_path: PathBuf::from("./keypair.json"),
        }
    }
}
