use anyhow::anyhow;

use crate::pacifica::structs::{MarketInfo, MarketInfoData};

pub async fn get_pacifica_market_data(market_name: &str) -> anyhow::Result<MarketInfoData> {
    // Create a client with browser-like headers
    let client = reqwest::Client::new();
    let market_data = client
        .get("https://api.pacifica.fi/api/v1/info/prices")
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
        .await?
        .json::<MarketInfo>()
        .await?;

    if market_data.success == false || market_data.data.len() == 0 {
        return Err(anyhow!("Invalid Market Data"));
    }

    for data in market_data.data {
        if data.symbol == market_name {
            return Ok(data);
        }
    }

    Err(anyhow!("Market Data not found"))
}
