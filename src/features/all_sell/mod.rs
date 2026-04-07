use crate::*;
use borsh::BorshDeserialize;
use solana_account_decoder_client_types::UiAccountData;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk_ids::system_program;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use tokio::time::{Duration, sleep};

pub async fn all_sell() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Get all token accounts the wallet holds (V1 + V2)
    let v1_token_ata_data = match RPC_CLIENT
        .get_token_accounts_by_owner(
            &SIGNER_PUBKEY,
            TokenAccountsFilter::ProgramId(spl_token::ID),
        )
        .await
    {
        Ok(data) => data,
        Err(e) => {
            error!("[ALL_SELL] Failed to fetch V1 token accounts: {}", e);
            vec![]
        }
    };

    let v2_token_ata_data = match RPC_CLIENT
        .get_token_accounts_by_owner(
            &SIGNER_PUBKEY,
            TokenAccountsFilter::ProgramId(TOKEN_2022_PROGRAM),
        )
        .await
    {
        Ok(data) => data,
        Err(e) => {
            error!("[ALL_SELL] Failed to fetch V2 token accounts: {}", e);
            vec![]
        }
    };

    // Collect (mint, token_program) pairs
    let token_entries: Vec<(Pubkey, Pubkey)> = v1_token_ata_data
        .iter()
        .filter_map(|account| {
            if let UiAccountData::Json(parsed_account) = &account.account.data {
                parsed_account.parsed["info"]["mint"]
                    .as_str()
                    .and_then(|mint_str| mint_str.parse::<Pubkey>().ok())
                    .map(|mint| (mint, spl_token::ID))
            } else {
                None
            }
        })
        .chain(v2_token_ata_data.iter().filter_map(|account| {
            if let UiAccountData::Json(parsed_account) = &account.account.data {
                parsed_account.parsed["info"]["mint"]
                    .as_str()
                    .and_then(|mint_str| mint_str.parse::<Pubkey>().ok())
                    .map(|mint| (mint, TOKEN_2022_PROGRAM))
            } else {
                None
            }
        }))
        .filter(|(mint, _)| *mint != WSOL)
        .collect();

    info!("[ALL_SELL] ─────────────────────────────────────────");
    info!("[ALL_SELL] Found {} token mints to process", token_entries.len());
    info!("[ALL_SELL] ─────────────────────────────────────────");

    let mut sold = 0u32;
    let mut skipped = 0u32;

    // 2. For each mint, derive bonding curve and check state
    for (i, (mint, token_program)) in token_entries.iter().enumerate() {
        let (bonding_curve, _) = Pubkey::find_program_address(
            &[PUMPFUN_BONDING_CURVE, mint.as_ref()],
            &PUMPFUN_PROGRAM_ID,
        );

        // Get token balance
        let associated_user = get_associated_token_address_with_program_id(
            &*SIGNER_PUBKEY,
            mint,
            token_program,
        );
        let balance = match RPC_CLIENT.get_token_account_balance(&associated_user).await {
            Ok(ui_amount) => ui_amount.amount.parse::<u64>().unwrap_or(0),
            Err(_) => 0,
        };

        if balance == 0 {
            continue;
        }

        // Fetch bonding curve to determine pumpfun vs pumpswap
        match RPC_CLIENT.get_account(&bonding_curve).await {
            Ok(account_data) => {
                if let Ok(curve) = PumpfunBondingCurve::try_from_slice(&account_data.data[8..83]) {
                    // Calculate SOL value of held tokens from bonding curve
                    let sol_value_lamports = (balance as u128)
                        .checked_mul(curve.virtual_sol_reserves as u128)
                        .unwrap_or(0)
                        / (curve.virtual_token_reserves as u128).max(1);

                    // Estimate sell tx fee: base (5000) + priority (CU * micro_lamports / 1e6)
                    let sell_fee_lamports: u64 =
                        5_000 + (200_000u64 * (*SELL_MICRO_LAMPORTS as u64) / 1_000_000);

                    if (sol_value_lamports as u64) <= sell_fee_lamports {
                        info!(
                            "[ALL_SELL] [{}/{}] SKIP  | {} | Value {:.6} SOL <= Fee {:.6} SOL",
                            i + 1, token_entries.len(), mint,
                            sol_value_lamports as f64 / 1e9,
                            sell_fee_lamports as f64 / 1e9,
                        );
                        skipped += 1;
                        continue;
                    }

                    let platform = if curve.complete { "Pumpswap" } else { "Pumpfun" };
                    info!(
                        "[ALL_SELL] [{}/{}] SELL  | {} | {} | Balance: {} | Value: {:.6} SOL",
                        i + 1, token_entries.len(), mint, platform, balance,
                        sol_value_lamports as f64 / 1e9
                    );

                    if curve.complete {
                        sell_on_pumpswap(mint, token_program, balance, curve.is_cashback_coin).await;
                    } else {
                        sell_on_pumpfun(
                            mint, token_program, &bonding_curve, &curve.creator, balance, curve.is_cashback_coin,
                        )
                        .await;
                    }
                    sold += 1;
                } else {
                    error!("[ALL_SELL] [{}/{}] ERR   | {} | Failed to decode bonding curve", i + 1, token_entries.len(), mint);
                }
            }
            Err(_) => {
                error!("[ALL_SELL] [{}/{}] ERR   | {} | No bonding curve found", i + 1, token_entries.len(), mint);
            }
        }

        sleep(Duration::from_secs(2)).await;
    }

    info!("[ALL_SELL] ─────────────────────────────────────────");
    info!("[ALL_SELL] Done | Sold: {} | Skipped: {}", sold, skipped);
    info!("[ALL_SELL] ─────────────────────────────────────────");
    Ok(())
}

async fn sell_on_pumpfun(
    mint: &Pubkey,
    token_program: &Pubkey,
    bonding_curve: &Pubkey,
    creator: &Pubkey,
    balance: u64,
    is_cashback_coin: bool,
) {
    let associated_bonding_curve = get_associated_token_address_with_program_id(
        bonding_curve,
        mint,
        token_program,
    );

    let associated_user = get_associated_token_address_with_program_id(
        &*SIGNER_PUBKEY,
        mint,
        token_program,
    );

    let (bonding_curve_v2_pda, _) = Pubkey::find_program_address(
        &[PUMPFUN_BONDING_CURVE_V2_SEED, mint.as_ref()],
        &PUMPFUN_PROGRAM_ID,
    );

    let mut pumpfun = PumpfunStruct {
        global: PUMPFUN_GLOBAL,
        fee_recipient: PUMPFUN_FEE_RECIPIENT,
        mint: *mint,
        bonding_curve: *bonding_curve,
        associated_bonding_curve,
        user: *SIGNER_PUBKEY,
        associated_user,
        system_program: system_program::ID,
        token_program: *token_program,
        event_authority: PUMP_FUN_EVENT_AUTHORITY,
        program: PUMPFUN_PROGRAM_ID,
        fee_config: PUMPFUN_FEE_CONFIG,
        fee_program: PUMPFUN_FEE_PROGRAM,
        global_volume_accumulator: PUMPFUN_GLOBAL_VOLUME_ACCUMULATOR,
        user_volume_accumulator: get_pumpfun_user_volume_accumulator(*SIGNER_PUBKEY),
        bonding_curve_v2_pda,
    };

    let create_ix = pumpfun.get_create_ata_idempotent_ix();
    let sell_ix = pumpfun.get_sell_ix(*creator, balance, is_cashback_coin);
    let close_ata_ix = pumpfun.get_close_ata_ix();

    let _ = confirm_sell(
        vec![create_ix, sell_ix, close_ata_ix],
        format!("[ALL_SELL] Pumpfun | {}", mint),
    )
    .await;
}

async fn sell_on_pumpswap(mint: &Pubkey, token_program: &Pubkey, balance: u64, is_cashback_coin: bool) {
    // Derive pool PDA
    let (pool, _) = Pubkey::find_program_address(
        &[POOL_SEED, mint.as_ref(), WSOL.as_ref()],
        &PUMPSWAP_PROGRAM_ID,
    );

    // Fetch pool state to get coin_creator and is_mayhem_mode
    let pool_data = match fetch_and_decode_pool(pool).await {
        Some(data) => data,
        None => {
            info!("[ALL_SELL] Failed to fetch pumpswap pool for {}", mint);
            return;
        }
    };

    let protocol_fee_recipient = if pool_data.is_mayhem_mode {
        MAYHEM_PROTOCOL_FEE_RECIPIENT
    } else {
        PUMPSWAP_FEE_1
    };

    let quote_token_program = spl_token::ID;

    let user_base_token_account = get_associated_token_address_with_program_id(
        &*SIGNER_PUBKEY,
        mint,
        token_program,
    );

    let user_quote_token_account = get_associated_token_address_with_program_id(
        &*SIGNER_PUBKEY,
        &WSOL,
        &quote_token_program,
    );

    let protocol_fee_recipient_token_account = get_associated_token_address_with_program_id(
        &protocol_fee_recipient,
        &WSOL,
        &quote_token_program,
    );

    let (pool_v2_pda, _) = Pubkey::find_program_address(
        &[PUMPSWAP_POOL_V2_SEED, mint.as_ref()],
        &PUMPSWAP_PROGRAM_ID,
    );

    let mut pumpswap = PumpSwapStruct {
        pool,
        user: *SIGNER_PUBKEY,
        global_config: PUMPSWAP_GLOBAL,
        base_mint: *mint,
        quote_mint: WSOL,
        user_base_token_account,
        user_quote_token_account,
        pool_base_token_account: pool_data.pool_base_token_account,
        pool_quote_token_account: pool_data.pool_quote_token_account,
        protocol_fee_recipient,
        protocol_fee_recipient_token_account,
        base_token_program: *token_program,
        quote_token_program,
        system_program: system_program::ID,
        associated_token_program: ASSOCIATED_PROGRAM,
        event_authority: PUMPSWAP_EVENT_AUTH,
        program: PUMPSWAP_PROGRAM_ID,
        global_volume_accumulator: PUMPSWAP_GLOBAL_VOLUME_ACCUMULATOR,
        user_volume_accumulator: get_pumpswap_user_volume_accumulator(*SIGNER_PUBKEY),
        fee_config: PUMPSWAP_FEE_CONFIG,
        fee_program: PUMPSWAP_FEE_PROGRAM,
        pool_v2_pda,
    };

    let close_token_ata_ix = spl_token::instruction::close_account(
        token_program,
        &user_base_token_account,
        &*SIGNER_PUBKEY,
        &*SIGNER_PUBKEY,
        &[&*SIGNER_PUBKEY],
    )
    .expect("close_account ix");

    let mut ixs: Vec<Instruction> = Vec::new();
    ixs.extend(pumpswap.get_create_ata_idempotent_ix());
    ixs.push(pumpswap.get_sell_ix(balance, pool_data.coin_creator, is_cashback_coin));
    ixs.push(close_token_ata_ix);
    ixs.push(pumpswap.close_wsol_ata());

    let _ = confirm_sell(
        ixs,
        format!("[ALL_SELL] Pumpswap | {}", mint),
    )
    .await;
}
