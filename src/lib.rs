pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod instructions;
pub mod utils;

use cli::{Cli, Command};
use config::Config;
use error::Result;
use tracing::info;
use utils::client::get_client;

pub async fn run(cli: Cli) -> Result<()> {
    // Load config if specified
    let config = if let Some(path) = cli.config {
        Config::from_file(path)?
    } else {
        Config::default()
    };

    let client = get_client(&config)?;

    // Execute the requested command
    match cli.command {
        Command::FetchPool { pool_id } => {
            info!("Fetching pool {}", pool_id);
            commands::fetch_pool::execute(&config, &pool_id).await
        }
        Command::AddLiquidity { pool_id } => {
            info!("Adding liquidity to pool {}", pool_id);
            commands::add_liquidity::execute(&config, &client, &pool_id).await
        }
    }
}
