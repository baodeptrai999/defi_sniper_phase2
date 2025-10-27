use crate::*;
use colored::*;

pub fn buy_filter_check(token_data: TokenDatabaseSchema, mode: String) -> bool {
    let mut market_cap_valid = true;
    let mut volume_valid = true;

    if mode == "Sniper_Mode".to_string() {
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
    } else {
        if *MARKET_CAP_FILTER {
            if token_data.token_marketcap < *MIN_MARKET_CAP_LIMIT_SOL as f64 {
                market_cap_valid = false;
            }
        }
    }

    market_cap_valid && volume_valid
}

//Black list filter for token creator, mint and token holders
pub async fn black_list_filter(mut token_data: TokenDatabaseSchema, mode: String) -> bool {
    let mut token_blacklist_valid = true;
    let mut holder_blacklist_valid = true;

    let wallet_blacklist = WALLET_BLACKLIST.read().await;

    if *TOKEN_BLACK_LIST_FILTER && token_data.token_is_blacklisted == TokenBlacklistInfo::None {
        //token creator check
        if wallet_blacklist.contains(&token_data.token_creator.to_string()) {
            error!(
                "Token creator is blacklisted wallet\t*creator: {}\t*mint: {}",
                &token_data.token_creator.to_string().red(),
                &token_data.token_mint.to_string().yellow()
            );
            token_blacklist_valid = false;
        }

        if mode != "Sniper_Mode".to_string() {
            //token mint address blacklist check
            let token_blacklist = TOKEN_BLACKLIST.read().await;
            if token_blacklist.contains(&token_data.token_mint.to_string()) {
                error!(
                    "Token is blacklisted token\t*mint: {}",
                    &token_data.token_mint.to_string().red()
                );
                token_blacklist_valid = false;
            }
        }
    }

    if *HOLDER_BLACK_LIST_FILTER && token_data.token_is_blacklisted == TokenBlacklistInfo::None {
        if mode == "Sniper_Mode".to_string() {
            let wallet_blacklist = WALLET_BLACKLIST.read().await;
            for holder in token_data.token_holders.iter() {
                if wallet_blacklist.contains(&holder.0.to_string()) {
                    holder_blacklist_valid = false;
                    break;
                }
            }
        } else {
            let data = match RPC_CLIENT
                .get_token_largest_accounts(&token_data.token_mint)
                .await
            {
                Ok(data) => data,
                Err(_) => vec![],
            };

            for holder in data.iter() {
                if wallet_blacklist.contains(&holder.address.clone()) {
                    holder_blacklist_valid = false;
                    error!(
                        "[FILTER] => MINT : {}\t*BLACKLISTED HOLDER {:?}",
                        token_data.token_mint, holder.address
                    );
                    break;
                }
            }
        }
    }

    if !token_blacklist_valid || !holder_blacklist_valid {
        let _ = TOKEN_DB.delete(token_data.token_mint);
    } else {
        token_data.token_is_blacklisted = TokenBlacklistInfo::NotBlacklistedToken;
        let _ = TOKEN_DB.upsert(token_data.token_mint, token_data);
    }

    token_blacklist_valid && holder_blacklist_valid
}

pub async fn max_token_holder_check(token_data: TokenDatabaseSchema, mode: String) -> bool {
    let mut max_token_holder_valid = true;

    if *MAX_TOKEN_HOLDER_FILTER {
        if mode != "Sniper_Mode" {
            let data = match RPC_CLIENT
                .get_token_largest_accounts(&token_data.token_mint)
                .await
            {
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
        } else {
            for holder in token_data.token_holders.iter() {
                if *holder.1 >= *MAX_TOKEN_HOLDER_LIMIT {
                    error!(
                        "[FILTER] => MINT : {}\t* MAX HOLDING {:?} LIMIT {}",
                        token_data.token_mint, holder.1, *MAX_TOKEN_HOLDER_LIMIT
                    );
                    max_token_holder_valid = false;
                    break;
                }
            }
        }
    }

    if !max_token_holder_valid {
        let _ = TOKEN_DB.delete(token_data.token_mint);
    }

    max_token_holder_valid
}
