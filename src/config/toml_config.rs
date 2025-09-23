use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WalletCredentialConfig {
    pub private_key: String,
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
    pub jito_api_key: String,
    pub nozomi_api_key: String,
    pub zero_slot_key: String,
}

#[derive(Debug, Deserialize)]
pub struct BuySetting {
    pub buy_amount_sol: f64,
}

#[derive(Debug, Deserialize)]
pub struct SlippageConfig {
    pub slippage: u32,
}

#[derive(Debug, Deserialize)]
pub struct FeeConfig {
    pub cu: u64,
    pub priority_fee_micro_lamport: u64,
    pub third_party_fee: f64,
}
