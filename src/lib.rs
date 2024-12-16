pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod utils;
use cli::{Cli, Command};
use config::Config;
use error::Result;
use tracing::info;

pub async fn run(cli: Cli) -> Result<()> {
    // Load config if specified
    let config = if let Some(path) = cli.config {
        Config::from_file(path)?
    } else {
        Config::default()
    };

    // Execute the requested command
    match cli.command {
        Command::FetchPool { pool_id } => {
            info!("Fetching pool {}", pool_id);
            commands::fetch_pool::execute(&config, &pool_id).await
        }
    }
}
