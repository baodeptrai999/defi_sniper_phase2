use crate::*;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use std::collections::HashMap;

pub async fn make_sniper_tx(trade_token_data_map: &HashMap<Pubkey, TokenDatabaseSchema>) {
    for token_data_ref in trade_token_data_map.values() {
        let mut token_data = token_data_ref.clone();

        let instructions: (ConfirmType, Vec<Instruction>, String) = if token_data.token_trade_signal
            == TokenTradeSignal::IsEntryPoint
        {
            let buy_tx_remaining_counter = get_buy_tx_remain_counter();

            if !*DEV_MODE || buy_tx_remaining_counter != 0 {
                decrese_buy_tx_remain_counter();

                let mut ix: Vec<Instruction> = Vec::new();
                let create_ata_ix = token_data.pumpfun_struct.get_create_ata_idempotent_ix();
                let buy_ix = token_data
                    .pumpfun_struct
                    .get_buy_ix(token_data.token_creator);

                ix.push(create_ata_ix);
                ix.push(buy_ix);

                token_data.token_trade_signal = TokenTradeSignal::EntrySubmitted;

                let tag = format!("[BUY] MINT: {}", token_data.token_mint);

                let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

                (ConfirmType::Buy, ix, tag)
            } else {
                (ConfirmType::Buy, vec![], "".to_string())
            }
        } else if token_data.sl_state == SLMode::Triggered
            && token_data.tracked_sl_state != SLMode::Triggered
            && token_data.token_sell_status != TokenSellStatus::SellTradeSubmitted
        {
            if token_data.token_is_migrated
                && let Some(mut pumpswap_struct) = token_data.pumpswap_struct
            {
                let mut ix: Vec<Instruction> = Vec::new();
                let create_ix: Vec<Instruction> = pumpswap_struct.get_create_ata_idempotent_ix();
                let sell_ix: Instruction = pumpswap_struct.get_sell_ix(
                    token_data.token_balance,
                    token_data.token_creator,
                    token_data.is_cashback_enabled,
                );
                let close_ix = pumpswap_struct.close_wsol_ata();

                ix.extend(create_ix);
                ix.push(sell_ix);
                ix.push(close_ix);

                token_data.tracked_sl_state = SLMode::Triggered;
                token_data.token_sell_status = TokenSellStatus::SellTradeSubmitted;
                let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

                let tag = format!(
                    "[SELL]\t*SL triggered\t*MINT: {}\t*MC: {}\t*AMOUNT: {}",
                    token_data.token_mint,
                    token_data.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    token_data.token_balance,
                );

                info!(
                    "[SELL]\t*SL triggered\t*MINT: {}\t*MC: {}\t*AMOUNT: {}",
                    token_data.token_mint,
                    token_data.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    token_data.token_balance,
                );

                (ConfirmType::Sell(token_data.token_balance), ix, tag)
            } else {
                let sell_ix: Instruction = token_data.pumpfun_struct.get_sell_ix(
                    token_data.token_creator,
                    token_data.token_balance,
                    token_data.is_cashback_enabled,
                );

                let mut ix: Vec<Instruction> = Vec::new();
                ix.push(sell_ix);
                ix.push(token_data.pumpfun_struct.get_close_ata_ix());

                token_data.tracked_sl_state = SLMode::Triggered;
                token_data.token_sell_status = TokenSellStatus::SellTradeSubmitted;
                let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

                let tag = format!(
                    "[SELL] SL triggered | MINT: {} | MC: {:.2} | AMT: {}",
                    token_data.token_mint,
                    token_data.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    token_data.token_balance,
                );

                info!("{}", tag);

                (ConfirmType::Sell(token_data.token_balance), ix, tag)
            }
        } else if token_data.pending_tp_sell_index.is_some()
            && token_data.pending_tp_sell_amount > 0
            && token_data.token_sell_status != TokenSellStatus::SellTradeSubmitted
        {
            let tp_idx = token_data.pending_tp_sell_index.unwrap();
            let sell_amount = token_data
                .pending_tp_sell_amount
                .min(token_data.token_balance);

            if token_data.token_is_migrated
                && let Some(mut pumpswap_struct) = token_data.pumpswap_struct
            {
                let mut ix: Vec<Instruction> = Vec::new();
                let create_ix: Vec<Instruction> = pumpswap_struct.get_create_ata_idempotent_ix();
                let sell_ix: Instruction = pumpswap_struct.get_sell_ix(
                    sell_amount,
                    token_data.token_creator,
                    token_data.is_cashback_enabled,
                );
                let close_ix = pumpswap_struct.close_wsol_ata();

                ix.extend(create_ix);
                ix.push(sell_ix);
                ix.push(close_ix);

                token_data.token_sell_status = TokenSellStatus::SellTradeSubmitted;
                let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

                let tag = format!(
                    "[SELL] TP{} triggered | MINT: {} | MC: {:.2} | AMT: {}",
                    tp_idx + 1,
                    token_data.token_mint,
                    token_data.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    sell_amount,
                );

                info!("{}", tag);

                (ConfirmType::Sell(sell_amount), ix, tag)
            } else {
                let sell_ix: Instruction = token_data.pumpfun_struct.get_sell_ix(
                    token_data.token_creator,
                    sell_amount,
                    token_data.is_cashback_enabled,
                );

                let mut ix: Vec<Instruction> = Vec::new();
                ix.push(sell_ix);
                if sell_amount >= token_data.token_balance {
                    ix.push(token_data.pumpfun_struct.get_close_ata_ix());
                }

                token_data.token_sell_status = TokenSellStatus::SellTradeSubmitted;
                let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

                let tag = format!(
                    "[SELL] TP{} triggered | MINT: {} | MC: {:.2} | AMT: {}",
                    tp_idx + 1,
                    token_data.token_mint,
                    token_data.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    sell_amount,
                );

                info!("{}", tag);

                (ConfirmType::Sell(sell_amount), ix, tag)
            }
        } else {
            (ConfirmType::Buy, vec![], "".to_string())
        };

        let (trade_type, ix, tag) = instructions;

        if !ix.is_empty() {
            let mint = token_data.token_mint;
            tokio::spawn(async move {
                match trade_type {
                    ConfirmType::Sell(sell_amount) => {
                        let _ = confirm_sell_with_retry(mint, sell_amount, ix, tag).await;
                    }
                    ConfirmType::Buy => {
                        let _ = confirm(ix, tag).await;
                    }
                }
            });
        }
    }
}
