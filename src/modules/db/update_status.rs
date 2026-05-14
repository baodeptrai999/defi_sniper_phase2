use crate::*;
use std::time::Instant;

pub fn update_status_from_pumpfun_buy_event(
    mut token_data: TokenDatabaseSchema,
    buy_event: PumpfunBuyEvent,
    tx_id: String,
) -> TokenDatabaseSchema {
    let updated_token_price = (buy_event.virtual_sol_reserves as f64 / 10f64.powi(9))
        / (buy_event.virtual_token_reserves as f64 / 10f64.powi(6));

    token_data.token_max_price = token_data.token_max_price.max(updated_token_price);
    token_data.token_price = updated_token_price;
    token_data.token_creator = buy_event.creator;
    token_data.token_last_activity_time = Instant::now();

    if buy_event.user == *SIGNER_PUBKEY {
        info!(
            "[My Tx]\t[{}]\t*Hash: {}\t*Mint: {}",
            "Buy".green(),
            tx_id,
            buy_event.mint
        );
        token_data.token_is_purchased = true;
        token_data.token_balance += buy_event.token_amount;
        token_data.token_buying_point_price = updated_token_price;
        token_data.token_sell_status = TokenSellStatus::None;
        token_data.initialize_sell_plan_if_needed();
    } else if buy_event.user == token_data.token_creator {
        if token_data.dev_buy_sol_lamports == None {
            token_data.dev_buy_sol_lamports = Some(buy_event.sol_amount);
        };
    }

    //update sell state flag
    token_data.update_sell_state_flag(tx_id.clone());

    let _ = TOKEN_DB.upsert(buy_event.mint.clone(), token_data.clone());
    token_data.clone()
}

pub fn update_status_from_pumpfun_sell_event(
    mut token_data: TokenDatabaseSchema,
    sell_event: PumpfunSellEvent,
    tx_id: String,
) -> Option<TokenDatabaseSchema> {
    let updated_token_price = (sell_event.virtual_sol_reserves as f64 / 10f64.powi(9))
        / (sell_event.virtual_token_reserves as f64 / 10f64.powi(6));

    token_data.token_max_price = token_data.token_max_price.max(updated_token_price);
    token_data.token_price = updated_token_price;
    token_data.token_creator = sell_event.creator;
    token_data.token_last_activity_time = Instant::now();

    //update sell state flag
    token_data.update_sell_state_flag(tx_id.clone());

    if sell_event.user == *SIGNER_PUBKEY {
        info!(
            "[My Tx]\t[{}]\t*Hash: {}\t*Mint: {}",
            "Sell".red(),
            tx_id,
            sell_event.mint.to_string()
        );
        token_data.token_balance -= sell_event.token_amount;
        token_data.token_sell_status = TokenSellStatus::None;
        token_data.pending_tp_sell_index = None;
        token_data.pending_tp_sell_amount = 0;
        token_data.tp_trailing_active = false;
        token_data.tp_trailing_max_price = 0.0;

        if token_data.token_balance > 0 {
            token_data.next_tp_index_to_sell = token_data.next_tp_index_to_sell.saturating_add(1);
            let _ = TOKEN_DB.upsert(sell_event.mint.clone(), token_data.clone());
            Some(token_data.clone())
        } else {
            let is_profit = token_data.token_price > token_data.token_buying_point_price;
            DYNAMIC_BUY.record_outcome(&token_data.matched_pattern_label, &sell_event.mint.to_string(), is_profit);
            let _ = TOKEN_DB.delete(sell_event.mint.clone());
            None
        }
    } else {
        let _ = TOKEN_DB.upsert(sell_event.mint.clone(), token_data.clone());
        Some(token_data.clone())
    }
}

///Migration data handler

pub fn update_status_from_migration_event(
    mut token_data: TokenDatabaseSchema,
    create_pool_accounts: CreatePoolInstructionAccounts,
    create_pool_event_data: CreatePoolEventData,
    tx_id: String,
) -> TokenDatabaseSchema {
    info!("[MIGRATED]: {}", token_data.token_mint);
    let updated_token_price = (create_pool_event_data.quote_amount_in as f64 / 10f64.powi(9))
        / (create_pool_event_data.base_amount_in as f64 / 10f64.powi(6));

    token_data.token_price = updated_token_price;
    token_data.token_max_price = token_data.token_max_price.max(updated_token_price);
    token_data.token_is_migrated = true;
    token_data.token_creator = create_pool_event_data.coin_creator;
    token_data.token_last_activity_time = Instant::now();

    token_data.pumpswap_struct = Some(PumpSwapStruct::from_migrate(
        &create_pool_accounts,
        create_pool_event_data,
    ));

    token_data.update_sell_state_flag(tx_id.clone());

    let _ = TOKEN_DB.upsert(token_data.token_mint.clone(), token_data.clone());
    token_data
}

////Pumpswap trade data handler
pub fn update_status_from_pumpswap_buy_event(
    mut token_data: TokenDatabaseSchema,
    buy_event: PumpswapBuyEvent,
    buy_accounts: PumpswapBuyInstructionAccounts,
    tx_id: String,
) -> TokenDatabaseSchema {
    let updated_token_price = (buy_event.pool_quote_token_reserves as f64 / 10f64.powi(9))
        / (buy_event.pool_base_token_reserves as f64 / 10f64.powi(6));

    token_data.token_max_price = token_data.token_max_price.max(updated_token_price);

    token_data.token_creator = buy_event.coin_creator;
    token_data.token_price = updated_token_price;
    token_data.token_last_activity_time = Instant::now();

    token_data.update_sell_state_flag(tx_id.clone());

    if buy_event.user == *SIGNER_PUBKEY {
        info!(
            "[My tx]\t[{}]\t*Hash: {}\t*mint: {}",
            "Buy".green(),
            tx_id,
            buy_accounts.base_mint.to_string()
        );

        token_data.token_is_purchased = true;
        token_data.token_buying_point_price = updated_token_price;
        token_data.token_balance += buy_event.base_amount_out;
        token_data.token_sell_status = TokenSellStatus::None;
        token_data.initialize_sell_plan_if_needed();
    }

    let _ = TOKEN_DB.upsert(token_data.token_mint.clone(), token_data.clone());
    token_data
}

pub fn update_status_from_pumpswap_sell_event(
    mut token_data: TokenDatabaseSchema,
    sell_event: PumpswapSellEvent,
    sell_accounts: PumpswapSellInstructionAccounts,
    tx_id: String,
) -> Option<TokenDatabaseSchema> {
    let updated_token_price = (sell_event.pool_quote_token_reserves as f64 / 10f64.powi(9))
        / (sell_event.pool_base_token_reserves as f64 / 10f64.powi(6));

    token_data.token_creator = sell_event.coin_creator;
    token_data.token_price = updated_token_price;
    token_data.token_last_activity_time = Instant::now();

    token_data.update_sell_state_flag(tx_id.clone());

    if sell_event.user == *SIGNER_PUBKEY {
        info!(
            "[My Tx]\t[{}]\t*Hash: {}\t*mint: {}",
            "Sell".red(),
            tx_id,
            sell_accounts.base_mint.to_string()
        );

        token_data.token_balance -= sell_event.base_amount_in;
        token_data.token_sell_status = TokenSellStatus::None;
        token_data.pending_tp_sell_index = None;
        token_data.pending_tp_sell_amount = 0;
        token_data.tp_trailing_active = false;
        token_data.tp_trailing_max_price = 0.0;

        if token_data.token_balance > 0 {
            token_data.next_tp_index_to_sell = token_data.next_tp_index_to_sell.saturating_add(1);
            let _ = TOKEN_DB.upsert(sell_accounts.base_mint.clone(), token_data.clone());
            Some(token_data.clone())
        } else {
            let is_profit = token_data.token_price > token_data.token_buying_point_price;
            DYNAMIC_BUY.record_outcome(&token_data.matched_pattern_label, &sell_accounts.base_mint.to_string(), is_profit);
            let _ = TOKEN_DB.delete(sell_accounts.base_mint.clone());
            None
        }
    } else {
        let _ = TOKEN_DB.upsert(sell_accounts.base_mint.clone(), token_data.clone());
        Some(token_data.clone())
    }
}
