use crate::pacifica::structs::{OpenPosition, OpenPositionData};

pub async fn get_pacifica_open_positions() -> anyhow::Result<Vec<OpenPositionData>> {
    let wallet_address = std::env::var("PACIFICA_WALLET_ADDRESS").unwrap();
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

pub async fn get_pacifica_open_position<'a>(
    market_name: &str,
    open_positions: &'a Vec<OpenPositionData>,
) -> Option<&'a OpenPositionData> {
    for position in open_positions.iter() {
        if position.symbol == market_name {
            return Some(position);
        }
    }
    None
}
