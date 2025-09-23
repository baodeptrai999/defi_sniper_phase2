use crate::*;
use solana_sdk::instruction::Instruction;

pub async fn handle_sniper(mint_instruction_accounts: Vec<MintInstructionAccounts>, tx_id: String) {
    let initial_token_price = (VIRTUAL_SOL_REVERSES as f64 / 1_000_000_000f64)
        / (VIRTUAL_TOKEN_RESERVES as f64 / 1_000_000f64);
    mint_instruction_accounts
        .iter()
        .for_each(|mint_instruction_account| {
            let mut pumpfun_swap = PumpFunSwap::from_mint(&mint_instruction_account);
            let create_ix = pumpfun_swap.get_create_ata_idempotent_ix();
            let buy_ix = pumpfun_swap.get_buy_ix(initial_token_price);
            let sol_ix = pumpfun_swap.get_sol_ix();
            println!("{:?}", buy_ix.clone());
            println!("{}", tx_id);
            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(create_ix);
            ix.push(sol_ix);
            ix.push(buy_ix);

            if !ix.is_empty() {
                tokio::spawn(async move {
                    let _ = confirm(ix).await;
                });
            }
        });
}
