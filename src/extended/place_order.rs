use crate::{
    extended::structs::PlaceOrderResponse,
    utils::utils::{RoundingMode, round_to_min_change_f64},
};
use reqwest::Client;
use starknet::core::types::Felt;
use std::ops::{Add, Mul};

use crate::extended::structs::{
    CreateOrderParams, DebuggingAmounts, FeeResponse, FeeResponseData, MarketInfoData,
    OrderContext, PlaceOrder, Settlement, Side, Signature, StarknetDomain, StarknetDomainData,
};

const SLIPPAGE: f64 = 0.001;
const STARKNET_SETTLEMENT_BUFFER_SECONDS: u64 = 14 * 24 * 60 * 60;
const MILLIS_IN_SECOND: u64 = 1_000;

pub async fn place_extended_order(
    market_name: &str,
    market: &MarketInfoData,
    side: Side,
    qty: f64,
) -> anyhow::Result<()> {
    let api_key = std::env::var("EXTENDED_API_KEY").unwrap();
    let stark_private_key = std::env::var("EXTENDED_STARK_PRIVATE_KEY").unwrap();
    let vault_id = std::env::var("EXTENDED_VAULT_ID").unwrap();
    let client = reqwest::Client::new();

    let fees_vec = get_fees(&client, market_name, &api_key).await?;
    let fees = fees_vec.first().unwrap();

    let order_price = if matches!(side, Side::Buy) {
        market
            .market_stats
            .ask_price
            .parse::<f64>()
            .unwrap()
            .mul(1.0 + SLIPPAGE)
    } else {
        market
            .market_stats
            .bid_price
            .parse::<f64>()
            .unwrap()
            .mul(1.0 - SLIPPAGE)
    };

    let starknet_domain = get_starknet_domain(&client).await?;
    let ctx =
        create_order_context(market, fees, starknet_domain, &vault_id, &stark_private_key).await;

    let place_order = create_order(
        market_name,
        side,
        &round_to_min_change_f64(
            qty,
            market
                .trading_config
                .min_order_size_change
                .parse::<f64>()
                .unwrap(),
            Some(RoundingMode::Floor),
        ),
        &round_to_min_change_f64(
            order_price,
            market
                .trading_config
                .min_price_change
                .parse::<f64>()
                .unwrap(),
            Some(RoundingMode::Floor),
        ),
        &ctx,
    )
    .await?;
    println!(
        "Place Order JSON: {}",
        serde_json::to_string_pretty(&place_order).unwrap()
    );

    let response = client
        .post("https://api.starknet.extended.exchange/api/v1/user/order")
        .json(&place_order)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("X-Api-Key", &api_key)
        .send()
        .await?
        .json::<PlaceOrderResponse>()
        .await?;

    println!("Response: {:?}", response);
    if response.status.eq("OK") {
        return Ok(());
    }

    Err(anyhow::anyhow!("Failed to place order"))
}

pub async fn get_fees(
    client: &Client,
    market_name: &str,
    api_key: &str,
) -> anyhow::Result<Vec<FeeResponseData>> {
    let fee_response = client
        .get(&format!(
            "https://api.starknet.extended.exchange/api/v1/user/fees?market={}",
            market_name
        ))
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("X-Api-Key", api_key)
        .send()
        .await?
        .json::<FeeResponse>()
        .await?;

    if fee_response.status.eq("ERROR") || fee_response.data.len() == 0 {
        return Err(anyhow::anyhow!("Failed to get fees"));
    }

    Ok(fee_response.data)
}

pub async fn get_starknet_domain(client: &Client) -> anyhow::Result<StarknetDomainData> {
    let starknet_domain_response = client
        .get("https://api.starknet.extended.exchange/api/v1/info/starknet")
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
        .await?
        .json::<StarknetDomain>()
        .await?;

    if starknet_domain_response.status.eq("ERROR") {
        return Err(anyhow::anyhow!("Failed to get starknet domain"));
    }

    Ok(starknet_domain_response.data)
}

pub async fn create_order_context(
    market: &MarketInfoData,
    fees: &FeeResponseData,
    starknet_domain: StarknetDomainData,
    vault_id: &str,
    stark_private_key: &str,
) -> OrderContext {
    OrderContext {
        asset_id_collateral: market.l2_config.collateral_id.to_string(),
        asset_id_synthetic: market.l2_config.synthetic_id.to_string(),
        settlement_resolution_collateral: market.l2_config.collateral_resolution.to_string(),
        settlement_resolution_synthetic: market.l2_config.synthetic_resolution.to_string(),
        min_order_size_change: market.trading_config.min_order_size_change.to_string(),
        max_position_value: market.trading_config.max_position_value.to_string(),
        fee_rate: fees.taker_fee_rate.to_string(),
        vault_id: vault_id.to_string(),
        stark_private_key: stark_private_key.to_string(),
        starknet_domain: starknet_domain,
    }
}

pub async fn create_order(
    market_name: &str,
    side: Side,
    qty: &f64,
    price: &f64,
    ctx: &OrderContext,
) -> Result<PlaceOrder, anyhow::Error> {
    let nonce = rand::random_range(0..u32::MAX);
    let expiry_epoch_millis = chrono::Utc::now().timestamp_millis() as u64 + 1000 * 60 * 60;

    let create_order_params = get_create_order_params(
        &side,
        &qty,
        price,
        &expiry_epoch_millis,
        &nonce,
        &ctx.fee_rate.parse::<f64>().unwrap(),
        ctx,
    )
    .await?;

    Ok(PlaceOrder {
        id: create_order_params.order_hash,
        market: market_name.to_string(),
        side: side,
        qty: qty.to_string(),
        r#type: "MARKET".to_string(),
        price: price.to_string(),
        reduce_only: false,
        post_only: false,
        time_in_force: "IOC".to_string(),
        expiry_epoch_millis: expiry_epoch_millis,
        fee: ctx.fee_rate.to_string(),
        nonce: nonce.to_string(),
        settlement: create_order_params.order_signature,
        self_trade_protection_level: "ACCOUNT".to_string(),
        take_profit: None,
        stop_loss: None,
        debugging_amounts: create_order_params.debug_amounts,
    })
}

pub async fn get_create_order_params(
    side: &Side,
    amount_of_synthetic: &f64,
    price: &f64,
    expiry_epoch_millis: &u64,
    nonce: &u32,
    total_fee_rate: &f64,
    ctx: &OrderContext,
) -> Result<CreateOrderParams, anyhow::Error> {
    let collateral_amount = amount_of_synthetic.mul(price);
    let fee = total_fee_rate.mul(collateral_amount);

    let collateral_amount_stark = if matches!(side, Side::Buy) {
        collateral_amount
            .mul(ctx.settlement_resolution_collateral.parse::<f64>().unwrap())
            .ceil()
    } else {
        collateral_amount
            .mul(ctx.settlement_resolution_collateral.parse::<f64>().unwrap())
            .floor()
    };

    let fee_stark = fee
        .mul(ctx.settlement_resolution_collateral.parse::<f64>().unwrap())
        .ceil();
    let synthetic_amount_stark = if matches!(side, Side::Buy) {
        amount_of_synthetic
            .mul(ctx.settlement_resolution_synthetic.parse::<f64>().unwrap())
            .ceil()
    } else {
        amount_of_synthetic
            .mul(ctx.settlement_resolution_synthetic.parse::<f64>().unwrap())
            .floor()
    };

    let stark_public_key = std::env::var("EXTENDED_STARK_PUBLIC_KEY").unwrap();

    let order_hash = get_starknet_order_msg_hash(
        side,
        nonce,
        &ctx.asset_id_collateral,
        &ctx.asset_id_synthetic,
        &collateral_amount_stark,
        &fee_stark,
        &synthetic_amount_stark,
        expiry_epoch_millis,
        &ctx.vault_id,
        &stark_public_key.to_string(),
        &ctx.starknet_domain,
    )
    .await?;

    let order_signature = sign_message(
        &order_hash,
        &ctx.stark_private_key,
        &stark_public_key,
        &ctx.vault_id,
    )
    .await?;

    Ok(CreateOrderParams {
        order_hash: order_hash.to_string(),
        order_signature: order_signature,
        debug_amounts: DebuggingAmounts {
            collateral_amount: collateral_amount_stark.to_string(),
            fee_amount: fee_stark.to_string(),
            synthetic_amount: synthetic_amount_stark.to_string(),
        },
    })
}

pub async fn get_starknet_order_msg_hash(
    side: &Side,
    nonce: &u32,
    asset_id_collateral: &str,
    asset_id_synthetic: &str,
    collateral_amount_stark: &f64,
    fee_stark: &f64,
    synthetic_amount_stark: &f64,
    expiry_epoch_millis: &u64,
    vault_id: &str,
    stark_public_key: &str,
    starknet_domain: &StarknetDomainData,
) -> Result<Felt, anyhow::Error> {
    let is_buying_synthetic = matches!(side, Side::Buy);
    let expiration_timestamp = expiry_epoch_millis
        .div_ceil(MILLIS_IN_SECOND)
        .add(STARKNET_SETTLEMENT_BUFFER_SECONDS);

    let amount_collateral = if is_buying_synthetic {
        &collateral_amount_stark.mul(-1.0)
    } else {
        collateral_amount_stark
    };

    let amount_synthetic = if is_buying_synthetic {
        synthetic_amount_stark
    } else {
        &synthetic_amount_stark.mul(-1.0)
    };

    let wasm_hash: Result<Felt, String> = rust_crypto_lib_base::get_order_hash(
        vault_id.to_string(),
        asset_id_synthetic.to_string(),
        amount_synthetic.to_string(),
        asset_id_collateral.to_string(),
        amount_collateral.to_string(),
        asset_id_collateral.to_string(),
        fee_stark.to_string(),
        expiration_timestamp.to_string(),
        nonce.to_string(),
        stark_public_key.to_string(),
        starknet_domain.name.to_string(),
        starknet_domain.version.to_string(),
        starknet_domain.chain_id.to_string(),
        starknet_domain.revision.to_string(),
    );

    match wasm_hash {
        Ok(hash) => Ok(hash),
        Err(error) => Err(anyhow::anyhow!("Failed to get wasm hash: {}", error)),
    }
}

pub async fn sign_message(
    message: &Felt,
    private_key: &str,
    stark_public_key: &str,
    vault_id: &str,
) -> Result<Settlement, anyhow::Error> {
    let wasm_signature =
        rust_crypto_lib_base::sign_message(message, &Felt::from_hex(private_key).unwrap());

    match wasm_signature {
        Ok(signature) => {
            let result = Settlement {
                signature: Signature {
                    r: signature.r.to_hex_string(),
                    s: signature.s.to_hex_string(),
                },
                stark_key: stark_public_key.to_string(),
                collateral_position: vault_id.to_string(),
            };
            Ok(result)
        }
        Err(error) => Err(anyhow::anyhow!("Failed to sign message: {}", error)),
    }
}
