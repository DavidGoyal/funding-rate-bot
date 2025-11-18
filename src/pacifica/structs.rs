use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MarketInfo {
    pub success: bool,
    pub data: Vec<MarketInfoData>,
}

#[derive(Deserialize, Debug)]
pub struct MarketInfoData {
    pub mid: String,
    pub next_funding: String,
    pub symbol: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpenPosition {
    pub success: bool,
    pub data: Vec<OpenPositionData>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpenPositionData {
    pub symbol: String,
    pub side: String,
    pub amount: String,
    pub entry_price: String,
    pub margin: String,
    pub funding: String,
    pub isolated: bool,
    pub liquidation_price: String,
    pub created_at: u64,
    pub updated_at: u64,
}
