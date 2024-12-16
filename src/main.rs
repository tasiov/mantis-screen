use clap::Parser;
use mantis_raydium_client::cli::Cli;
use mantis_raydium_client::error::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments first
    let cli = Cli::parse();

    // Initialize logging with debug flag from CLI
    setup_logging(cli.debug);

    // Run the application
    info!("Starting application");
    mantis_raydium_client::run(cli).await
}

fn setup_logging(debug: bool) {
    let level = if debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false) // Removes the target from the output
        .with_thread_ids(false) // Removes thread IDs from the output
        .with_line_number(true) // Includes line numbers
        .init();
}
