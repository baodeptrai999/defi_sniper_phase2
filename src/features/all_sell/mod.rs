use crate::*;
use solana_sdk::instruction::Instruction;
use tokio::time::{Duration, sleep};

pub async fn all_sell() {
    for token_data_map in TOKEN_DB.map.iter() {
        let mut token_data = token_data_map.value().clone();
        if token_data.token_balance > 0
            && token_data.token_sell_status != TokenSellStatus::SellTradeSubmitted
        {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.token_balance);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[Sell]\t*AUTO_TURN_OFF_TIME\t*Mint: {}\t*MC: {}\t*Amount: {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.token_balance
            );
            let _ = confirm(ix, tag).await;
            sleep(Duration::from_secs(2)).await;
        }
    }
}
