use once_cell::sync::Lazy;
use reqwest::Client;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    commitment_config::CommitmentLevel,
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

use crate::CONFIG;

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

//HTTP endpoint
pub static ZERO_SLOT_HTTP_CLIENT: Lazy<Arc<Client>> = Lazy::new(|| {
    println!("🔄 Initializing 0-slot HTTP client...");

    let client = Client::builder()
        .pool_idle_timeout(Duration::from_secs(300))
        .pool_max_idle_per_host(5)
        .tcp_keepalive(Duration::from_secs(10))
        .tcp_nodelay(true)
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(10))
        .http2_keep_alive_interval(Duration::from_secs(20))
        .http2_keep_alive_timeout(Duration::from_secs(90))
        .http2_keep_alive_while_idle(true)
        .use_rustls_tls()
        .build()
        .expect("Failed to build 0-slot HTTP client");

    Arc::new(client)
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

//Buy setting
pub static BUY_AMOUNT_SOL: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.buy_amount_sol);

//Slippage
pub static SLIPPAGE: Lazy<f64> =
    Lazy::new(|| 1.0 + CONFIG.slippage_config.slippage_percent as f64 / 100.0);

//Fee
pub static PRIORITY_FEE: Lazy<(u64, u64, f64)> = Lazy::new(|| {
    let cu: u64 = CONFIG.fee_config.cu;
    let priority_fee_micro_lamport = CONFIG.fee_config.priority_fee_micro_lamport;

    let third_party_fee = CONFIG.fee_config.third_party_fee;

    (cu, priority_fee_micro_lamport, third_party_fee)
});

pub async fn pre_warm_zero_slot_endpoint(client: Arc<Client>) {
    println!("🔥 Pre-warming 0-slot endpoint...");

    for attempt in 1..=3 {
        let url = "http://la1.0slot.trade?api-key=335e371309b6492584368e9dc553622d".to_string();

        match client.get(&url).send().await {
            Ok(response) => {
                println!(
                    "✅ 0-slot endpoint ready (attempt {}): HTTP {}",
                    attempt,
                    response.status()
                );

                if response.status().is_success() {
                    println!("🎯 Successfully connected to 0-slot service");
                }
                break;
            }
            Err(e) if attempt < 3 => {
                println!("⚠️ 0-slot warm-up attempt {} failed: {:?}", attempt, e);
                tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
            }
            Err(e) => {
                eprintln!("❌ Failed to pre-warm 0-slot endpoint: {:?}", e);
            }
        }
    }
}

pub fn get_zero_slot_client() -> Arc<Client> {
    ZERO_SLOT_HTTP_CLIENT.clone()
}
