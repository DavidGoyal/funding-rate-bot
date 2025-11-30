use anyhow::anyhow;

use crate::pacifica::structs::{MarketInfoData, MarketPricesInfo, MarketTradingInfo};

pub async fn get_pacifica_market_data(market_name: &str) -> anyhow::Result<MarketInfoData> {
    // Create a client with browser-like headers
    let client = reqwest::Client::new();
    let market_price_data = client
        .get("https://api.pacifica.fi/api/v1/info/prices")
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
        .await?
        .json::<MarketPricesInfo>()
        .await?;

    let market_trading_data= client
    .get("https://api.pacifica.fi/api/v1/info")
    .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
    .send()
    .await?
    .json::<MarketTradingInfo>()
    .await?;

    if market_price_data.success == false || market_price_data.data.len() == 0 {
        return Err(anyhow!("Invalid Market Data"));
    }

    if market_trading_data.success == false || market_trading_data.data.len() == 0 {
        return Err(anyhow!("Invalid Market Data"));
    }

    for data in market_price_data.data {
        if data.symbol == market_name {
            for trading_data in market_trading_data.data.iter() {
                if trading_data.symbol == market_name {
                    return Ok(MarketInfoData {
                        mid: data.mid,
                        next_funding: data.next_funding,
                        symbol: data.symbol,
                        tick_size: trading_data.tick_size.to_string(),
                        min_tick: trading_data.min_tick.to_string(),
                        max_tick: trading_data.max_tick.to_string(),
                        lot_size: trading_data.lot_size.to_string(),
                        min_order_size: trading_data.min_order_size.to_string(),
                        max_order_size: trading_data.max_order_size.to_string(),
                    });
                }
            }
        }
    }

    Err(anyhow!("Market Data not found"))
}
