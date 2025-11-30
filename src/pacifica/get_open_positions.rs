use solana_sdk::{signature::Keypair, signer::Signer};

use crate::pacifica::structs::{OpenPosition, OpenPositionData};

pub async fn get_pacifica_open_positions() -> anyhow::Result<Vec<OpenPositionData>> {
    let private_key = std::env::var("PACIFICA_PRIVATE_KEY").unwrap();
    let keypair = Keypair::from_base58_string(&private_key);
    let wallet_address = keypair.pubkey().to_string();
    let url = format!(
        "https://api.pacifica.fi/api/v1/positions?account={}",
        wallet_address
    );

    let client = reqwest::Client::new();
    let open_orders_data = client
        .get(&url)
        .send()
        .await?
        .json::<OpenPosition>()
        .await?;

    if open_orders_data.success == false {
        return Err(anyhow::anyhow!("Failed to get open orders"));
    }

    Ok(open_orders_data.data)
}
