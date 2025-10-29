use crate::*;
use colored::*;
use dashmap::DashMap;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use std::time::Instant;

pub async fn make_copy_tx(trade_token_data_map: &DashMap<Pubkey, TokenDatabaseSchema>) {
    for trade_token_data in trade_token_data_map.iter() {
        let mut token_data = trade_token_data.value().clone();

        let instructions: (Vec<Instruction>, String) = if token_data.token_is_purchased
            && token_data.bundle_tx_counter >= *BUNDLE_TX_LIMIT
        {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.token_balance);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[ALL SELL]\t*RUG DETECTED\t*Mint: {}\t*MC: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_marketcap,
                token_data.token_balance
            );

            warning!(
                "[ALL SELL]\t*{}\t*Mint: {}\t*MC: {}\t*Amount: {}",
                "RUG DETECTED".yellow(),
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_marketcap,
                token_data.token_balance
            );

            (ix, tag)
        } else if black_list_filter(token_data.clone(), "Copy_Mode".to_string()).await
            && token_data.token_copy_trade_status == TokenCopyTradeStatus::TargetBought
            && buy_filter_check(token_data.clone(), "Copy_Mode".to_string())
        {
            let buy_tx_remaining_counter = get_buy_tx_remain_counter();

            if !*DEV_MODE || buy_tx_remaining_counter != 0 {
                decrese_buy_tx_remain_counter();

                if !max_token_holder_check(token_data.clone(), "Copy_Mode".to_string()).await {
                    continue;
                }

                let copy_trade_buy_amount;
                if *COPY_PCNT_MODE {
                    let target_amount = match token_data.target_buy_amount {
                        Some(amount) => amount,
                        None => {
                            println!("Error fetching target buy amount, skipping..");
                            continue;
                        }
                    };
                    copy_trade_buy_amount =
                        target_amount as f64 * *BUY_AMOUNT_PERCENT as f64 / 100.0;
                } else {
                    copy_trade_buy_amount = *BUY_AMOUNT_SOL * 10f64.powi(9)
                };

                token_data.token_copy_trade_status = TokenCopyTradeStatus::CopyTradeSubmitted;
                let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

                let build_tx_start = Instant::now();
                let mut ix: Vec<Instruction> = Vec::new();
                let create_ata_ix = token_data
                    .pump_fun_swap_accounts
                    .get_create_ata_idempotent_ix();
                let buy_ix = token_data
                    .pump_fun_swap_accounts
                    .get_buy_ix(copy_trade_buy_amount, token_data.token_price);

                ix.push(create_ata_ix);
                ix.push(buy_ix);

                let building_tx_time = build_tx_start.elapsed();
                println!(
                    "{}",
                    format!(
                        "{}: {}",
                        "Building tx took:".blue(),
                        format_elapsed_time(building_tx_time).blue()
                    )
                );

                let tag = format!(
                    "[Buy]\t*Mint: {}\t*MC: {}\t*Amount: {:.5} SOL",
                    token_data.pump_fun_swap_accounts.mint,
                    token_data.token_marketcap,
                    copy_trade_buy_amount as f64 / 10f64.powi(9)
                );

                info!(
                    "[Buy]\t*Mint: {}\t*MC: {}\t*Amount: {:.5} SOL",
                    token_data.pump_fun_swap_accounts.mint,
                    token_data.token_marketcap,
                    copy_trade_buy_amount as f64 / 10f64.powi(9)
                );
                (ix, tag)
            } else {
                (vec![], "".to_string())
            }
        } else if token_data.token_copy_trade_status == TokenCopyTradeStatus::TargetSold
            && token_data.token_is_purchased
        {
            let target_amount = match token_data.target_sell_amount {
                Some(amount) => amount,
                None => {
                    println!("Error fetching target buy amount, skipping..");
                    continue;
                }
            };
            let copy_trade_sell_amount = if token_data.token_balance as f64
                >= (target_amount as f64 * *SELL_AMOUNT_PCNT as f64 / 100.0)
            {
                target_amount as f64 * *SELL_AMOUNT_PCNT as f64 / 100.0
            } else {
                token_data.token_balance as f64
            };

            token_data.token_copy_trade_status = TokenCopyTradeStatus::CopyTradeSubmitted;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let build_tx_start = Instant::now();
            let mut ix: Vec<Instruction> = Vec::new();
            let sell_ix = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(copy_trade_sell_amount.trunc() as u64);

            ix.push(sell_ix);

            let building_tx_time = build_tx_start.elapsed();
            println!(
                "{}",
                format!(
                    "{}: {}",
                    "Building tx took:".blue(),
                    format_elapsed_time(building_tx_time).blue()
                )
            );

            let tag = format!(
                "[Sell]\t*Mint: {}\t*MC: {}\t*Amount: {:.5} token",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_marketcap,
                copy_trade_sell_amount as f64 / 10f64.powi(6)
            );

            info!(
                "[Sell]\t*Mint: {}\t*MC: {}\t*Amount: {:.5} token",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_marketcap,
                copy_trade_sell_amount as f64 / 10f64.powi(6)
            );
            (ix, tag)
        } else if token_data.tp_state == TPMode::CopyModeTp
            && token_data.tracked_tp_state != TPMode::CopyModeTp
        {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.token_balance);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            token_data.tracked_tp_state = TPMode::CopyModeTp;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\t*CopyModeTp triggered\t*Mint: {}\t*MC: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_marketcap,
                token_data.token_balance,
            );

            info!(
                "[SELL]\t*CopyModeTp triggered\t*Mint: {}\t*MC: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_marketcap,
                token_data.token_balance
            );

            (ix, tag)
        } else if token_data.tp_state == TPMode::SL && token_data.tracked_tp_state != TPMode::SL {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.token_balance);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            token_data.tracked_tp_state = TPMode::SL;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\t*SL triggered\t*Mint: {}\t*MC: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_marketcap,
                token_data.token_balance,
            );

            info!(
                "[SELL]\t*SL triggered\t*Mint: {}\t*MC: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_marketcap,
                token_data.token_balance,
            );
            (ix, tag)
        } else {
            (vec![], "".to_string())
        };

        let (ix, tag) = instructions;

        if !ix.is_empty() {
            tokio::spawn(async move {
                let _ = confirm(ix, tag).await;
            });
        }
    }
}
