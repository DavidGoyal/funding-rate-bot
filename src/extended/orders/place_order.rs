use crate::{
    extended::structs::{StopLoss, TakeProfit},
    utils::utils::{RoundingMode, calc_entire_position_size, round_to_min_change_f64},
};
use reqwest::Client;
use starknet::core::types::Felt;
use std::ops::{Add, Mul};

use crate::extended::structs::{
    CreateOrderParams, DebuggingAmounts, FeeResponse, FeeResponseData, MarketInfoData,
    OrderContext, PlaceOrder, Settlement, Side, Signature, StarknetDomain, StarknetDomainData,
};

const SLIPPAGE: f64 = 0.01;
const STARKNET_SETTLEMENT_BUFFER_SECONDS: u64 = 14 * 24 * 60 * 60;
const MILLIS_IN_SECOND: u64 = 1_000;

pub async fn place_extended_order(
    market_name: &str,
    market: &MarketInfoData,
    side: Side,
    qty: f64,
    tp_sl_included: bool,
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
            .bid_price
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
        order_price,
        tp_sl_included,
    )
    .await?;

    let response = client
        .post("https://api.starknet.extended.exchange/api/v1/user/order")
        .json(&place_order)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("X-Api-Key", &api_key)
        .send()
        .await?
        .text()
        .await?;

    println!("Response: {}", response);

    if response.contains("ERROR") {
        return Err(anyhow::anyhow!("Failed to place order"));
    }

    Ok(())
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
        min_price_change: market.trading_config.min_price_change.to_string(),
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
    normal_price: f64,
    tp_sl_included: bool,
) -> Result<PlaceOrder, anyhow::Error> {
    let nonce = rand::random_range(0..u32::MAX);
    let expiry_epoch_millis = chrono::Utc::now().timestamp_millis() as u64 + 1000 * 60 * 60;

    let is_buying = matches!(&side, &Side::Buy);

    if tp_sl_included {
        let rounding_mode = if is_buying {
            RoundingMode::Floor
        } else {
            RoundingMode::Ceil
        };
        let min_price_change = ctx.min_price_change.parse::<f64>().unwrap();

        let tp_trigger_price = round_to_min_change_f64(
            if is_buying {
                normal_price * 1.05
            } else {
                normal_price * 0.95
            },
            min_price_change,
            Some(rounding_mode),
        );
        let tp_price = round_to_min_change_f64(
            if is_buying {
                normal_price * 1.045
            } else {
                normal_price * 0.965
            },
            min_price_change,
            Some(rounding_mode),
        );
        let sl_trigger_price = round_to_min_change_f64(
            if is_buying {
                normal_price * 0.95
            } else {
                normal_price * 1.05
            },
            min_price_change,
            Some(rounding_mode),
        );
        let sl_price = round_to_min_change_f64(
            if is_buying {
                normal_price * 0.945
            } else {
                normal_price * 1.055
            },
            min_price_change,
            Some(rounding_mode),
        );

        let tp_amount_of_synthetic = calc_entire_position_size(
            &tp_price,
            &ctx.min_order_size_change.parse::<f64>().unwrap(),
            &ctx.max_position_value.parse::<f64>().unwrap(),
        );
        let sl_amount_of_synthetic = calc_entire_position_size(
            &sl_price,
            &ctx.min_order_size_change.parse::<f64>().unwrap(),
            &ctx.max_position_value.parse::<f64>().unwrap(),
        );

        let create_tp_order_params = get_create_order_params(
            &tp_amount_of_synthetic,
            &tp_price,
            &expiry_epoch_millis,
            &nonce,
            &ctx.fee_rate.parse::<f64>().unwrap(),
            ctx,
            !is_buying,
        )
        .await?;

        let create_sl_order_params = get_create_order_params(
            &sl_amount_of_synthetic,
            &sl_price,
            &expiry_epoch_millis,
            &nonce,
            &ctx.fee_rate.parse::<f64>().unwrap(),
            ctx,
            !is_buying,
        )
        .await?;

        let create_order_params = get_create_order_params(
            &qty,
            price,
            &expiry_epoch_millis,
            &nonce,
            &ctx.fee_rate.parse::<f64>().unwrap(),
            ctx,
            is_buying,
        )
        .await?;

        return Ok(PlaceOrder {
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
            debugging_amounts: create_order_params.debug_amounts,
            tp_sl_type: Some("POSITION".to_string()),
            take_profit: Some(TakeProfit {
                trigger_price: tp_trigger_price.to_string(),
                trigger_price_type: "LAST".to_string(),
                price: tp_price.to_string(),
                price_type: "MARKET".to_string(),
                settlement: create_tp_order_params.order_signature,
                debugging_amounts: create_tp_order_params.debug_amounts,
            }),
            stop_loss: Some(StopLoss {
                trigger_price: sl_trigger_price.to_string(),
                trigger_price_type: "LAST".to_string(),
                price: sl_price.to_string(),
                price_type: "MARKET".to_string(),
                settlement: create_sl_order_params.order_signature,
                debugging_amounts: create_sl_order_params.debug_amounts,
            }),
        });
    } else {
        let create_order_params = get_create_order_params(
            &qty,
            price,
            &expiry_epoch_millis,
            &nonce,
            &ctx.fee_rate.parse::<f64>().unwrap(),
            ctx,
            is_buying,
        )
        .await?;

        return Ok(PlaceOrder {
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
            debugging_amounts: create_order_params.debug_amounts,
            tp_sl_type: None,
            take_profit: None,
            stop_loss: None,
        });
    }
}

pub async fn get_create_order_params(
    amount_of_synthetic: &f64,
    price: &f64,
    expiry_epoch_millis: &u64,
    nonce: &u32,
    total_fee_rate: &f64,
    ctx: &OrderContext,
    is_buying: bool,
) -> Result<CreateOrderParams, anyhow::Error> {
    let collateral_amount = amount_of_synthetic.mul(price);
    let fee = total_fee_rate.mul(collateral_amount);

    let collateral_amount_stark = if is_buying {
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
    let synthetic_amount_stark = if is_buying {
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
        is_buying,
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
    is_buying_synthetic: bool,
) -> Result<Felt, anyhow::Error> {
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
