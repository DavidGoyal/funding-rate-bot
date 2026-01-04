use crate::pacifica::structs::{TradeableBalance, TradeableBalanceData};

pub async fn get_pacifica_tradeable_balance(
    wallet_address: &str,
) -> anyhow::Result<TradeableBalanceData> {
    let url = format!(
        "https://api.pacifica.fi/api/v1/account?account={}",
        wallet_address
    );

    let client = reqwest::Client::new();
    let tradeable_balance_data = client
        .get(&url)
        .send()
        .await?
        .json::<TradeableBalance>()
        .await?;

    if tradeable_balance_data.success == false {
        return Err(anyhow::anyhow!("Failed to get tradeable balance"));
    }

    Ok(tradeable_balance_data.data)
}
