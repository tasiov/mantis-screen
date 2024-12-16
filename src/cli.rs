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
    FetchPool {
        /// Pool ID
        #[arg(short, long)]
        pool_id: String,
    },
    /// Add liquidity to a pool
    AddLiquidity {
        /// Pool ID
        #[arg(short, long)]
        pool_id: String,
    },
}
