use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarketInfo {
    pub status: String,
    pub data: Vec<MarketInfoData>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarketInfoData {
    pub market_stats: MarketStats,
    pub trading_config: TradingConfig,
    pub l2_config: L2Config,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarketStats {
    pub ask_price: String,
    pub bid_price: String,
    pub mark_price: String,
    pub last_price: String,
    pub index_price: String,
    pub funding_rate: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TradingConfig {
    pub min_order_size_change: String,
    pub max_position_value: String,
    pub min_price_change: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L2Config {
    pub collateral_id: String,
    pub synthetic_id: String,
    pub synthetic_resolution: u64,
    pub collateral_resolution: u64,
}

#[derive(Deserialize, Debug)]
pub struct OpenPosition {
    pub status: String,
    pub data: Vec<OpenPositionData>,
}

#[derive(Deserialize, Debug)]
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrder {
    pub id: String,
    pub market: String,
    pub side: Side,
    pub qty: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub price: String,
    pub reduce_only: bool,
    pub post_only: bool,
    pub time_in_force: String,
    pub expiry_epoch_millis: u64,
    pub fee: String,
    pub nonce: String,
    pub settlement: Settlement,
    pub self_trade_protection_level: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<TakeProfit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<StopLoss>,
    pub debugging_amounts: DebuggingAmounts,
}

#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderParams {
    pub order_hash: String,
    pub order_signature: Settlement,
    pub debug_amounts: DebuggingAmounts,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settlement {
    pub signature: Signature,
    pub stark_key: String,
    pub collateral_position: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub r: String,
    pub s: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TakeProfit {
    pub take_profit_type: String,
    pub take_profit_price: String,
    pub take_profit_time: u64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StopLoss {
    pub stop_loss_type: String,
    pub stop_loss_price: String,
    pub stop_loss_time: u64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DebuggingAmounts {
    pub collateral_amount: String,
    pub fee_amount: String,
    pub synthetic_amount: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeeResponse {
    pub status: String,
    pub data: Vec<FeeResponseData>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeeResponseData {
    pub market: String,
    pub maker_fee_rate: String,
    pub taker_fee_rate: String,
    pub builder_fee_rate: String,
}

#[derive(Deserialize, Debug)]
pub struct StarknetDomain {
    pub status: String,
    pub data: StarknetDomainData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StarknetDomainData {
    pub name: String,
    pub version: String,
    pub chain_id: String,
    pub revision: u64,
}

#[derive(Deserialize, Debug)]
pub struct OrderContext {
    pub asset_id_collateral: String,
    pub asset_id_synthetic: String,
    pub settlement_resolution_collateral: String,
    pub settlement_resolution_synthetic: String,
    pub min_order_size_change: String,
    pub max_position_value: String,
    pub fee_rate: String,
    pub vault_id: String,
    pub stark_private_key: String,
    pub starknet_domain: StarknetDomainData,
}

#[derive(Deserialize, Debug)]
pub struct PlaceOrderResponse {
    pub status: String,
}
