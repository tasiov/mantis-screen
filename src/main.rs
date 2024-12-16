use clap::Parser;
use mantis_raydium_client::cli::Cli;
use mantis_raydium_client::error::Result;
use tracing::info;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Run the application
    info!("Starting application");
    mantis_raydium_client::run(cli)
}
