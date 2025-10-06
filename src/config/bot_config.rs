use crate::*;
use lazy_static::lazy_static;
use std::sync::Arc;

use once_cell::sync::Lazy;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    commitment_config::CommitmentLevel,
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use colored::*;
use console::Emoji;

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

//Target wallets
pub static TARGET_WALLETS: Lazy<Vec<String>> =
    Lazy::new(|| CONFIG.target_config.target_wallets.clone());

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

//Confirm service
pub static CONFIRM_SERVICE: Lazy<String> =
    Lazy::new(|| CONFIG.relayer_config.confirm_service.clone());
pub static JITO_API_KEY: Lazy<String> = Lazy::new(|| CONFIG.relayer_config.jito_api_key.clone());
pub static NOZOMI_API_KEY: Lazy<String> =
    Lazy::new(|| CONFIG.relayer_config.nozomi_api_key.clone());
pub static ZERO_SLOT_API_KEY: Lazy<String> =
    Lazy::new(|| CONFIG.relayer_config.zero_slot_key.clone());

//Buy setting
pub static BUY_AMOUNT_SOL: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.buy_amount_sol);

//Slippage
pub static SLIPPAGE: Lazy<f64> =
    Lazy::new(|| 1.0 + CONFIG.slippage_config.slippage_percent as f64 / 100.0);
pub static HALF_COPY_PCNT_MODE: Lazy<bool> = Lazy::new(|| CONFIG.buy_setting.half_copy_pcnt_mode);
pub static BUY_AMOUNT_PERCENT: Lazy<u32> = Lazy::new(|| CONFIG.buy_setting.buy_amount_percent);

//Fee
pub static PRIORITY_FEE: Lazy<(u64, u64, f64)> = Lazy::new(|| {
    let cu: u64 = CONFIG.fee_config.cu;
    let priority_fee_micro_lamport = CONFIG.fee_config.priority_fee_micro_lamport;

    let third_party_fee = CONFIG.fee_config.third_party_fee;

    (cu, priority_fee_micro_lamport, third_party_fee)
});

//Filter
pub static BLACK_LIST_FILTER: Lazy<bool> = Lazy::new(|| CONFIG.filter_setting.black_list_filter);
pub static WALLET_BLACKLIST_PATH: Lazy<String> =
    Lazy::new(|| CONFIG.filter_setting.wallet_blacklist_path.clone());
pub static TOKEN_BLACKLIST_PATH: Lazy<String> =
    Lazy::new(|| CONFIG.filter_setting.rug_token_blacklist_path.clone());

pub static WALLET_BLACKLIST: Lazy<Vec<String>> =
    Lazy::new(|| BlackList::get_blacklist(&*WALLET_BLACKLIST_PATH));
pub static TOKEN_BLACKLIST: Lazy<Vec<String>> =
    Lazy::new(|| BlackList::get_blacklist(&*TOKEN_BLACKLIST_PATH));

pub static RUG_DETECT: Lazy<bool> = Lazy::new(|| CONFIG.filter_setting.rug_detect);
pub static BUNDLE_TX_LIMIT: Lazy<i32> = Lazy::new(|| CONFIG.filter_setting.bundle_tx_limit);

pub static VOLUME_FILTER: Lazy<bool> = Lazy::new(|| CONFIG.filter_setting.volume_filter);
pub static MIN_VOLUME_LIMIT_SOL: Lazy<i32> =
    Lazy::new(|| CONFIG.filter_setting.min_volume_limit_sol);

pub static MARKET_CAP_FILTER: Lazy<bool> = Lazy::new(|| CONFIG.filter_setting.market_cap_filter);
pub static MIN_MARKET_CAP_LIMIT_SOL: Lazy<i32> =
    Lazy::new(|| CONFIG.filter_setting.min_market_cap_limit_sol);

pub static MAX_TOKEN_HOLDER_FILTER: Lazy<bool> =
    Lazy::new(|| CONFIG.filter_setting.max_token_holder_filter);
pub static MAX_TOKEN_HOLDER_LIMIT: Lazy<u64> =
    Lazy::new(|| CONFIG.filter_setting.max_token_holder_limit);

//Stop monitor
pub static STOP_NO_ACTIVITY_TOKEN_MONITORING: Lazy<bool> =
    Lazy::new(|| CONFIG.monitor_setting.stop_no_activity_token_monitoring);
pub static NO_ACTIVITY_TIME: Lazy<i64> = Lazy::new(|| CONFIG.monitor_setting.no_activity_time);

// [shut_down_setting]
pub static AUTO_SHUT_DOWN: Lazy<bool> = Lazy::new(|| CONFIG.shut_down_setting.auto_shut_down);
pub static SHUT_DOWN_TIMER_SELL_ALL: Lazy<bool> =
    Lazy::new(|| CONFIG.shut_down_setting.shut_down_sell_all);
pub static SHUT_DOWN_TIME: Lazy<String> =
    Lazy::new(|| CONFIG.shut_down_setting.shut_down_time.clone());

lazy_static! {
    pub static ref AUTO_TURNOFF: AtomicBool = AtomicBool::new(false);
}

pub fn show_bot_settings() {
    log!("Public key: {:?}", *SIGNER_PUBKEY);
    log!("Confirm service: {:?}", *CONFIRM_SERVICE);
    log!("Buy settings: {:?}", CONFIG.buy_setting);
    log!("Slippage: {:?}%", *SLIPPAGE);
    log!("Grpc endpoint: {:?}", *GRPC_ENDPOINT);
    log!("Grpc token: {:?}", *GRPC_TOKEN);
    log!("RPC endpoint: {:?}", *RPC_ENDPOINT);
    log!("Blacklist filter: {:?}", *BLACK_LIST_FILTER);
    log!("Rug detect: {:?}", *RUG_DETECT);
    log!("Bundle tx limit: {:?}", *BUNDLE_TX_LIMIT);
    log!("Volume filter: {:?}", *VOLUME_FILTER);
    log!("Min volume limit: {:?} SOL", *MIN_VOLUME_LIMIT_SOL);
    log!("Marketcap filter: {:?}", *MARKET_CAP_FILTER);
    log!("Min marketcap limit: {:?} SOL", *MIN_MARKET_CAP_LIMIT_SOL);
    log!(
        "Stop no activity token monitoring: {:?}",
        *STOP_NO_ACTIVITY_TOKEN_MONITORING
    );
    log!("No activity time: {:?} seconds", *NO_ACTIVITY_TIME);

    init_validator();

    log!(
        "TAKE_PROFIT_1 : {:<5.3} % , TAKE_PROFIT_2 : {:<5.3} % , TAKE_PROFIT_3 : {:<5.3} % , TAKE_PROFIT_4 : {:<5.3} % , TAKE_PROFIT_5 : {:<5.3} % , SL : {:<5.3} %",
        *TAKE_PROFIT_1 * 100.0,
        *TAKE_PROFIT_2 * 100.0,
        *TAKE_PROFIT_3 * 100.0,
        *TAKE_PROFIT_4 * 100.0,
        *TAKE_PROFIT_5 * 100.0,
        *STOP_LOSS * 100.0
    );
    log!(
        "TS_1 : {:<5.3} %, TS_1_STOP : {:<5.3} %, TS_1_SELL_PCNT : {:<5.3} %",
        *TS_1 * 100.0,
        *TS_1 * (1.0 - *TS_1_STOP) * 100.0,
        *TS_1_SELL_PCNT * 100.0
    );
    log!(
        "TS_2 : {:<5.3} %, TS_2_STOP : {:<5.3} %, TS_2_SELL_PCNT : {:<5.3} %",
        *TS_2 * 100.0,
        *TS_2 * (1.0 - *TS_2_STOP) * 100.0,
        *TS_2_SELL_PCNT * 100.0
    );
    log!(
        "TS_3 : {:<5.3} %, TS_3_STOP : {:<5.3} %, TS_3_SELL_PCNT : {:<5.3} %",
        *TS_3 * 100.0,
        *TS_3 * (1.0 - *TS_3_STOP) * 100.0,
        *TS_3_SELL_PCNT * 100.0
    );
    log!(
        "TS_4 : {:<5.3} %, TS_4_STOP : {:<5.3} %, TS_4_SELL_PCNT : {:<5.3} %",
        *TS_4 * 100.0,
        *TS_4 * (1.0 - *TS_4_STOP) * 100.0,
        *TS_4_SELL_PCNT * 100.0
    );
    log!(
        "TS_5 : {:<5.3} %, TS_5_STOP : {:<5.3} %, TS_5_SELL_PCNT : {:<5.3} %",
        *TS_5 * 100.0,
        *TS_5 * (1.0 - *TS_5_STOP) * 100.0,
        *TS_5_SELL_PCNT * 100.0
    );

    println!(
        "{} {}",
        Emoji("\n💳", ""),
        "Loading wallet blacklist...".green()
    );

    for blacklisted_wallet in WALLET_BLACKLIST.iter() {
        println!("- {}", blacklisted_wallet.red());
    }
    println!("Loaded {} blacked wallets.\n", WALLET_BLACKLIST.len());

    println!(
        "{} {}",
        Emoji("💱", ""),
        "Loading token blacklist...".yellow()
    );

    for blacklisted_wallet in TOKEN_BLACKLIST.iter() {
        println!("- {}", blacklisted_wallet.red());
    }
    println!("Loaded {} blacked tokens.\n", TOKEN_BLACKLIST.len());
}
