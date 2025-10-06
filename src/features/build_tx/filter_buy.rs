use crate::*;
use colored::*;

pub fn sniper_buy_filter_check(token_data: TokenDatabaseSchema) -> bool {
    let mut market_cap_valid = true;
    let mut volume_valid = true;

    if *MARKET_CAP_FILTER {
        if token_data.token_marketcap < *MIN_MARKET_CAP_LIMIT_SOL as f64 {
            market_cap_valid = false;
        }
    }

    if *VOLUME_FILTER {
        if let Some(val) = token_data.token_volume {
            if val < *MIN_VOLUME_LIMIT_SOL as f64 {
                volume_valid = false;
            }
        }
    }

    market_cap_valid && volume_valid
}

pub fn half_copy_buy_filter_check(token_data: TokenDatabaseSchema) -> bool {
    let mut market_cap_valid = true;

    if *MARKET_CAP_FILTER {
        if token_data.token_marketcap < *MIN_MARKET_CAP_LIMIT_SOL as f64 {
            market_cap_valid = false;
        }
    }

    market_cap_valid
}

pub fn black_list_filter(mut token_data: TokenDatabaseSchema) -> bool {
    let mut blacklist_valid = true;
    if *BLACK_LIST_FILTER {
        if token_data.token_is_blacklisted == TokenBlacklistInfo::None {
            if WALLET_BLACKLIST.contains(&token_data.token_creator.to_string()) {
                warning!(
                    "Token creator is blacklisted wallet: {}",
                    &token_data
                        .pump_fun_swap_accounts
                        .creator_vault
                        .to_string()
                        .red()
                );
                blacklist_valid = false;
                token_data.token_is_blacklisted = TokenBlacklistInfo::BlacklistedToken;
            }

            if TOKEN_BLACKLIST.contains(&token_data.token_mint.to_string()) {
                warning!(
                    "Token is blacklisted token: {}",
                    &token_data.token_mint.to_string().red()
                );
                blacklist_valid = false;
                token_data.token_is_blacklisted = TokenBlacklistInfo::BlacklistedToken;
            }

            if !blacklist_valid {
                let _ = TOKEN_DB.delete(token_data.token_mint);
            } else {
                token_data.token_is_blacklisted = TokenBlacklistInfo::NotBlacklistedToken;
                let _ = TOKEN_DB.upsert(token_data.token_mint, token_data);
            }
        }
    }
    blacklist_valid
}

pub fn max_token_holder_check(token_data: TokenDatabaseSchema) -> bool {
    let mut max_token_holder_valid = true;
    if *MAX_TOKEN_HOLDER_FILTER {
        let data = match RPC_CLIENT.get_token_largest_accounts(&token_data.token_mint) {
            Ok(data) => data,
            Err(_) => vec![],
        };

        if let Some(first) = data.get(0) {
            if first.address
                == token_data
                    .pump_fun_swap_accounts
                    .associated_bonding_curve
                    .to_string()
            {
                if let Some(second) = data.get(1) {
                    if let Some(val) = second.amount.ui_amount {
                        println!("Max holder amount (second): {}", val);
                        if val > *MAX_TOKEN_HOLDER_LIMIT as f64 {
                            error!(
                                "[FILTER] => MINT : {}\t* MAX HOLDING {:?} LIMIT {}",
                                token_data.token_mint, val, *MAX_TOKEN_HOLDER_LIMIT
                            );
                            max_token_holder_valid = false;
                        }
                    }
                }
            } else {
                if let Some(val) = first.amount.ui_amount {
                    println!("Max holder amount (first): {}", val);
                    if val > *MAX_TOKEN_HOLDER_LIMIT as f64 {
                        error!(
                            "[FILTER] => MINT : {}\t* MAX HOLDING {:?} LIMIT {}",
                            token_data.token_mint, val, *MAX_TOKEN_HOLDER_LIMIT
                        );
                        max_token_holder_valid = false;
                    }
                }
            }
        }
    }
    if !max_token_holder_valid {
        let _ = TOKEN_DB.delete(token_data.token_mint);
    }

    if max_token_holder_valid {
        println!("Max token holder limit passed");
    }
    max_token_holder_valid
}
