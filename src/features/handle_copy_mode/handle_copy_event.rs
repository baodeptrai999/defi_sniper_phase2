use crate::*;
use colored::*;
use dashmap::DashMap;
use solana_sdk::pubkey::Pubkey;

pub async fn handle_copy_events(
    trade_data: (
        Vec<MintEvent>,
        Vec<BuyEvent>,
        Vec<SellEvent>,
        Vec<MintInstructionAccounts>,
        Vec<BuyInstructionAccounts>,
        Vec<SellInstructionAccounts>,
    ),
    tx_id: String,
) -> DashMap<Pubkey, TokenDatabaseSchema> {
    let (
        _mint_events,
        buy_events,
        sell_events,
        _mint_ixs_accounts,
        buy_ixs_accounts,
        sell_ixs_accounts,
    ) = trade_data;

    let return_data: DashMap<Pubkey, TokenDatabaseSchema> = DashMap::new();
    for (i, buy_event) in buy_events.iter().enumerate() {
        if TARGET_WALLETS.contains(&buy_event.user.to_string()) {
            info!(
                "Target [{}] buy\t*Mint: {}\t*Tx: {}",
                buy_event.user.to_string().cyan(),
                buy_event.mint.to_string().green(),
                solscan!(tx_id.to_string().purple())
            );
            if let Some(token_data) = TOKEN_DB.get(buy_event.mint).unwrap() {
                if *ONE_TIME_COPY {
                    let updated_token_data = update_status_from_buy_event_copy_mode(
                        token_data.clone(),
                        buy_event.clone(),
                        tx_id.to_string(),
                    );
                    return_data.insert(updated_token_data.token_mint, updated_token_data);
                } else {
                    let mut updated_token_data = update_status_from_buy_event_copy_mode(
                        token_data.clone(),
                        buy_event.clone(),
                        tx_id.to_string(),
                    );
                    updated_token_data.token_copy_trade_status = TokenCopyTradeStatus::TargetBought;
                    updated_token_data.target_buy_amount = Some(buy_event.sol_amount);
                    let _ = TOKEN_DB.upsert(buy_event.mint.clone(), updated_token_data.clone());
                    return_data.insert(buy_event.mint, updated_token_data);
                }
            } else {
                let token_data: TokenDatabaseSchema = TokenDatabaseSchema::new_from_target_buy(
                    buy_event.clone(),
                    buy_ixs_accounts[i].clone(),
                    tx_id.to_string(),
                );
                return_data.insert(token_data.token_mint, token_data);
            }
        } else {
            if let Some(token_data) = TOKEN_DB.get(buy_event.mint).unwrap() {
                let updated_token_data: TokenDatabaseSchema = update_status_from_buy_event_copy_mode(
                    token_data.clone(),
                    buy_event.clone(),
                    tx_id.to_string(),
                );
                return_data.insert(updated_token_data.token_mint, updated_token_data);
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
                                    "*Stop monitoring\t*Mint: {}\t*Target {} sold token before our filter",
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
                            "*Stop monitoring\t*Mint: {}\t*Target {} sold token before our filter",
                            sell_event.mint,
                            sell_event.user
                        );
                        let _ = TOKEN_DB.delete(sell_event.mint);
                        continue;
                    }
                }
            }

            if TARGET_WALLETS.contains(&sell_event.user.to_string()) {
                info!(
                    "Target [{}] sell\t*Mint: {}\t*Tx: {}",
                    sell_event.user.to_string().cyan(),
                    sell_event.mint.to_string().green(),
                    solscan!(tx_id.to_string().purple())
                );
                let updated_token_data =
                    update_status_from_sell_event_copy_mode(token_data, sell_event.clone(), tx_id.clone());
                if let Some(mut updated_data) = updated_token_data {
                    updated_data.token_copy_trade_status = TokenCopyTradeStatus::TargetSold;
                    updated_data.target_sell_amount = Some(sell_event.token_amount);
                    let _ = TOKEN_DB.upsert(sell_event.mint.clone(), updated_data.clone());

                    return_data.insert(sell_event.mint, updated_data);
                }
            } else if let Some(updated_token_data) = update_status_from_sell_event_copy_mode(
                token_data.clone(),
                sell_event.clone(),
                tx_id.to_string(),
            ) {
                return_data.insert(updated_token_data.token_mint, updated_token_data);
            }
        }
    }
    return_data
}
