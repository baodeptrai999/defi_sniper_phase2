use crate::*;
use futures::FutureExt;
use futures::future::BoxFuture;
use solana_relayer_adapter_rust::*;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::{instruction::Instruction, native_token::lamports_to_sol};
use tokio::time::{Duration, sleep};

// --- Your Nozomi, ZSlot, Jito clients assumed imported here ---
// --- Your get_slot(), PRIORITY_FEE, PUBKEY, PRIVATE_KEY, CONFIRM_SERVICE, etc. ---
// --- LOG macro assumed ---

#[derive(PartialEq)]
pub enum TradeType {
    Buy,
    Sell,
}

pub fn confirm(
    raw_instructions: Vec<Instruction>,
    // tag: String,
    // force_sell: bool,
    // trade_type: TradeType,
    // buy_sol_lamport: u64,
    // temp: usize,
) -> BoxFuture<'static, Option<Signature>> {
    async move {
        let (cu, priority_fee_micro_lamport, third_party_fee) = *PRIORITY_FEE;

        let results = match CONFIRM_SERVICE.as_str() {
            "NOZOMI" => {
                let nozomi = NOZOMI_CLIENT.get().expect("Nozomi client not initialized");
                let ixs = nozomi.add_tip_ix(Tips {
                    cu: Some(cu),
                    priority_fee_micro_lamport: Some(priority_fee_micro_lamport),
                    payer: *SIGNER_PUBKEY,
                    pure_ix: raw_instructions.clone(),
                    tip_addr_idx: 1,
                    tip_sol_amount: third_party_fee,
                });
                let recent_blockhash = get_slot();
                let encoded_tx = nozomi.build_v0_bs64(
                    ixs,
                    &*SIGNER_PUBKEY,
                    &vec![&*SIGNER_KEYPAIR],
                    recent_blockhash,
                    None,
                );
                match nozomi.send_transaction(&encoded_tx).await {
                    Ok(data) => data.result,
                    Err(err) => Some(err.to_string()),
                }
            }
            "ZERO_SLOT" => {
                let zero_slot = ZERO_SLOT_CLIENT
                    .get()
                    .expect("ZSlot client not initialized");
                let ixs = zero_slot.add_tip_ix(Tips {
                    cu: Some(cu),
                    priority_fee_micro_lamport: Some(priority_fee_micro_lamport),
                    payer: *SIGNER_PUBKEY,
                    pure_ix: raw_instructions.clone(),
                    tip_addr_idx: 1,
                    tip_sol_amount: third_party_fee,
                });
                let recent_blockhash = get_slot();
                let encoded_tx = zero_slot.build_v0_bs64(
                    ixs,
                    &*SIGNER_PUBKEY,
                    &vec![&*SIGNER_KEYPAIR],
                    recent_blockhash,
                    None,
                );
                match zero_slot.send_transaction(&encoded_tx).await {
                    Ok(data) => data.result,
                    Err(err) => Some(err.to_string()),
                }
            }
            "JITO" => {
                let jito = JITO_CLIENT.get().expect("Jito client not initialized");
                let ixs = jito.add_tip_ix(Tips {
                    cu: Some(cu),
                    priority_fee_micro_lamport: Some(priority_fee_micro_lamport),
                    payer: *SIGNER_PUBKEY,
                    pure_ix: raw_instructions.clone(),
                    tip_addr_idx: 1,
                    tip_sol_amount: third_party_fee,
                });
                let recent_blockhash = get_slot();
                println!("recent blockhash --- {:?}", recent_blockhash);
                let encoded_tx = jito.build_v0_bs64(
                    ixs,
                    &*SIGNER_PUBKEY,
                    &vec![&*SIGNER_KEYPAIR],
                    recent_blockhash,
                    None,
                );
                match jito.send_transaction(&encoded_tx).await {
                    Ok(data) => data.result,
                    Err(err) => Some(err.to_string()),
                }
            }
            _ => Some("unknown confirmation service".to_string()),
        };

        info!(
            "[SUBMIT]
                \t* Service: Jito
                \t* Hash : {:?}
                \t* Force_sell : ok
                \t* ok",
            results, 
            // force_sell, tag
        );

        if let Some(signature_str) = results {
            if let Some(confirmed_sig) = wait_for_confirmation(&signature_str).await {
                return Some(confirmed_sig);
            } else {
                // Recursive retry
                return confirm(
                    raw_instructions.clone(),
                    // tag.clone(),
                    // force_sell,
                    // trade_type,
                    // buy_sol_lamport,
                    // temp,
                )
                .await;
            }
        }

        if let Some(result_raw) = results {
            match result_raw.parse::<Signature>() {
                Ok(sig) => {
                    success!(
                        "[CONFIRM]
                            \t* Hash : {}
                            \t* Force_sell : ok
                            \t* ok
                            ",
                        sig.to_string(),
                        // force_sell,
                        // tag
                    );

                    Some(sig)
                }
                Err(_) => None,
            }
        } else {
            None
        }
    }
    .boxed()
}

pub async fn wait_for_confirmation(signature_str: &str) -> Option<Signature> {
    let signature = match signature_str.parse::<Signature>() {
        Ok(sig) => sig,
        Err(_) => {
            error!(
                "[FORCE_CHECK]
                \t* Hash : {}
                \t* States : Invalid signature
                \t* ",
                signature_str,
                //  tag
            );

            return None;
        }
    };

    let mut attempts = 0;

    loop {
        match RPC_CLIENT.get_signature_statuses(&[signature]) {
            Ok(statuses) => {
                if let Some(Some(status)) = statuses.value.get(0) {
                    if status.confirmations.is_none() || status.confirmations.unwrap_or(0) > 0 {
                        success!(
                            "[FORCE_CHECK]
                            \t* Hash : {}
                            \t* States : Confirmed
                            \t* Check : https://solscan.io/tx/{}
                            \t* ",
                            signature,
                            signature,
                            // tag
                        );
                        return Some(signature);
                    } else {
                        // err!(
                        //     "[FORCE_CHECK] => HASH : {}
                        //     \t* STATES : Not Yet Confirmed
                        //     \t* {}",
                        //     signature,
                        //     tag
                        // );
                    }
                } else {
                    // err!(
                    //     "[FORCE_CHECK] => HASH : {}
                    //     \t* STATES : States Not Found
                    //     \t* {}",
                    //     signature,
                    //     tag
                    // );
                }
            }
            Err(err) => {
                // err!(
                //     "[FORCE_CHECK]
                //     \t* HASH : {}
                //     \t* STATES : Error Fetching Status
                //     \t* {}",
                //     signature,
                //     tag
                // );
            }
        }

        attempts += 1;
        if attempts >= 15 {
            error!(
                "[FORCE_CHECK]
                \t* Hash : {}
                \t* States : Failed
                \t* Check : https://solscan.io/tx/{}
                \t* ",
                signature, signature, 
                // tag
            );
            return None;
        }

        sleep(Duration::from_secs(2)).await;
    }
}
