use crate::*;
use once_cell::sync::Lazy;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    commitment_config::CommitmentLevel,
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

//Bot mode
pub static DEV_MODE: Lazy<bool> = Lazy::new(|| CONFIG.mode.is_dev_mode);
pub static BUY_TX_COUNTER: Lazy<AtomicI32> =
    Lazy::new(|| AtomicI32::new(CONFIG.mode.buy_tx_counter));

pub fn decrese_buy_tx_remain_counter() {
    BUY_TX_COUNTER.fetch_sub(1, Ordering::SeqCst);
}

pub fn get_buy_tx_remain_counter() -> i32 {
    BUY_TX_COUNTER.load(Ordering::SeqCst)
}

//Wallet key
pub static SIGNER_KEYPAIR: Lazy<Keypair> = Lazy::new(|| {
    let wallet: Keypair = Keypair::from_base58_string(&CONFIG.wallet_config.private_key);
    wallet
});

pub static SIGNER_PUBKEY: Lazy<Pubkey> = Lazy::new(|| {
    let wallet: Keypair = Keypair::from_base58_string(&CONFIG.wallet_config.private_key);
    wallet.pubkey()
});

//RPC endpoint
pub static RPC_ENDPOINT: Lazy<String> = Lazy::new(|| CONFIG.connection_config.rpc_endpoint.clone());
pub static RPC_CLIENT: Lazy<Arc<RpcClient>> = Lazy::new(|| {
    Arc::new(RpcClient::new_with_commitment(
        CONFIG.connection_config.rpc_endpoint.clone(),
        CommitmentConfig {
            commitment: CommitmentLevel::Processed,
        },
    ))
});
pub static GRPC_ENDPOINT: Lazy<String> =
    Lazy::new(|| CONFIG.connection_config.grpc_endpoint.clone());
pub static GRPC_TOKEN: Lazy<String> = Lazy::new(|| CONFIG.connection_config.grpc_token.clone());
pub static BNB_RPC_ENDPOINT: Lazy<String> =
    Lazy::new(|| CONFIG.connection_config.bnb_rpc_endpoint.clone());

//Buy setting
pub static BUY_AMOUNT_SOL: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.buy_amount_sol);
pub static DYNAMIC_BUY_AMOUNT_MODE: Lazy<bool> = Lazy::new(|| CONFIG.buy_setting.dynamic_buy_amount_mode);
pub static LOSS_SEQUENCE: Lazy<u32> = Lazy::new(|| CONFIG.buy_setting.loss_sequence.max(1));
pub static PROFIT_SEQUENCE: Lazy<u32> = Lazy::new(|| CONFIG.buy_setting.profit_sequence.max(1));
pub static LOSS_MULTIPLY: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.loss_multiply.clamp(0.01, 1.0));
pub static PROFIT_MULTIPLY: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.profit_multiply.max(1.0));
pub static MAX_BUY_AMOUNT_MULTIPLY: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.max_buy_amount_multiply.max(0.01));
pub static MIN_BUY_AMOUNT_MULTIPLY: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.min_buy_amount_multiply.clamp(0.001, 1.0));

//Slippage
pub static SLIPPAGE: Lazy<f64> =
    Lazy::new(|| 1.0 + CONFIG.slippage_config.slippage_percent as f64 / 100.0);

//Landing service config
pub static LANDING_SERVICE: Lazy<String> =
    Lazy::new(|| CONFIG.landing_service_config.landing_service.clone());
pub static ZERO_SLOT_API_KEY: Lazy<String> = Lazy::new(|| CONFIG.landing_service_config.zero_slot_api_key.clone());
pub static HELIUS_API_KEY: Lazy<String> = Lazy::new(|| CONFIG.landing_service_config.helius_api_key.clone());

pub static ZERO_SLOT_ENDPOINT: Lazy<String> = Lazy::new(|| format!("http://de1.0slot.trade?api-key={}", *ZERO_SLOT_API_KEY));
pub static HELIUS_ENDPOINT: Lazy<String> = Lazy::new(|| "http://fra-sender.helius-rpc.com/fast".to_string());

//Fee
pub static BUY_COMPUTE_UNIT_LIMIT: Lazy<u64> =
    Lazy::new(|| CONFIG.fee_config.buy_compute_unit_limit);
pub static BUY_MICRO_LAMPORTS: Lazy<u64> = Lazy::new(|| CONFIG.fee_config.buy_micro_lamports);
pub static SELL_MICRO_LAMPORTS: Lazy<f64> = Lazy::new(|| CONFIG.fee_config.sell_micro_lamports);
pub static ZERO_SLOT_FEE: Lazy<f64> = Lazy::new(|| CONFIG.fee_config.zero_slot_fee);
pub static HELIUS_FEE: Lazy<f64> = Lazy::new(|| CONFIG.fee_config.helius_fee);