pub mod cli;
pub mod config;
pub mod error;

use cli::Cli;
use config::Config;
use error::Result;

pub fn run(cli: Cli) -> Result<()> {
    // Load config if specified
    let _config = if let Some(path) = cli.config {
        Config::from_file(path)?
    } else {
        Config::default()
    };

    // Execute the requested command
    match cli.command {
        cli::Command::FetchPool { pool_id } => {
            println!("Fetching pool {}", pool_id);
            Ok(())
        }
    }
}
