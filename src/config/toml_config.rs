use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ModeConfig {
    pub is_dev_mode: bool,
    pub buy_tx_counter: i32,
}

#[derive(Debug, Deserialize)]
pub struct WalletCredentialConfig {
    pub private_key: String,
}

#[derive(Debug, Deserialize)]
pub struct TargetConfig {
    pub target_wallets: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectionConfig {
    pub grpc_endpoint: String,
    pub grpc_token: String,
    pub rpc_endpoint: String,
}

#[derive(Debug, Deserialize)]
pub struct RelayerConfig {
    pub confirm_service: String,
}

#[derive(Debug, Deserialize)]
pub struct BuySetting {
    pub buy_amount_sol: f64,
}
#[derive(Debug, Deserialize)]
pub struct BuyConditionConfig {
    pub price_variant_width_percent: f64,
}

#[derive(Debug, Deserialize)]
pub struct SellSetting {
    pub stop_loss: f64,
    pub real_tp_multiply: f64,
    pub trailing_1: f64,
    pub trailing_1_stop: f64,
    pub trailing_1_sell_percentage: f64,
    pub trailing_2: f64,
    pub trailing_2_stop: f64,
    pub trailing_2_sell_percentage: f64,
    pub trailing_3: f64,
    pub trailing_3_stop: f64,
    pub trailing_3_sell_percentage: f64,
    pub trailing_4: f64,
    pub trailing_4_stop: f64,
    pub trailing_4_sell_percentage: f64,
    pub trailing_5: f64,
    pub trailing_5_stop: f64,
    pub trailing_5_sell_percentage: f64,
}

#[derive(Debug, Deserialize)]
pub struct SlippageConfig {
    pub slippage_percent: u32,
}

#[derive(Debug, Deserialize)]
pub struct FeeConfig {
    pub cu: u64,
    pub priority_fee_micro_lamport: u64,
    pub third_party_fee: f64,
}

#[derive(Debug, Deserialize)]
pub struct FilterSetting {
    pub rug_detect: bool,
    pub bundle_tx_limit: i32,
    pub volume_filter: bool,
    pub min_volume_limit_sol: i32,
    pub market_cap_filter: bool,
    pub min_market_cap_limit_sol: i32,
    pub max_token_holder_filter: bool,
    pub max_token_holder_limit: u64,
}
#[derive(Debug, Deserialize)]
pub struct MonitorConfig {
    pub stop_no_activity_token_monitoring: bool,
    pub no_activity_time: u64,
}
