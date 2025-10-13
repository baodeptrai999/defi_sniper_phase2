use crate::*;
use colored::*;
use dashmap::DashMap;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use std::time::Instant;

pub async fn make_half_copy_tx(trade_token_data_map: &DashMap<Pubkey, TokenDatabaseSchema>) {
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
                "[ALL SELL]\t*RUG DETECTED\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.token_balance
            );

            warning!(
                "[ALL SELL]\t*{}\t*Mint: {}\t*Price: {}\t*Amount: {}",
                "RUG DETECTED".yellow(),
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.token_balance
            );

            (ix, tag)
        } else if black_list_filter(token_data.clone()).await
            && token_data.token_copy_trade_status == TokenCopyTradeStatus::TargetBought
            && half_copy_buy_filter_check(token_data.clone())
        {
            let buy_tx_remaining_counter = get_buy_tx_remain_counter();

            if !*DEV_MODE || buy_tx_remaining_counter != 0 {
                decrese_buy_tx_remain_counter();

                if !max_token_holder_check_and_top_twenty_holders(token_data.clone()).await {
                    continue;
                }

                let half_copy_trade_amount;
                if *HALF_COPY_PCNT_MODE {
                    let target_amount = match token_data.target_buy_amount {
                        Some(amount) => amount,
                        None => {
                            println!("Error fetching target buy amount, skipping..");
                            continue;
                        }
                    };
                    half_copy_trade_amount =
                        target_amount as f64 * *BUY_AMOUNT_PERCENT as f64 / 100.0;
                } else {
                    half_copy_trade_amount = *BUY_AMOUNT_SOL * 10f64.powi(9)
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
                    .get_buy_ix(half_copy_trade_amount, token_data.token_price);

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
                    "[Buy]\t*Mint: {}\t*Price: {}\t*Amount: {:.5} SOL",
                    token_data.pump_fun_swap_accounts.mint,
                    token_data.token_price,
                    half_copy_trade_amount as f64 / 10f64.powi(9)
                );

                info!(
                    "[Buy]\t*Mint: {}\t*Price: {}\t*Amount: {:.5} SOL",
                    token_data.pump_fun_swap_accounts.mint,
                    token_data.token_price,
                    half_copy_trade_amount as f64 / 10f64.powi(9)
                );
                (ix, tag)
            } else {
                (vec![], "".to_string())
            }
        } else if token_data.ts_state == TSMode::TS5Stop
            && token_data.tracked_ts_state != TSMode::TS5Stop
        {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.ts_stop_selling_plan.ts_5_stop);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            token_data.tracked_ts_state = TSMode::TS5Stop;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\t*TS_5_Stop triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_5_stop
            );

            info!(
                "[SELL]\t*TS_5_Stop triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_5_stop
            );

            (ix, tag)
        } else if token_data.ts_state == TSMode::TS4Stop
            && token_data.tracked_ts_state != TSMode::TS4Stop
        {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.ts_stop_selling_plan.ts_4_stop);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            token_data.tracked_ts_state = TSMode::TS4Stop;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\t*TS_4_Stop\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_4_stop,
            );

            info!(
                "[SELL]
                    \t*TS_4_Stop triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_4_stop,
            );

            (ix, tag)
        } else if token_data.ts_state == TSMode::TS3Stop
            && token_data.tracked_ts_state != TSMode::TS3Stop
        {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.ts_stop_selling_plan.ts_3_stop);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            token_data.tracked_ts_state = TSMode::TS3Stop;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\t*TS_3_Stop triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_3_stop,
            );

            info!(
                "[SELL]\t*TS_3_Stop triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_3_stop,
            );

            (ix, tag)
        } else if token_data.ts_state == TSMode::TS2Stop
            && token_data.tracked_ts_state != TSMode::TS2Stop
        {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.ts_stop_selling_plan.ts_2_stop);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            token_data.tracked_ts_state = TSMode::TS2Stop;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]
                    \t*TS_2_Stop
                    \t*Mint: {}
                    \t*Price: {}
                    \t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_2_stop,
            );

            info!(
                "[SELL]\t*TS_2_Stop\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_2_stop,
            );

            (ix, tag)
        } else if token_data.ts_state == TSMode::TS1Stop
            && token_data.tracked_ts_state != TSMode::TS1Stop
        {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.ts_stop_selling_plan.ts_1_stop);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            token_data.tracked_ts_state = TSMode::TS1Stop;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\t*TS_1_Stop truggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_1_stop,
            );

            info!(
                "[SELL]\t*TS_1_Stop triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_1_stop,
            );

            (ix, tag)
        } else if token_data.tp_state == TPMode::TP1 && token_data.tracked_tp_state != TPMode::TP1 {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.tp_selling_plan.tp_1);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            token_data.tracked_tp_state = TPMode::TP1;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\t*TP1 triggered\t*MINT: {}\t*PRICE: {}\t*AMOUNT: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_1,
            );

            info!(
                "[SELL]\t*TP1 triggered\t*MINT: {}\t*PRICE: {}\t*AMOUNT: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_1,
            );

            (ix, tag)
        } else if token_data.tp_state == TPMode::TP2 && token_data.tracked_tp_state != TPMode::TP2 {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.tp_selling_plan.tp_2);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            token_data.tracked_tp_state = TPMode::TP2;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\tTP2 triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_2,
            );

            info!(
                "[SELL]\tTP2 triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_2,
            );

            (ix, tag)
        } else if token_data.tp_state == TPMode::TP3 && token_data.tracked_tp_state != TPMode::TP3 {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.tp_selling_plan.tp_3);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            token_data.tracked_tp_state = TPMode::TP3;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\t*TP3 triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_3,
            );

            info!(
                "[SELL]\t*TP3 triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_3,
            );

            (ix, tag)
        } else if token_data.tp_state == TPMode::TP4 && token_data.tp_state != TPMode::TP4 {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.tp_selling_plan.tp_4);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            token_data.tracked_tp_state = TPMode::TP4;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\t*TP4 triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_4,
            );

            info!(
                "[SELL]\t*TP4 triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_4
            );

            (ix, tag)
        } else if token_data.tp_state == TPMode::TP5 && token_data.tracked_tp_state != TPMode::TP5 {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.tp_selling_plan.tp_5);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            token_data.tracked_tp_state = TPMode::TP5;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\t*TP5 triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_5,
            );

            info!(
                "[SELL]\t*TP5 triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_5
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
                "[SELL]\t*SL triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.token_balance,
            );

            info!(
                "[SELL]\t*SL triggered\t*Mint: {}\t*Price: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
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
