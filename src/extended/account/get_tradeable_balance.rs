use anyhow::anyhow;

use crate::extended::structs::{TradeableBalance, TradeableBalanceData};

pub async fn get_extended_tradeable_balance(api_key: &str) -> anyhow::Result<TradeableBalanceData> {
    let url = String::from("https://api.starknet.extended.exchange/api/v1/user/balance");

    let client = reqwest::Client::new();
    let tradeable_balance_data = client
         .get(&url)
         .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
         .header("X-Api-Key",  api_key)
         .send()
         .await?
         .json::<TradeableBalance>()
         .await?;

    if tradeable_balance_data.status.eq("ERROR") {
        return Err(anyhow!("Failed to get tradeable balance"));
    }

    Ok(tradeable_balance_data.data)
}
