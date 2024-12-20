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
        Command::FetchPoolInfo { pool_id } => {
            info!("Fetching pool {}", pool_id);
            commands::fetch_pool_info::execute(&config, &pool_id).await
        }
        Command::FetchPoolKeys { pool_id } => {
            info!("Fetching pool keys {}", pool_id);
            commands::fetch_pool_keys::execute(&config, &pool_id).await
        }
        Command::AddLiquidity {
            pool_id,
            mint_pubkey,
            amount,
            slippage_percentage,
        } => {
            info!("Adding liquidity to pool {}", pool_id);
            commands::add_liquidity::execute(
                &config,
                &client,
                &pool_id,
                &mint_pubkey,
                amount,
                slippage_percentage,
            )
            .await
        }
        Command::RemoveLiquidity {
            pool_id,
            lp_amount,
            slippage_percentage,
            base_amount_min,
            quote_amount_min,
        } => {
            info!("Removing liquidity from pool {}", pool_id);
            commands::remove_liquidity::execute(
                &config,
                &client,
                &pool_id,
                lp_amount,
                slippage_percentage,
                base_amount_min,
                quote_amount_min,
            )
            .await
        }
    }
}
