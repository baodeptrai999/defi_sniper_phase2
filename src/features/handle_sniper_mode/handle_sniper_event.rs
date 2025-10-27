use crate::*;
use dashmap::DashMap;
use solana_sdk::pubkey::Pubkey;

pub async fn handle_sniper_event(
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
        mint_events,
        buy_events,
        sell_events,
        mint_ixs_accounts,
        _buy_ixs_accounts,
        _sell_ixs_accounts,
    ) = trade_data;

    let return_data: DashMap<Pubkey, TokenDatabaseSchema> = DashMap::new();

    for (i, mint_event) in mint_events.iter().enumerate() {
        let token_data: TokenDatabaseSchema = TokenDatabaseSchema::new_from_mint(
            mint_event.clone(),
            mint_ixs_accounts[i].clone(),
            tx_id.to_string(),
        );
        return_data.insert(token_data.token_mint, token_data);
    }

    for (_i, buy_event) in buy_events.iter().enumerate() {
        if let Some(token_data) = TOKEN_DB.get(buy_event.mint).unwrap() {
            let updated_token_data: TokenDatabaseSchema = update_status_from_buy_event(
                token_data.clone(),
                buy_event.clone(),
                tx_id.to_string(),
                "Sniper_Mode".to_string(),
            );
            return_data.insert(updated_token_data.token_mint, updated_token_data);
        }
    }

    for (_i, sell_event) in sell_events.iter().enumerate() {
        if let Some(token_data) = TOKEN_DB.get(sell_event.mint).unwrap() {
            if let Some(updated_token_data) = update_status_from_sell_event(
                token_data.clone(),
                sell_event.clone(),
                tx_id.to_string(),
                "Sniper_Mode".to_string(),
            ) {
                return_data.insert(updated_token_data.token_mint, updated_token_data);
            }
        }
    }
    return_data
}
