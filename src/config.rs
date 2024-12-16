use crate::{
    error::{Error, Result},
    utils::pretty_print,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub rpc_endpoint: String,
    pub keypair_path: PathBuf,
    pub urls: Urls,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Urls {
    pub raydium_base_host: String,
    pub raydium_pool_search_by_id: String,
}

impl Config {
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config: {}", e)))?;

        let v = toml::from_str(&contents)
            .map_err(|e| Error::Config(format!("Invalid config format: {}", e)))?;

        debug!("Config: {}", pretty_print(&v));

        Ok(v)
    }

    pub fn default() -> Self {
        Self {
            rpc_endpoint: "https://api.devnet.solana.com".to_string(),
            keypair_path: PathBuf::from("./keypair.json"),
            urls: Urls {
                raydium_base_host: "".to_string(),
                raydium_pool_search_by_id: "".to_string(),
            },
        }
    }
}
