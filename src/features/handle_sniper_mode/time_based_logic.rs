use crate::*;
use solana_sdk::instruction::Instruction;
use tokio::time::{Duration, sleep};

pub async fn proceed_time_based_buying_logic(
    token_data: TokenDatabaseSchema,
) -> Result<(), Box<dyn std::error::Error>> {
    sleep(Duration::from_millis(50)).await;

    match TOKEN_DB.get(token_data.token_mint) {
        Ok(Some(mut updated_token_data)) => {
            let dev_buy_amount_filtered = dev_buy_filter(updated_token_data.dev_buy_sol_lamports);
            if dev_buy_amount_filtered && updated_token_data.token_marketcap < 40.0 {
                let mut ix: Vec<Instruction> = Vec::new();
                let create_ata_ix = updated_token_data
                    .pump_fun_swap_accounts
                    .get_create_ata_idempotent_ix();
                let buy_sol = updated_token_data.override_buy_amount_sol.unwrap_or(*BUY_AMOUNT_SOL);
                let buy_ix = updated_token_data.pump_fun_swap_accounts.get_buy_ix(
                    buy_sol * 10f64.powi(9),
                    updated_token_data.token_price,
                );

                ix.push(create_ata_ix);
                ix.push(buy_ix);

                let _ = TOKEN_DB.upsert(updated_token_data.token_mint, updated_token_data.clone());

                let tag = format!(
                    "[BUY]\t*Token mc after mint bundle is lower than 40 SOL\t*Mint: {}\t*MC: {} SOL",
                    updated_token_data.token_mint, updated_token_data.token_marketcap,
                );

                if !ix.is_empty() {
                    let _ = confirm(ix, tag).await;
                }
            }
        }
        Ok(None) => {}
        Err(_) => {}
    };

    Ok(())
}

//dev buy amount filter logic
pub fn dev_buy_filter(dev_buy_amount_sol_lamport_option: Option<u64>) -> bool {
    let filtered = if let Some(dev_buy_amount_sol_lamports) = dev_buy_amount_sol_lamport_option{
        if dev_buy_amount_sol_lamports % 10u64.pow(6) >= 1 && dev_buy_amount_sol_lamports % 10u64.pow(6) < 3000 {
            true
        } else {
            false
        }
    }else{
        false
    };

    filtered
}
