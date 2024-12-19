use crate::{config::Config, error::Error, utils::printer::pretty_print};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub id: String,
    pub success: bool,
    pub data: Vec<PoolKeys>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolKeys {
    pub programId: String,
    pub id: String,
    pub mintA: TokenInfo,
    pub mintB: TokenInfo,
    pub lookupTableAccount: String,
    pub openTime: String,
    pub vault: VaultInfo,
    pub authority: String,
    pub openOrders: String,
    pub targetOrders: String,
    pub mintLp: TokenInfo,
    pub marketProgramId: String,
    pub marketId: String,
    pub marketAuthority: String,
    pub marketBaseVault: String,
    pub marketQuoteVault: String,
    pub marketBids: String,
    pub marketAsks: String,
    pub marketEventQueue: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub chainId: i64,
    pub address: String,
    pub programId: String,
    pub logoURI: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
    pub tags: Vec<String>,
    pub extensions: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultInfo {
    pub A: String,
    pub B: String,
}

pub async fn execute(config: &Config, pool_id: &str) -> Result<(), Error> {
    let pool = fetch_pool_keys(config, pool_id).await?;
    info!("{}", pretty_print(&pool));
    Ok(())
}

pub async fn fetch_pool_keys(config: &Config, pool_id: &str) -> Result<ApiResponse, Error> {
    let url = format!("https://api-v3.raydium.io/pools/key/ids?ids={}", pool_id);

    debug!("Requesting URL: {}", url);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| Error::Api(e.to_string()))?;

    if !response.status().is_success() {
        return Err(Error::Api(format!("API error: {}", response.status())));
    }

    // Get raw response and log it in debug mode
    let text = response
        .text()
        .await
        .map_err(|e| Error::Api(e.to_string()))?;
    debug!("Raw response: {}", text);

    // Parse response
    let pool: ApiResponse =
        serde_json::from_str(&text).map_err(|e| Error::Api(format!("Parse error: {}", e)))?;

    Ok(pool)
}
