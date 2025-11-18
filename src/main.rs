use dotenvy::dotenv;
use tokio::time::{Duration, interval};

use crate::{
    extended::{
        get_market_data::get_extended_market_data,
        get_open_positions::{get_extended_open_position, get_extended_open_positions},
        structs::OpenPositionData as ExtendedOpenPositionData,
    },
    pacifica::{
        get_market_data::get_pacifica_market_data,
        get_open_positions::{get_pacifica_open_position, get_pacifica_open_positions},
        structs::OpenPositionData as PacificaOpenPositionData,
    },
};

mod extended;
mod pacifica;

const FUNDING_RATE_THRESHOLD: f64 = 0.001;
const PRICE_SPREAD_THRESHOLD: f64 = 0.01;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let mut interval = interval(Duration::from_secs(30));
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

        for i in 0..extended_market_names.len() {
            let result = fetch_market_info(
                &extended_market_names[i],
                &pacifica_market_names[i],
                &extended_open_positions,
                &pacifica_open_positions,
            )
            .await;

            match result {
                Ok(_) => println!("Success for market {}", extended_market_names[i]),
                Err(e) => println!("Error for market {}: {}", extended_market_names[i], e),
            }
        }
    }
}

async fn fetch_market_info(
    extended_market_name: &str,
    pacifica_market_name: &str,
    extended_open_positions: &Vec<ExtendedOpenPositionData>,
    pacifica_open_positions: &Vec<PacificaOpenPositionData>,
) -> anyhow::Result<()> {
    let extended_result = get_extended_market_data(extended_market_name).await?;
    let pacifica_result = get_pacifica_market_data(pacifica_market_name).await?;

    let funding_rate_extended = extended_result.funding_rate.parse::<f64>()? * 100.0;
    let funding_rate_pacifica = pacifica_result.next_funding.parse::<f64>()? * 100.0;

    let price_extended = extended_result.ask_price.parse::<f64>()?;
    let price_pacifica = pacifica_result.mid.parse::<f64>()?;

    let mut price_spread = 0.0;

    if price_extended > price_pacifica {
        let price_diff = price_extended - price_pacifica;
        price_spread = price_diff / price_extended * 100.0;
    } else {
        let price_diff = price_pacifica - price_extended;
        price_spread = price_diff / price_pacifica * 100.0;
    }

    let funding_rate_diff = (funding_rate_extended - funding_rate_pacifica).abs();

    if price_spread > PRICE_SPREAD_THRESHOLD || funding_rate_diff < FUNDING_RATE_THRESHOLD {
        //TODO: Check  if open order on current market if yes then close
        println!("Price Spread: {}", price_spread);
        println!("Funding Rate Diff: {}", funding_rate_diff);

        return Err(anyhow::anyhow!(
            "Price Spread or Funding Rate Diff is too high"
        ));
    }

    println!("Extended Funding Rate: {}", funding_rate_extended);
    println!("Pacific Funding Rate: {}", funding_rate_pacifica);
    println!("Price Spread: {}", price_spread);

    let extended_open_position =
        get_extended_open_position(extended_market_name, extended_open_positions).await;
    let pacifica_open_position =
        get_pacifica_open_position(pacifica_market_name, pacifica_open_positions).await;

    if funding_rate_extended > funding_rate_pacifica {
        // SHORT on extended, LONG on pacifica
        // This works for all cases
    } else {
        // LONG on extended, SHORT on pacifica
        // This works for all cases
    }

    Ok(())
}
