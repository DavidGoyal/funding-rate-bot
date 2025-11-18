use std::sync::Arc;

use anyhow::anyhow;

use crate::extended::structs::{OpenPosition, OpenPositionData};

pub async fn get_extended_open_positions() -> anyhow::Result<Vec<OpenPositionData>> {
    let url = String::from("https://api.starknet.extended.exchange/api/v1/user/positions");
    let api_key = std::env::var("EXTENDED_API_KEY").unwrap();

    let client = reqwest::Client::new();
    let open_positions_data = client
         .get(&url)
         .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
         .header("X-Api-Key",  api_key)
         .send()
         .await?
         .json::<OpenPosition>()
         .await?;

    if open_positions_data.status.eq("ERROR") || open_positions_data.data.len() == 0 {
        return Err(anyhow!("Invalid Open Positions Data"));
    }

    Ok(open_positions_data.data)
}

pub async fn get_extended_open_position(
    market_name: &str,
    open_positions: Arc<Vec<OpenPositionData>>,
) -> Option<OpenPositionData> {
    for position in open_positions.iter() {
        if position.market == market_name {
            return Some(position.clone());
        }
    }
    None
}
