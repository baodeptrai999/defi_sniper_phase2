use crate::*;
use colored::*;

pub fn update_status_from_buy_event(
    mut token_data: TokenDatabaseSchema,
    buy_event: BuyEvent,
    tx_id: String,
) -> TokenDatabaseSchema {
    let updated_token_price = (buy_event.virtual_sol_reserves as f64 / 10f64.powi(9))
        / (buy_event.virtual_token_reserves as f64 / 10f64.powi(6));

    token_data.token_peak_price = token_data.token_peak_price.max(updated_token_price);
    token_data.token_price = updated_token_price;
    token_data.token_marketcap = updated_token_price * token_data.token_total_supply as f64;

    token_data.token_volume = if let Some(val) = token_data.token_volume {
        Some(val + buy_event.sol_amount as f64 / 10f64.powi(9))
    } else {
        None
    };

    token_data.last_event = LastEvent {
        tx_hash: tx_id.clone(),
        last_tracked_event: TokenEvent::BuyTokenEvent,
        last_activity_timestamp: buy_event.timestamp,
    };

    info!(
        "[{}]\t*Mint: {}\t*MC: {:.2} SOL\t{}",
        if token_data.token_is_purchased {
            "Detect-Holding"
        } else {
            "Detect-Waiting"
        },
        token_data.token_mint,
        token_data.token_marketcap,
        match token_data.token_volume {
            Some(val) => format!("*Volume: {:.4} SOL", val),
            None => "".to_string(),
        }
    );

    token_data.update_sell_state_flag(tx_id.clone());

    if buy_event.user == *SIGNER_PUBKEY {
        info!(
            "[My tx]\t[{}]\t*Hash: {}\t*mint: {}",
            "Buy".green(),
            tx_id,
            buy_event.mint.to_string()
        );
        token_data.token_is_purchased = true;
        token_data.token_buying_point_price = (buy_event.sol_amount as f64 / 10f64.powi(9))
            / (buy_event.token_amount as f64 / 10f64.powi(6));
        token_data.token_balance += buy_event.token_amount;

        token_data.tp_selling_plan = TPSellingPlan {
            tp_1: (*TAKE_PROFIT_1_PCNT * (token_data.token_balance as f64)) as u64,
            tp_2: (*TAKE_PROFIT_2_PCNT * (token_data.token_balance as f64)) as u64,
            tp_3: (*TAKE_PROFIT_3_PCNT * (token_data.token_balance as f64)) as u64,
            tp_4: (*TAKE_PROFIT_4_PCNT * (token_data.token_balance as f64)) as u64,
            tp_5: (*TAKE_PROFIT_5_PCNT * (token_data.token_balance as f64)) as u64,
        };

        token_data.ts_stop_selling_plan = TSStopSellingPlan {
            ts_1_stop: (*TS_1_SELL_PCNT * (token_data.token_balance as f64)) as u64,
            ts_2_stop: (*TS_2_SELL_PCNT * (token_data.token_balance as f64)) as u64,
            ts_3_stop: (*TS_3_SELL_PCNT * (token_data.token_balance as f64)) as u64,
            ts_4_stop: (*TS_4_SELL_PCNT * (token_data.token_balance as f64)) as u64,
            ts_5_stop: (*TS_5_SELL_PCNT * (token_data.token_balance as f64)) as u64,
        };

        update!(
            "Mint: {}\t*TSStopSellingPlan: {:#?}\t*TPSellingPlan {:#?}",
            buy_event.mint.to_string(),
            token_data.tp_selling_plan,
            token_data.ts_stop_selling_plan
        );
    }
    let _ = TOKEN_DB.upsert(buy_event.mint.clone(), token_data.clone());
    token_data.clone()
}

pub fn update_status_from_sell_event(
    mut token_data: TokenDatabaseSchema,
    sell_event: SellEvent,
    tx_id: String,
) -> Option<TokenDatabaseSchema> {
    let updated_token_price = (sell_event.virtual_sol_reserves as f64 / 10f64.powi(9))
        / (sell_event.virtual_token_reserves as f64 / 10f64.powi(6));

    token_data.token_peak_price = token_data.token_peak_price.max(updated_token_price);
    token_data.token_price = updated_token_price;
    token_data.token_marketcap = updated_token_price * token_data.token_total_supply as f64;

    token_data.token_volume = if let Some(val) = token_data.token_volume {
        Some(val + sell_event.sol_amount as f64 / 10f64.powi(9))
    } else {
        None
    };

    if *RUG_DETECT
        && token_data.last_event.tx_hash == tx_id
        && token_data.last_event.last_tracked_event == TokenEvent::BuyTokenEvent
    {
        token_data.bundle_tx_counter += 1;

        warning!(
            "{}\t*Tx: {}
            *Mint: {}\t*Bundle_counter: {}",
            "[Bundle tx]".yellow(),
            token_data.token_mint,
            solscan!(tx_id),
            token_data.bundle_tx_counter
        );
    }

    token_data.last_event = LastEvent {
        tx_hash: tx_id.clone(),
        last_tracked_event: TokenEvent::SellTokenEvent,
        last_activity_timestamp: sell_event.timestamp,
    };

    info!(
        "[{}]\t*Mint: {}\t*MC: {:.2} SOL\t{}",
        if token_data.token_is_purchased {
            "Detect-Holding"
        } else {
            "Detect-Waiting"
        },
        token_data.token_mint,
        token_data.token_marketcap,
        match token_data.token_volume {
            Some(val) => format!("*Volume: {:.4} SOL", val),
            None => "".to_string(),
        }
    );

    token_data.update_sell_state_flag(tx_id.clone());

    if sell_event.user == *SIGNER_PUBKEY {
        info!(
            "[My Tx]\t[{}]\t*Hash: {}\t*mint: {}",
            "Sell".red(),
            tx_id,
            sell_event.mint.to_string()
        );
        token_data.token_balance -= sell_event.token_amount;

        if token_data.token_balance > 0 {
            let _ = TOKEN_DB.upsert(sell_event.mint.clone(), token_data.clone());
            Some(token_data.clone())
        } else {
            let _ = TOKEN_DB.delete(sell_event.mint.clone());
            None
        }
    } else {
        if token_data.bundle_tx_counter >= *BUNDLE_TX_LIMIT && !token_data.token_is_purchased {
            warning!(
                "[RUG]\t*Stop tracking\t*Mint: {}\t*Bundle_counter: {}",
                token_data.token_mint,
                token_data.bundle_tx_counter
            );
            let _ = TOKEN_DB.delete(token_data.token_mint);
            None
        } else {
            let _ = TOKEN_DB.upsert(sell_event.mint.clone(), token_data.clone());
            Some(token_data.clone())
        }
    }
}

////////////////////////// copy mode //////////////////

pub fn update_status_from_buy_event_copy_mode(
    mut token_data: TokenDatabaseSchema,
    buy_event: BuyEvent,
    tx_id: String,
) -> TokenDatabaseSchema {
    let updated_token_price = (buy_event.virtual_sol_reserves as f64 / 10f64.powi(9))
        / (buy_event.virtual_token_reserves as f64 / 10f64.powi(6));

    token_data.token_peak_price = token_data.token_peak_price.max(updated_token_price);
    token_data.token_price = updated_token_price;
    token_data.token_marketcap = updated_token_price * token_data.token_total_supply as f64;

    token_data.token_volume = if let Some(val) = token_data.token_volume {
        Some(val + buy_event.sol_amount as f64 / 10f64.powi(9))
    } else {
        None
    };

    token_data.last_event = LastEvent {
        tx_hash: tx_id.clone(),
        last_tracked_event: TokenEvent::BuyTokenEvent,
        last_activity_timestamp: buy_event.timestamp,
    };

    info!(
        "[{}]\t*Mint: {}\t*MC: {:.2} SOL\t{}",
        if token_data.token_is_purchased {
            "Detect-Holding"
        } else {
            "Detect-Waiting"
        },
        token_data.token_mint,
        token_data.token_marketcap,
        match token_data.token_volume {
            Some(val) => format!("*Volume: {:.4} SOL", val),
            None => "".to_string(),
        }
    );

    token_data.update_sell_state_flag_copy_mode(tx_id.clone());

    if buy_event.user == *SIGNER_PUBKEY {
        info!(
            "[My tx]\t[{}]\t*Hash: {}\t*mint: {}",
            "Buy".green(),
            tx_id,
            buy_event.mint.to_string()
        );
        token_data.token_is_purchased = true;
        token_data.token_buying_point_price = (buy_event.sol_amount as f64 / 10f64.powi(9))
            / (buy_event.token_amount as f64 / 10f64.powi(6));
        token_data.token_balance += buy_event.token_amount;
    }
    let _ = TOKEN_DB.upsert(buy_event.mint.clone(), token_data.clone());
    token_data.clone()
}


pub fn update_status_from_sell_event_copy_mode(
    mut token_data: TokenDatabaseSchema,
    sell_event: SellEvent,
    tx_id: String,
) -> Option<TokenDatabaseSchema> {
    let updated_token_price = (sell_event.virtual_sol_reserves as f64 / 10f64.powi(9))
        / (sell_event.virtual_token_reserves as f64 / 10f64.powi(6));

    token_data.token_peak_price = token_data.token_peak_price.max(updated_token_price);
    token_data.token_price = updated_token_price;
    token_data.token_marketcap = updated_token_price * token_data.token_total_supply as f64;

    token_data.token_volume = if let Some(val) = token_data.token_volume {
        Some(val + sell_event.sol_amount as f64 / 10f64.powi(9))
    } else {
        None
    };

    if *RUG_DETECT
        && token_data.last_event.tx_hash == tx_id
        && token_data.last_event.last_tracked_event == TokenEvent::BuyTokenEvent
    {
        token_data.bundle_tx_counter += 1;

        warning!(
            "{}\t*Tx: {}
            *Mint: {}\t*Bundle_counter: {}",
            "[Bundle tx]".yellow(),
            token_data.token_mint,
            solscan!(tx_id),
            token_data.bundle_tx_counter
        );
    }

    token_data.last_event = LastEvent {
        tx_hash: tx_id.clone(),
        last_tracked_event: TokenEvent::SellTokenEvent,
        last_activity_timestamp: sell_event.timestamp,
    };

    info!(
        "[{}]\t*Mint: {}\t*MC: {:.2} SOL\t{}",
        if token_data.token_is_purchased {
            "Detect-Holding"
        } else {
            "Detect-Waiting"
        },
        token_data.token_mint,
        token_data.token_marketcap,
        match token_data.token_volume {
            Some(val) => format!("*Volume: {:.4} SOL", val),
            None => "".to_string(),
        }
    );

    token_data.update_sell_state_flag_copy_mode(tx_id.clone());

    if sell_event.user == *SIGNER_PUBKEY {
        info!(
            "[My Tx]\t[{}]\t*Hash: {}\t*mint: {}",
            "Sell".red(),
            tx_id,
            sell_event.mint.to_string()
        );
        token_data.token_balance -= sell_event.token_amount;

        if token_data.token_balance > 0 {
            let _ = TOKEN_DB.upsert(sell_event.mint.clone(), token_data.clone());
            Some(token_data.clone())
        } else {
            let _ = TOKEN_DB.delete(sell_event.mint.clone());
            None
        }
    } else {
        if token_data.bundle_tx_counter >= *BUNDLE_TX_LIMIT && !token_data.token_is_purchased {
            warning!(
                "[RUG]\t*Stop tracking\t*Mint: {}\t*Bundle_counter: {}",
                token_data.token_mint,
                token_data.bundle_tx_counter
            );
            let _ = TOKEN_DB.delete(token_data.token_mint);
            None
        } else {
            let _ = TOKEN_DB.upsert(sell_event.mint.clone(), token_data.clone());
            Some(token_data.clone())
        }
    }
}