use crate::*;
use borsh::BorshDeserialize;
use solana_account_decoder_client_types::UiAccountData;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use tokio::time::{Duration, sleep};

pub async fn all_sell() -> Result<(), Box<dyn std::error::Error>> {
    let token_ata_data = match RPC_CLIENT
        .get_token_accounts_by_owner(
            &SIGNER_PUBKEY,
            TokenAccountsFilter::ProgramId(spl_token::ID),
        )
        .await
    {
        Ok(data) => data,
        Err(_) => vec![],
    };

    let mints: Vec<Pubkey> = token_ata_data
        .into_iter()
        .filter_map(|account| {
            if let UiAccountData::Json(parsed_account) = &account.account.data {
                parsed_account.parsed["info"]["mint"]
                    .as_str()
                    .and_then(|mint_str| mint_str.parse::<Pubkey>().ok())
            } else {
                None
            }
        })
        .filter(|mint| *mint != WSOL) // Exclude WSOL
        .collect();

    let mut pumpfun_keys = Vec::new();

    for mint in &mints {
        let (bonding_curve, _) = Pubkey::find_program_address(
            &[PUMPFUN_BONDING_CURVE, mint.as_ref()],
            &PUMPFUN_PROGRAM_ID,
        );

        match RPC_CLIENT.get_account(&bonding_curve).await {
            Ok(data) => {
                if let Ok(curve) = PumpfunBondingCurve::try_from_slice(&data.data[8..81]) {
                    if !curve.complete {
                        let curve_keys = BondingCurveAccounts {
                            mint: *mint,
                            bonding_curve: bonding_curve,
                            creator: curve.creator,
                        };
                        pumpfun_keys.push(curve_keys);
                    }
                }
            }
            Err(_) => {}
        }
    }

    for curve_accounts in &pumpfun_keys {
        let mut swap_accounts =
            PumpFunSwapAccounts::from_bonding_curve_accounts(curve_accounts.clone());
        match RPC_CLIENT
            .get_token_account_balance(&swap_accounts.associated_user)
            .await
        {
            Ok(ui_token_amount) => {
                if let Ok(balance) = ui_token_amount.amount.parse::<u64>() {
                    if balance > 0 {
                        println!("Balance for mint {}: {}", curve_accounts.mint, balance);

                        let mut ixs = Vec::new();
                        let create_ix: Instruction = swap_accounts.get_create_ata_idempotent_ix();
                        let swap_ix = swap_accounts.get_sell_ix(balance);

                        ixs.push(create_ix);
                        ixs.push(swap_ix);

                        let _ = send_zero_slot_transaction(
                            ixs,
                            format!("[AUTO_TURN_OFF]\t*Sell\t*Mint: {}", curve_accounts.mint),
                        )
                        .await;

                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
            Err(_) => {}
        }
    }

    info!("AUTO TURN OFF BOT PROCESS COMPLETE");
    Ok(())
}
