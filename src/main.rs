use chrono::{FixedOffset, Timelike, Utc};
use dotenvy::dotenv;
use tokio::time::Duration;

use crate::{
    extended::{
        account::{
            get_open_positions::get_extended_open_positions,
            get_tradeable_balance::get_extended_tradeable_balance,
        },
        markets::get_market_data::get_extended_market_data,
        orders::place_order::place_extended_order,
        structs::{OpenPositionData as ExtendedOpenPositionData, Side as ExtendedSide},
    },
    pacifica::{
        account::{
            get_open_positions::get_pacifica_open_positions,
            get_tradeable_balance::get_pacifica_tradeable_balance,
        },
        markets::get_market_data::get_pacifica_market_data,
        orders::place_order::place_pacifica_order,
        structs::{OpenPositionData as PacificaOpenPositionData, Side as PacificaSide},
    },
};

mod extended;
mod pacifica;
mod utils;

const FUNDING_RATE_THRESHOLD: f64 = 0.001;
const PRICE_SPREAD_THRESHOLD: f64 = 0.02;

const TARGET_MINUTE: u32 = 28;
const BUY_AMOUNT: f64 = 25.0;

/// Calculates the duration until the next target minute (:29) in IST
fn duration_until_next_target() -> Duration {
    // IST is UTC+5:30
    let ist = FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
    let now_ist = Utc::now().with_timezone(&ist);

    let current_minute = now_ist.minute();
    let current_second = now_ist.second();

    let minutes_until_target = if current_minute < TARGET_MINUTE {
        TARGET_MINUTE - current_minute
    } else {
        // Next hour's :29
        60 - current_minute + TARGET_MINUTE
    };

    // Calculate total seconds, subtracting current seconds within the minute
    let total_seconds = (minutes_until_target * 60) as i64 - current_second as i64;

    // If we're exactly at :29:00, wait for next hour
    let total_seconds = if total_seconds <= 0 {
        total_seconds + 3600
    } else {
        total_seconds
    };

    Duration::from_secs(total_seconds as u64)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let extended_market_names = vec![
        String::from("ETH-USD"),
        String::from("HYPE-USD"),
        String::from("1000BONK-USD"),
        String::from("1000PEPE-USD"),
        String::from("PENGU-USD"),
        String::from("DOGE-USD"),
        String::from("UNI-USD"),
        String::from("SOL-USD"),
        String::from("PUMP-USD"),
        String::from("XRP-USD"),
        String::from("ASTER-USD"),
        String::from("AVAX-USD"),
        String::from("TRUMP-USD"),
        String::from("SUI-USD"),
        String::from("FARTCOIN-USD"),
        String::from("LINK-USD"),
    ];

    let pacifica_market_names = vec![
        String::from("ETH"),
        String::from("HYPE"),
        String::from("kBONK"),
        String::from("kPEPE"),
        String::from("PENGU"),
        String::from("DOGE"),
        String::from("UNI"),
        String::from("SOL"),
        String::from("PUMP"),
        String::from("XRP"),
        String::from("ASTER"),
        String::from("AVAX"),
        String::from("TRUMP"),
        String::from("SUI"),
        String::from("FARTCOIN"),
        String::from("LINK"),
    ];

    let pacifica_wallet_address =
        std::env::var("PACIFICA_WALLET_ADDRESS").expect("PACIFICA_WALLET_ADDRESS must be set");
    let pacifica_private_key =
        std::env::var("PACIFICA_PRIVATE_KEY").expect("PACIFICA_PRIVATE_KEY must be set");
    let extended_api_key = std::env::var("EXTENDED_API_KEY").expect("EXTENDED_API_KEY must be set");
    let extended_stark_private_key = std::env::var("EXTENDED_STARK_PRIVATE_KEY")
        .expect("EXTENDED_STARK_PRIVATE_KEY must be set");
    let extended_vault_id =
        std::env::var("EXTENDED_VAULT_ID").expect("EXTENDED_VAULT_ID must be set");
    let extended_stark_public_key =
        std::env::var("EXTENDED_STARK_PUBLIC_KEY").expect("EXTENDED_STARK_PUBLIC_KEY must be set");

    loop {
        let wait_duration = duration_until_next_target();
        let ist = FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
        let now_ist = Utc::now().with_timezone(&ist);
        println!(
            "Current Time IST: {}, Waiting for {} seconds until next :{TARGET_MINUTE} IST",
            now_ist.format("%H:%M:%S"),
            wait_duration.as_secs()
        );

        tokio::time::sleep(wait_duration).await;

        let now_ist = Utc::now().with_timezone(&ist);
        println!(
            "Current Time IST: {}, Checking for opportunities",
            now_ist.format("%H:%M:%S")
        );

        let extended_open_positions = get_extended_open_positions(&extended_api_key).await?;
        println!("Extended Open Positions: {:?}", extended_open_positions);

        let pacifica_open_positions = get_pacifica_open_positions(&pacifica_wallet_address).await?;
        println!("Pacific Open Positions: {:?}", pacifica_open_positions);

        for i in 0..extended_open_positions.len() {
            let pacifica_market_index = extended_market_names
                .iter()
                .position(|p| *p == extended_open_positions[i].market)
                .unwrap();
            let result = close_if_necessary(
                &extended_open_positions[i].market,
                &pacifica_market_names[pacifica_market_index],
                &extended_open_positions[i],
                &pacifica_open_positions
                    .iter()
                    .find(|p| p.symbol == pacifica_market_names[pacifica_market_index])
                    .unwrap(),
                &extended_api_key,
                &extended_stark_private_key,
                &extended_vault_id,
                &extended_stark_public_key,
                &pacifica_private_key,
                &pacifica_wallet_address,
            )
            .await;

            match result {
                Ok(_) => println!(
                    "-------------------------------- Success for market {} --------------------------------",
                    extended_open_positions[i].market
                ),
                Err(e) => println!(
                    "-------------------------------- Error for market {} : {} --------------------------------",
                    extended_open_positions[i].market, e
                ),
            }
        }

        for i in 0..extended_market_names.len() {
            let result = place_arb_order(
                &extended_market_names[i],
                &pacifica_market_names[i],
                &extended_api_key,
                &extended_stark_private_key,
                &extended_vault_id,
                &extended_stark_public_key,
                &pacifica_private_key,
                &pacifica_wallet_address,
            )
            .await;

            match result {
                Ok(_) => println!(
                    "-------------------------------- Success for market {} --------------------------------",
                    extended_market_names[i]
                ),
                Err(e) => println!(
                    "-------------------------------- Error for market {} : {} --------------------------------",
                    extended_market_names[i], e
                ),
            }
        }
    }
}

async fn close_if_necessary(
    extended_market_name: &str,
    pacifica_market_name: &str,
    extended_open_position: &ExtendedOpenPositionData,
    pacifica_open_position: &PacificaOpenPositionData,
    extended_api_key: &str,
    extended_stark_private_key: &str,
    extended_vault_id: &str,
    extended_stark_public_key: &str,
    pacifica_private_key: &str,
    pacifica_wallet_address: &str,
) -> anyhow::Result<()> {
    println!(
        "Closing if necessary for market: {} and {}",
        extended_market_name, pacifica_market_name
    );
    let extended_result_vec = get_extended_market_data(extended_market_name).await?;
    let extended_result = extended_result_vec.first().unwrap();
    let pacifica_result = get_pacifica_market_data(pacifica_market_name).await?;

    let funding_rate_extended = extended_result.market_stats.funding_rate.parse::<f64>()? * 100.0;
    let funding_rate_pacifica = pacifica_result.next_funding.parse::<f64>()? * 100.0;

    if funding_rate_extended > funding_rate_pacifica {
        if extended_open_position.side == "LONG" {
            // Close long
            place_extended_order(
                &extended_market_name,
                &extended_result,
                ExtendedSide::Sell,
                extended_open_position.size.parse::<f64>()?,
                false,
                &extended_api_key,
                &extended_stark_private_key,
                &extended_vault_id,
                &extended_stark_public_key,
            )
            .await?;
        }

        if pacifica_open_position.side == "SHORT" {
            // Close short
            place_pacifica_order(
                pacifica_market_name,
                PacificaSide::Bid,
                pacifica_open_position.amount.parse::<f64>()?,
                &pacifica_result,
                false,
                &pacifica_private_key,
                &pacifica_wallet_address,
            )
            .await?;
        }
    } else {
        if extended_open_position.side == "SHORT" {
            // Close short
            place_extended_order(
                &extended_market_name,
                &extended_result,
                ExtendedSide::Buy,
                extended_open_position.size.parse::<f64>()?,
                false,
                &extended_api_key,
                &extended_stark_private_key,
                &extended_vault_id,
                &extended_stark_public_key,
            )
            .await?;
        }

        if pacifica_open_position.side == "LONG" {
            // Close long
            place_pacifica_order(
                pacifica_market_name,
                PacificaSide::Ask,
                pacifica_open_position.amount.parse::<f64>()?,
                &pacifica_result,
                false,
                &pacifica_private_key,
                &pacifica_wallet_address,
            )
            .await?;
        }
    }

    Ok(())
}

async fn place_arb_order(
    extended_market_name: &str,
    pacifica_market_name: &str,
    extended_api_key: &str,
    extended_stark_private_key: &str,
    extended_vault_id: &str,
    extended_stark_public_key: &str,
    pacifica_private_key: &str,
    pacifica_wallet_address: &str,
) -> anyhow::Result<()> {
    println!(
        "Checking funding arb for market: {} and {}",
        extended_market_name, pacifica_market_name
    );
    let extended_result_vec = get_extended_market_data(extended_market_name).await?;
    let extended_result = extended_result_vec.first().unwrap();
    let pacifica_result = get_pacifica_market_data(pacifica_market_name).await?;

    let funding_rate_extended = extended_result.market_stats.funding_rate.parse::<f64>()? * 100.0;
    let funding_rate_pacifica = pacifica_result.next_funding.parse::<f64>()? * 100.0;

    let price_extended = extended_result.market_stats.bid_price.parse::<f64>()?;
    let price_pacifica = pacifica_result.mid.parse::<f64>()?;

    let price_spread = if price_extended > price_pacifica {
        let price_diff = price_extended - price_pacifica;
        price_diff / price_extended * 100.0
    } else {
        let price_diff = price_pacifica - price_extended;
        price_diff / price_pacifica * 100.0
    };

    let funding_rate_diff = (funding_rate_extended - funding_rate_pacifica).abs();

    println!("Extended Funding Rate: {}", funding_rate_extended);
    println!("Pacific Funding Rate: {}", funding_rate_pacifica);
    println!("Price Spread: {}", price_spread);
    println!("Funding Rate Diff: {}", funding_rate_diff);

    let extended_tradeable_balance = get_extended_tradeable_balance(extended_api_key)
        .await?
        .available_for_trade
        .parse::<f64>()?;
    let pacifica_tradeable_balance = get_pacifica_tradeable_balance(pacifica_wallet_address)
        .await?
        .available_to_spend
        .parse::<f64>()?;

    let min_amount = (if extended_result.market_stats.bid_price.parse::<f64>()?
        < pacifica_result.mid.parse::<f64>()?
    {
        extended_result.market_stats.bid_price.parse::<f64>()?
    } else {
        pacifica_result.mid.parse::<f64>()?
    }) * 0.99;

    let tradeable_amount = BUY_AMOUNT / min_amount;

    if funding_rate_extended > funding_rate_pacifica {
        // SHORT on extended, LONG on pacifica
        if price_spread > PRICE_SPREAD_THRESHOLD || funding_rate_diff < FUNDING_RATE_THRESHOLD {
            return Err(anyhow::anyhow!(
                "Price Spread or Funding Rate Diff is too high"
            ));
        }

        if extended_tradeable_balance < BUY_AMOUNT || pacifica_tradeable_balance < BUY_AMOUNT {
            return Err(anyhow::anyhow!("Tradeable balance is too low"));
        }

        place_extended_order(
            &extended_market_name,
            &extended_result,
            ExtendedSide::Sell,
            tradeable_amount,
            true,
            &extended_api_key,
            &extended_stark_private_key,
            &extended_vault_id,
            &extended_stark_public_key,
        )
        .await?;
        let has_placed = place_pacifica_order(
            pacifica_market_name,
            PacificaSide::Bid,
            tradeable_amount,
            &pacifica_result,
            true,
            &pacifica_private_key,
            &pacifica_wallet_address,
        )
        .await;

        if has_placed.is_err() {
            place_extended_order(
                &extended_market_name,
                &extended_result,
                ExtendedSide::Buy,
                tradeable_amount,
                false,
                &extended_api_key,
                &extended_stark_private_key,
                &extended_vault_id,
                &extended_stark_public_key,
            )
            .await?;
            return Err(anyhow::anyhow!("Failed to place pacifica order"));
        }
    } else {
        if price_spread > PRICE_SPREAD_THRESHOLD || funding_rate_diff < FUNDING_RATE_THRESHOLD {
            return Err(anyhow::anyhow!(
                "Price Spread or Funding Rate Diff is too high"
            ));
        }

        if extended_tradeable_balance < BUY_AMOUNT || pacifica_tradeable_balance < BUY_AMOUNT {
            return Err(anyhow::anyhow!("Tradeable balance is too low"));
        }
        // LONG on extended, SHORT on pacifica
        place_extended_order(
            &extended_market_name,
            &extended_result,
            ExtendedSide::Buy,
            tradeable_amount,
            true,
            &extended_api_key,
            &extended_stark_private_key,
            &extended_vault_id,
            &extended_stark_public_key,
        )
        .await?;
        let has_placed = place_pacifica_order(
            pacifica_market_name,
            PacificaSide::Ask,
            tradeable_amount,
            &pacifica_result,
            true,
            &pacifica_private_key,
            &pacifica_wallet_address,
        )
        .await;

        if has_placed.is_err() {
            place_extended_order(
                &extended_market_name,
                &extended_result,
                ExtendedSide::Sell,
                tradeable_amount,
                false,
                &extended_api_key,
                &extended_stark_private_key,
                &extended_vault_id,
                &extended_stark_public_key,
            )
            .await?;
            return Err(anyhow::anyhow!("Failed to place extended order"));
        }
    }

    Ok(())
}
