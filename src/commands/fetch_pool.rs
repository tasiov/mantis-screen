use crate::{config::Config, error::Error, utils::pretty_print};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub id: String,
    pub success: bool,
    pub data: Vec<PoolData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolData {
    #[serde(rename = "type")]
    pub pool_type: String,
    pub programId: String, // Changed from program_id to match JSON
    pub id: String,
    pub tvl: f64,
    pub price: f64,
    pub mintAmountA: f64, // Changed from mint_amount_a to match JSON
    pub mintAmountB: f64, // Changed from mint_amount_b to match JSON
    pub feeRate: f64,     // Changed from fee_rate to match JSON
    pub mintA: TokenInfo, // Changed from mint_a to match JSON
    pub mintB: TokenInfo, // Changed from mint_b to match JSON
    pub day: PeriodStats,
    pub week: PeriodStats,
    pub month: PeriodStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub chainId: i64, // Added this field
    pub address: String,
    pub programId: String, // Added this field
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
    pub tags: Vec<String>, // Added this field
    pub extensions: Value, // Added this for flexible JSON object
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
    pub rewardApr: Vec<f64>, // Added this field
}

pub async fn execute(config: &Config, pool_id: &str) -> Result<(), Error> {
    let pool = fetch_pool(config, pool_id).await?;
    info!("{}", pretty_print(&pool));
    Ok(())
}

pub async fn fetch_pool(config: &Config, pool_id: &str) -> Result<ApiResponse, Error> {
    let url = format!(
        "{}{}?ids={}",
        config.urls.raydium_base_host, config.urls.raydium_pool_search_by_id, pool_id
    );

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
