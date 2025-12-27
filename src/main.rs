use dotenvy::dotenv;
use tokio::time::{Duration, interval};

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let mut interval = interval(Duration::from_secs(60 * 5));
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

    loop {
        interval.tick().await;

        let extended_open_positions = get_extended_open_positions().await?;
        println!("Extended Open Positions: {:?}", extended_open_positions);

        let pacifica_open_positions = get_pacifica_open_positions().await?;
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
            let result =
                place_arb_order(&extended_market_names[i], &pacifica_market_names[i]).await;

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
            )
            .await?;
        }
    }

    Ok(())
}

async fn place_arb_order(
    extended_market_name: &str,
    pacifica_market_name: &str,
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

    let extended_tradeable_balance = get_extended_tradeable_balance()
        .await?
        .available_for_trade
        .parse::<f64>()?;
    let pacifica_tradeable_balance = get_pacifica_tradeable_balance()
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

    let tradeable_amount = 50.0 / min_amount;

    if funding_rate_extended > funding_rate_pacifica {
        // SHORT on extended, LONG on pacifica
        if price_spread > PRICE_SPREAD_THRESHOLD || funding_rate_diff < FUNDING_RATE_THRESHOLD {
            return Err(anyhow::anyhow!(
                "Price Spread or Funding Rate Diff is too high"
            ));
        }

        if extended_tradeable_balance < 50.0 || pacifica_tradeable_balance < 50.0 {
            return Err(anyhow::anyhow!("Tradeable balance is too low"));
        }

        place_extended_order(
            &extended_market_name,
            &extended_result,
            ExtendedSide::Sell,
            tradeable_amount,
            true,
        )
        .await?;
        let has_placed = place_pacifica_order(
            pacifica_market_name,
            PacificaSide::Bid,
            tradeable_amount,
            &pacifica_result,
            true,
        )
        .await;

        if has_placed.is_err() {
            place_extended_order(
                &extended_market_name,
                &extended_result,
                ExtendedSide::Buy,
                tradeable_amount,
                false,
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

        if extended_tradeable_balance < 50.0 || pacifica_tradeable_balance < 50.0 {
            return Err(anyhow::anyhow!("Tradeable balance is too low"));
        }
        // LONG on extended, SHORT on pacifica
        place_extended_order(
            &extended_market_name,
            &extended_result,
            ExtendedSide::Buy,
            tradeable_amount,
            true,
        )
        .await?;
        let has_placed = place_pacifica_order(
            pacifica_market_name,
            PacificaSide::Ask,
            tradeable_amount,
            &pacifica_result,
            true,
        )
        .await;

        if has_placed.is_err() {
            place_extended_order(
                &extended_market_name,
                &extended_result,
                ExtendedSide::Sell,
                tradeable_amount,
                false,
            )
            .await?;
            return Err(anyhow::anyhow!("Failed to place extended order"));
        }
    }

    Ok(())
}
