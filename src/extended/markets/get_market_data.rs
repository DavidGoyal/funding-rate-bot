use anyhow::anyhow;

use crate::extended::structs::{MarketInfo, MarketInfoData};

pub async fn get_extended_market_data(market_name: &str) -> anyhow::Result<Vec<MarketInfoData>> {
    let url = format!(
        "https://api.starknet.extended.exchange/api/v1/info/markets?market={}",
        market_name
    );

    // Create a client with browser-like headers
    let client = reqwest::Client::new();
    let market_data = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
        .await?
        .json::<MarketInfo>()
        .await?;

    if market_data.status.eq("ERROR") || market_data.data.len() == 0 {
        return Err(anyhow!("Invalid Market Data"));
    }

    Ok(market_data.data)
}
