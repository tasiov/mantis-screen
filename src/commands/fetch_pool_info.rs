use crate::{config::Config, error::Error, utils::printer::pretty_print};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub id: String,
    pub success: bool,
    pub data: Vec<PoolInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolInfo {
    #[serde(rename = "type")]
    pub pool_type: String,
    pub programId: String,
    pub id: String,
    pub mintA: TokenInfo,
    pub mintB: TokenInfo,
    pub price: f64,
    pub mintAmountA: f64,
    pub mintAmountB: f64,
    pub feeRate: f64,
    pub openTime: String,
    pub tvl: f64,
    pub day: PeriodStats,
    pub week: PeriodStats,
    pub month: PeriodStats,
    pub pooltype: Vec<String>,
    pub rewardDefaultInfos: Vec<Value>,
    pub farmUpcomingCount: i32,
    pub farmOngoingCount: i32,
    pub farmFinishedCount: i32,
    pub marketId: String,
    pub lpMint: LpMintInfo,
    pub lpPrice: f64,
    pub lpAmount: f64,
    pub burnPercent: f64,
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
pub struct PeriodStats {
    pub volume: f64,
    pub volumeQuote: f64,
    pub volumeFee: f64,
    pub apr: f64,
    pub feeApr: f64,
    pub priceMin: f64,
    pub priceMax: f64,
    pub rewardApr: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LpMintInfo {
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

pub async fn execute(config: &Config, pool_id: &str) -> Result<(), Error> {
    let pool = fetch_pool_info(config, pool_id).await?;
    info!("{}", pretty_print(&pool));
    Ok(())
}

pub async fn fetch_pool_info(config: &Config, pool_id: &str) -> Result<ApiResponse, Error> {
    let url = format!("https://api-v3.raydium.io/pools/info/ids?ids={}", pool_id);

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
