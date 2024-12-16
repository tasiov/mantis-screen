use crate::{
    commands::fetch_pool::fetch_pool,
    config::Config,
    error::Error,
    utils::{client::Client, pretty_print},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::{
    commitment_config::CommitmentConfig, hash::Hash, pubkey::Pubkey, signature::Signature,
    transaction::Transaction,
};
use std::str::FromStr;
use tracing::{debug, info};

use super::fetch_pool::PoolData;

pub async fn execute(config: &Config, client: &Client, pool_id: &str) -> Result<(), Error> {
    let api_response = fetch_pool(config, pool_id).await?;
    let pool = api_response.data.first().unwrap();
    let signature = add_liquidity(config, client, pool).await?;

    if let Some(signature) = signature {
        info!("Transaction sent with signature: {}", signature);
    } else {
        info!("No transaction sent");
    }

    Ok(())
}

async fn add_liquidity(
    config: &Config,
    client: &Client,
    pool: &PoolData,
) -> Result<Option<Signature>, Error> {
    let mint_a =
        Pubkey::from_str(&pool.mintA.address).map_err(|e| Error::RpcClient(e.to_string()))?;
    let mint_b =
        Pubkey::from_str(&pool.mintB.address).map_err(|e| Error::RpcClient(e.to_string()))?;

    let (token_account_a, create_ata_ix) = client.get_or_create_token_account(&mint_a)?;
    let (token_account_b, create_ata_b_ix) = client.get_or_create_token_account(&mint_b)?;

    let mut instructions = vec![];

    if let Some(create_ata_ix) = create_ata_ix {
        instructions.push(create_ata_ix);
    }

    if let Some(create_ata_b_ix) = create_ata_b_ix {
        instructions.push(create_ata_b_ix);
    }

    let mut signature = None;

    if !instructions.is_empty() {
        signature = Some(client.send_transaction(&instructions)?);
    }

    Ok(signature)
}
