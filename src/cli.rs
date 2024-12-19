use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Optional config file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Enable debug mode
    #[arg(short, long)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Fetch pool data by pool id
    FetchPoolInfo {
        /// Pool ID
        #[arg(short, long)]
        pool_id: String,
    },
    /// Fetch pool keys by pool id
    FetchPoolKeys {
        /// Pool ID
        #[arg(short, long)]
        pool_id: String,
    },
    /// Add liquidity to a pool
    AddLiquidity {
        /// Pool ID
        #[arg(short, long)]
        pool_id: String,
        /// Base mint pubkey
        #[arg(short, long)]
        mint_pubkey: String,
        /// Base amount
        #[arg(short, long)]
        amount: f64,
        /// Slippage percentage
        #[arg(short, long)]
        slippage_percentage: f64,
    },
    /// Remove liquidity from a pool
    RemoveLiquidity {
        /// Pool ID
        #[arg(short, long)]
        pool_id: String,
        /// LP amount
        #[arg(short, long)]
        lp_amount: f64,
        /// Slippage percentage
        #[arg(short, long)]
        slippage_percentage: f64,
        // Base amount min
        #[arg(short, long)]
        base_amount_min: f64,
        // Quote amount min
        #[arg(short, long)]
        quote_amount_min: f64,
    },
}
