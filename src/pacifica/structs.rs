use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct MarketPricesInfo {
    pub success: bool,
    pub data: Vec<MarketPricesInfoData>,
}

#[derive(Deserialize, Debug)]
pub struct MarketPricesInfoData {
    pub mid: String,
    pub next_funding: String,
    pub symbol: String,
}

#[derive(Deserialize, Debug)]
pub struct MarketInfoData {
    pub mid: String,
    pub next_funding: String,
    pub symbol: String,
    pub tick_size: String,
    pub min_tick: String,
    pub max_tick: String,
    pub lot_size: String,
    pub min_order_size: String,
    pub max_order_size: String,
}

#[derive(Deserialize, Debug)]
pub struct MarketTradingInfo {
    pub success: bool,
    pub data: Vec<MarketTradingInfoData>,
}

#[derive(Deserialize, Debug)]
pub struct MarketTradingInfoData {
    pub symbol: String,
    pub tick_size: String,
    pub min_tick: String,
    pub max_tick: String,
    pub lot_size: String,
    pub min_order_size: String,
    pub max_order_size: String,
}

#[derive(Deserialize, Debug)]
pub struct OpenPosition {
    pub success: bool,
    pub data: Vec<OpenPositionData>,
}

#[derive(Deserialize, Debug)]
pub struct OpenPositionData {
    pub symbol: String,
    pub side: String,
    pub amount: String,
    pub entry_price: String,
    pub margin: String,
    pub funding: String,
    pub isolated: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Deserialize, Debug)]
pub struct SignatureHeader {
    pub timestamp: u64,
    pub expiry_window: u64,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlaceOrder {
    pub account: String,
    pub agent_wallet: String,
    pub signature: String,
    pub timestamp: u64,
    pub expiry_window: u64,
    pub symbol: String,
    pub side: Side,
    pub reduce_only: bool,
    pub amount: String,
    pub slippage_percent: String,
    pub client_order_id: String,
    pub take_profit: Option<TakeProfit>,
    pub stop_loss: Option<StopLoss>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignaturePayload {
    pub symbol: String,
    pub side: Side,
    pub reduce_only: bool,
    pub amount: String,
    pub slippage_percent: String,
    pub client_order_id: String,
    pub take_profit: Option<TakeProfit>,
    pub stop_loss: Option<StopLoss>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TakeProfit {
    pub stop_price: String,
    pub client_order_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StopLoss {
    pub stop_price: String,
    pub client_order_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedMessage {
    pub timestamp: u64,
    pub expiry_window: u64,
    #[serde(rename = "type")]
    pub r#type: String,
    pub data: SignaturePayload,
}

impl SignedMessage {
    pub fn into_string(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TradeableBalance {
    pub success: bool,
    pub data: TradeableBalanceData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TradeableBalanceData {
    pub balance: String,
    pub available_to_spend: String,
}
