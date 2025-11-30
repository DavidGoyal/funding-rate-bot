use chrono::Utc;
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::{
    pacifica::structs::{
        MarketInfoData, PlaceOrder, Side, SignatureHeader, SignaturePayload, SignedMessage,
        StopLoss, TakeProfit,
    },
    utils::utils::{RoundingMode, round_to_min_change_f64},
};

const SLIPPAGE: f64 = 0.01;

pub async fn place_pacifica_order(
    market_name: &str,
    side: Side,
    qty: f64,
    market_info: &MarketInfoData,
) -> anyhow::Result<()> {
    let private_key = std::env::var("PACIFICA_PRIVATE_KEY").unwrap();

    let keypair = Keypair::from_base58_string(&private_key);
    let public_key = keypair.pubkey();

    let current_timestamp = Utc::now().timestamp_millis();

    let market_price = market_info.mid.parse::<f64>()?;
    let qty = round_to_min_change_f64(
        qty,
        market_info.lot_size.parse::<f64>()?,
        Some(RoundingMode::Floor),
    );

    let take_profit_price = round_to_min_change_f64(
        market_price * 1.05,
        market_info.tick_size.parse::<f64>()?,
        Some(RoundingMode::Ceil),
    );
    let stop_loss_price = round_to_min_change_f64(
        market_price * 0.95,
        market_info.tick_size.parse::<f64>()?,
        Some(RoundingMode::Floor),
    );

    let signature_header = SignatureHeader {
        timestamp: current_timestamp as u64,
        expiry_window: 5000u64,
        r#type: "create_market_order".to_string(),
    };

    let signature_payload = SignaturePayload {
        symbol: market_name.to_string(),
        side: side,
        reduce_only: false,
        amount: qty.to_string(),
        slippage_percent: SLIPPAGE.to_string(),
        client_order_id: uuid::Uuid::new_v4().to_string(),
        take_profit: Some(TakeProfit {
            stop_price: take_profit_price.to_string(),
            client_order_id: uuid::Uuid::new_v4().to_string(),
        }),
        stop_loss: Some(StopLoss {
            stop_price: stop_loss_price.to_string(),
            client_order_id: uuid::Uuid::new_v4().to_string(),
        }),
    };

    let signature = sign_message(&signature_header, &signature_payload, &keypair).await?;

    let place_order = PlaceOrder {
        account: public_key.to_string(),
        signature: signature,
        timestamp: signature_header.timestamp,
        expiry_window: signature_header.expiry_window,
        symbol: signature_payload.symbol,
        side: signature_payload.side,
        reduce_only: false,
        amount: signature_payload.amount,
        slippage_percent: SLIPPAGE.to_string(),
        client_order_id: signature_payload.client_order_id,
        take_profit: signature_payload.take_profit,
        stop_loss: signature_payload.stop_loss,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.pacifica.fi/api/v1/orders/create_market")
        .json(&place_order)
        .send()
        .await?;

    if response.status().is_success() {
        return Ok(());
    } else {
        return Err(anyhow::anyhow!("Failed to place order"));
    }
}

pub async fn sign_message(
    header: &SignatureHeader,
    payload: &SignaturePayload,
    keypair: &Keypair,
) -> Result<String, anyhow::Error> {
    let message = SignedMessage {
        timestamp: header.timestamp,
        expiry_window: header.expiry_window,
        r#type: header.r#type.to_string(),
        data: payload.clone(),
    };
    let sorted_message = sort_json_object(&message)?;
    println!("Sorted Message: {}", sorted_message);
    let signature = keypair.sign_message(sorted_message.as_bytes());

    Ok(bs58::encode(signature).into_string())
}

pub fn sort_json_object(message: &SignedMessage) -> Result<String, anyhow::Error> {
    // Serialize to JSON value
    let json_value = serde_json::to_value(message)?;

    // Sort the JSON object recursively
    let sorted_value = sort_json_value(json_value);

    // Convert to string without pretty printing
    Ok(serde_json::to_string(&sorted_value)?)
}

fn sort_json_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted_map = serde_json::Map::new();
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            for key in keys {
                if let Some(val) = map.get(&key) {
                    sorted_map.insert(key, sort_json_value(val.clone()));
                }
            }
            serde_json::Value::Object(sorted_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(sort_json_value).collect())
        }
        _ => value,
    }
}
