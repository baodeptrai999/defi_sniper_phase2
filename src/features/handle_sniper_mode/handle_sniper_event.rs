use crate::*;
use solana_sdk::pubkey::Pubkey;
use std::collections::{HashMap, HashSet};

const MAX_BUY_TX_HISTORY: usize = 4;

pub async fn handle_trade_events(
    budget_compute_data: (u32, u64),
    pumpfun_trade_data: (
        Vec<MintEvent>,
        Vec<PumpfunBuyEvent>,
        Vec<PumpfunSellEvent>,
        Vec<MintInstructionAccounts>,
        Vec<PumpfunBuyInstructionAccounts>,
        Vec<PumpfunSellInstructionAccounts>,
    ),
    migration_data: (
        Vec<MigrateInstructionAccounts>,
        Vec<CreatePoolInstructionAccounts>,
        Vec<CreatePoolEventData>,
    ),
    pumpswap_trade_data: (
        Vec<PumpswapBuyEvent>,
        Vec<PumpswapSellEvent>,
        Vec<PumpswapBuyInstructionAccounts>,
        Vec<PumpswapSellInstructionAccounts>,
    ),
    tx_id: String,
) -> HashMap<Pubkey, TokenDatabaseSchema> {
    let (unit, price) = budget_compute_data;

    let (
        mint_events,
        pumpfun_buy_events,
        pumpfun_sell_events,
        mint_ixs_accounts,
        _pumpfun_buy_ixs_accounts,
        _pumpfun_sell_ixs_accounts,
    ) = pumpfun_trade_data;

    let (_migrate_instruction_accounts, create_pool_instruction_accounts, create_pool_event_data) =
        migration_data;

    let (
        pumpswap_buy_events,
        pumpswap_sell_events,
        pumpswap_buy_ixs_accounts,
        pumpswap_sell_ixs_accounts,
    ) = pumpswap_trade_data;

    let mut return_data: HashMap<Pubkey, TokenDatabaseSchema> = HashMap::new();
    let patterns = get_cached_patterns();

    // ── Mint events ──

    let mut minted_in_this_tx: HashSet<Pubkey> = HashSet::new();

    for (i, mint_event) in mint_events.iter().enumerate() {
        let mint_ix_accounts = &mint_ixs_accounts[i];

        let mint_pattern_matched = patterns.iter().any(|p| p.mint_pattern == (unit, price))
            || MANUAL_MINT_PRICE_PATTERNS.contains(&price);

        // if mint_pattern_matched {
        //     let token_data = TokenDatabaseSchema::new_from_mint(
        //         mint_event.clone(),
        //         mint_ix_accounts.clone(),
        //         (unit, price),
        //         tx_id.clone(),
        //     );

        //     minted_in_this_tx.insert(token_data.token_mint);
        //     return_data.insert(token_data.token_mint, token_data);
        // }

        let mut token_data = TokenDatabaseSchema::new_from_mint(
            mint_event.clone(),
            mint_ix_accounts.clone(),
            (unit, price),
            tx_id.clone(),
        );

        token_data.token_trade_signal = TokenTradeSignal::IsEntryPoint;
        minted_in_this_tx.insert(token_data.token_mint);
        let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());
        return_data.insert(token_data.token_mint, token_data);

    }

    // ── Pumpfun Buy events: single pass for counting + state update ──

    let mut buy_counts: HashMap<Pubkey, u8> = HashMap::new();

    for pumpfun_buy_event in pumpfun_buy_events.iter() {
        let mint = pumpfun_buy_event.mint;

        if let Some(token_data) = TOKEN_DB.get(mint).unwrap() {
            if !minted_in_this_tx.contains(&mint) {
                if let Some(c) = buy_counts.get_mut(&mint) {
                    *c += 1;
                } else if token_data.buy_tx_history.len() < MAX_BUY_TX_HISTORY
                    && matches!(
                        token_data.token_trade_signal,
                        TokenTradeSignal::None | TokenTradeSignal::EntrySubmitted
                    )
                {
                    buy_counts.insert(mint, 1);
                }
            }

            let updated = update_status_from_pumpfun_buy_event(
                token_data,
                pumpfun_buy_event.clone(),
                tx_id.clone(),
            );
            return_data.insert(updated.token_mint, updated);
        }
    }

    // ── Pattern match check for eligible mints ──

    for (mint, buy_count) in buy_counts {
        if let Some(mut token_data) = TOKEN_DB.get(mint).unwrap() {
            token_data.buy_tx_history.push(((unit, price), buy_count));

            let mint_pat = (
                token_data.mint_budget_compute_unit_limit,
                token_data.mint_budget_compute_unit_price,
            );
            let history = &token_data.buy_tx_history;

            let mut matched_pattern: Option<&TokenFilter> = None;

            for pattern in patterns.iter() {
                if pattern.mint_pattern != mint_pat {
                    continue;
                }

                if *history == pattern.buy_pattern {
                    info!(
                        "[BUNDLE_MATCH] MINT: {} | exact | len: {}",
                        mint,
                        pattern.buy_pattern.len(),
                    );
                    matched_pattern = Some(pattern);
                    break;
                }
            }

            if let Some(pattern) = matched_pattern {
                let primary_tp_threshold = pattern.primary_tp_threshold();
                token_data.set_tp_sell_strategy(
                    pattern.tp_threshold.clone(),
                    pattern.sell_amounts.clone(),
                );

                match token_data.token_trade_signal {
                    TokenTradeSignal::None => {
                        info!(
                            "🎯 [MATCH] MINT: {} | pattern: {:?} {:?} | TP: {:?}% (real: {:?}%) | sell: {:?}% | profit: {:.4} | n: {} | avg: {:.4} | WR_low: {} | W/L: {}/{} | WR: {:.2}%",
                            mint,
                            pattern.mint_pattern,
                            pattern.buy_pattern,
                            pattern.tp_threshold,
                            primary_tp_threshold * *REAL_TP_MULTIPLY,
                            pattern.sell_amounts,
                            pattern.net_profit,
                            pattern.token_count,
                            pattern.avg_profit,
                            pattern.win_rate_low,
                            pattern.win_count,
                            pattern.loss_count,
                            pattern.win_rate,
                        );
                        token_data.token_trade_signal = TokenTradeSignal::IsEntryPoint;
                    }
                    TokenTradeSignal::EntrySubmitted => {
                        info!(
                            "🔄 [TP_UPDATE] MINT: {} | pattern: {:?} | new TP: {:?}% (real: {:?}%)",
                            mint,
                            pattern.buy_pattern,
                            pattern.tp_threshold,
                            primary_tp_threshold * *REAL_TP_MULTIPLY,
                        );
                    }
                    _ => {}
                }
            }

            let _ = TOKEN_DB.upsert(mint, token_data.clone());
            return_data.insert(mint, token_data);
        }
    }

    // ── Pumpfun Sell events ──

    for pumpfun_sell_event in pumpfun_sell_events.iter() {
        if let Some(token_data) = TOKEN_DB.get(pumpfun_sell_event.mint).unwrap() {
            if let Some(updated) = update_status_from_pumpfun_sell_event(
                token_data,
                pumpfun_sell_event.clone(),
                tx_id.clone(),
            ) {
                return_data.insert(updated.token_mint, updated);
            }
        }
    }

    // handle migration instructions
    for (pool_accounts, pool_event) in create_pool_instruction_accounts
        .iter()
        .zip(create_pool_event_data.iter())
    {
        if let Some(token_data) = TOKEN_DB.get(pool_accounts.base_mint).unwrap() {
            let updated_token_data = update_status_from_migration_event(
                token_data.clone(),
                pool_accounts.clone(),
                pool_event.clone(),
                tx_id.to_string(),
            );
            return_data.insert(updated_token_data.token_mint, updated_token_data);
        }
    }

    //handle pumpswap instructions
    for (i, pumpswap_buy_event) in pumpswap_buy_events.iter().enumerate() {
        if let Some(token_data) = TOKEN_DB
            .get(pumpswap_buy_ixs_accounts[i].base_mint)
            .unwrap()
        {
            let updated_token_data = update_status_from_pumpswap_buy_event(
                token_data.clone(),
                pumpswap_buy_event.clone(),
                pumpswap_buy_ixs_accounts[i].clone(),
                tx_id.to_string(),
            );
            return_data.insert(updated_token_data.token_mint, updated_token_data);
        }
    }

    for (i, pumpswap_sell_event) in pumpswap_sell_events.iter().enumerate() {
        if let Some(token_data) = TOKEN_DB
            .get(pumpswap_sell_ixs_accounts[i].base_mint)
            .unwrap()
        {
            let updated_token_data = update_status_from_pumpswap_sell_event(
                token_data,
                pumpswap_sell_event.clone(),
                pumpswap_sell_ixs_accounts[i].clone(),
                tx_id.clone(),
            );

            if let Some(updated_data) = updated_token_data {
                let _ = TOKEN_DB.upsert(
                    pumpswap_sell_ixs_accounts[i].base_mint.clone(),
                    updated_data.clone(),
                );

                return_data.insert(pumpswap_sell_ixs_accounts[i].base_mint, updated_data);
            }
        }
    }

    return_data
}
