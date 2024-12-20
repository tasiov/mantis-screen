use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Keypair error: {0}")]
    Keypair(String),

    #[error("RPC client error: {0}")]
    RpcClient(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Math error: {0}")]
    Math(String),

    #[error("Invalid token account: {0}")]
    InvalidTokenAccount(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
