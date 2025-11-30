use solana_sdk::{signature::Keypair, signer::Signer};

use crate::pacifica::structs::{TradeableBalance, TradeableBalanceData};

pub async fn get_pacifica_tradeable_balance() -> anyhow::Result<TradeableBalanceData> {
    let private_key = std::env::var("PACIFICA_PRIVATE_KEY").unwrap();
    let keypair = Keypair::from_base58_string(&private_key);
    let wallet_address = keypair.pubkey().to_string();
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
