use crate::*;
use colored::*;
use dashmap::DashMap;
use solana_sdk::pubkey::Pubkey;

pub async fn handle_half_copy_events(
    trade_data: (
        Vec<MintEvent>,
        Vec<BuyEvent>,
        Vec<SellEvent>,
        Vec<MintInstructionAccounts>,
        Vec<BuyInstructionAccounts>,
        Vec<SellInstructionAccounts>,
    ),
    tx_id: String,
) -> DashMap<Pubkey, (TokenDatabaseSchema, u64)> {
    let (
        _mint_events,
        buy_events,
        sell_events,
        _mint_ixs_accounts,
        buy_ixs_accounts,
        sell_ixs_accounts,
    ) = trade_data;

    let return_data: DashMap<Pubkey, (TokenDatabaseSchema, u64)> = DashMap::new();
    for (i, buy_event) in buy_events.iter().enumerate() {
        if TARGET_WALLETS.contains(&buy_event.user.to_string()) {
            info!(
                "Target [{}] buy\t*Mint: {}\t*Tx: {}",
                buy_event.user.to_string().cyan(),
                buy_event.mint.to_string().green(),
                solscan!(tx_id.to_string().purple())
            );
            if let Some(token_data) = TOKEN_DB.get(buy_event.mint).unwrap() {
                let updated_token_data = update_status_from_buy_event(
                    token_data.clone(),
                    buy_event.clone(),
                    tx_id.to_string(),
                );
                return_data.insert(
                    updated_token_data.token_mint,
                    (updated_token_data, buy_event.sol_amount),
                );
            } else {
                let token_data: TokenDatabaseSchema = TokenDatabaseSchema::new_from_target_buy(
                    buy_event.clone(),
                    buy_ixs_accounts[i].clone(),
                    tx_id.to_string(),
                );
                return_data.insert(token_data.token_mint, (token_data, buy_event.sol_amount));
            }
        } else {
            if let Some(token_data) = TOKEN_DB.get(buy_event.mint).unwrap() {
                let updated_token_data: TokenDatabaseSchema = update_status_from_buy_event(
                    token_data.clone(),
                    buy_event.clone(),
                    tx_id.to_string(),
                );
                return_data.insert(
                    updated_token_data.token_mint,
                    (updated_token_data, buy_event.sol_amount),
                );
            }
        }
    }

    for (i, sell_event) in sell_events.iter().enumerate() {
        if let Some(token_data) = TOKEN_DB.get(sell_event.mint).unwrap() {
            if !token_data.token_is_purchased
                && TARGET_WALLETS.contains(&sell_event.user.to_string())
                && !half_copy_buy_filter_check(token_data.clone())
            {
                let target_token_account_balance =
                    RPC_CLIENT.get_token_account_balance(&sell_ixs_accounts[i].associated_user);
                match target_token_account_balance {
                    Ok(balance) => {
                        if let Some(amount) = balance.ui_amount {
                            if amount <= 0.0 {
                                alert!(
                                    "[Sell]\t*Stop monitoring\t*Mint: {}\t*Target {} sold token before our filter",
                                    sell_event.mint,
                                    sell_event.user
                                );
                                let _ = TOKEN_DB.delete(sell_event.mint);
                                continue;
                            }
                        }
                    }
                    Err(_) => {
                        alert!(
                            "[Sell]\t*Stop monitoring\t*Mint: {}\t*Target {} sold token before our filter",
                            sell_event.mint,
                            sell_event.user
                        );
                        let _ = TOKEN_DB.delete(sell_event.mint);
                        continue;
                    }
                }
            }

            if let Some(updated_token_data) = update_status_from_sell_event(
                token_data.clone(),
                sell_event.clone(),
                tx_id.to_string(),
            ) {
                return_data.insert(
                    updated_token_data.token_mint,
                    (updated_token_data, sell_event.sol_amount),
                );
            }
        }
    }
    return_data
}
