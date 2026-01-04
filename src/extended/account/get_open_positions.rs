use anyhow::anyhow;

use crate::extended::structs::{OpenPosition, OpenPositionData};

pub async fn get_extended_open_positions(api_key: &str) -> anyhow::Result<Vec<OpenPositionData>> {
    let url = String::from("https://api.starknet.extended.exchange/api/v1/user/positions");

    let client = reqwest::Client::new();
    let open_positions_data = client
         .get(&url)
         .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
         .header("X-Api-Key",  api_key)
         .send()
         .await?
         .json::<OpenPosition>()
         .await?;

    if open_positions_data.status.eq("ERROR") {
        return Err(anyhow!("Failed to get open positions"));
    }

    Ok(open_positions_data.data)
}
