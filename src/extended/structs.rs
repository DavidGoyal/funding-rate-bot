use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MarketInfo {
    pub status: String,
    pub data: MarketInfoData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarketInfoData {
    pub daily_volume: String,
    pub ask_price: String,
    pub funding_rate: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpenPosition {
    pub status: String,
    pub data: Vec<OpenPositionData>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenPositionData {
    pub id: u64,
    pub account_id: u64,
    pub market: String,
    pub side: String,
    pub leverage: String,
    pub size: String,
    pub value: String,
    pub open_price: String,
    pub mark_price: String,
    pub liquidation_price: String,
    pub margin: String,
    pub unrealised_pnl: String,
    pub realised_pnl: String,
    pub tp_trigger_price: Option<String>,
    pub tp_limit_price: Option<String>,
    pub sl_trigger_price: Option<String>,
    pub sl_limit_price: Option<String>,
    pub adl: u32,
    pub max_position_size: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}
