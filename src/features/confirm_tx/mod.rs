use crate::*;
use futures::FutureExt;
use futures::future::BoxFuture;
use solana_relayer_adapter_rust::*;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::Signature;
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
    tag: String,
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
                // let zero_slot = ZERO_SLOT_CLIENT
                //     .get()
                //     .expect("ZSlot client not initialized");
                // let ixs = zero_slot.add_tip_ix(Tips {
                //     cu: Some(cu),
                //     priority_fee_micro_lamport: Some(priority_fee_micro_lamport),
                //     payer: *SIGNER_PUBKEY,
                //     pure_ix: raw_instructions.clone(),
                //     tip_addr_idx: 1,
                //     tip_sol_amount: third_party_fee,
                // });
                // let recent_blockhash = get_slot();
                // let encoded_tx = zero_slot.build_v0_bs64(
                //     ixs,
                //     &*SIGNER_PUBKEY,
                //     &vec![&*SIGNER_KEYPAIR],
                //     recent_blockhash,
                //     None,
                // );
                // match zero_slot.send_transaction(&encoded_tx).await {
                //     Ok(data) => data.result,
                //     Err(err) => Some(err.to_string()),
                // }
                send_zero_slot_transaction(raw_instructions, tag.clone()).await
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
                \t* Service: {}
                \t* Hash: {:?}
                \t* {}",
            *CONFIRM_SERVICE,
            results,
            tag.clone()
        );

        if let Some(signature_str) = results {
            if let Some(confirmed_sig) = wait_for_confirmation(&signature_str, tag.clone()).await {
                return Some(confirmed_sig);
            } else {
                // Recursive retry
                // return confirm(raw_instructions.clone(), tag.clone()).await;
                return None;
            }
        }

        if let Some(result_raw) = results {
            match result_raw.parse::<Signature>() {
                Ok(sig) => {
                    success!(
                        "[CONFIRM]
                            \t* CHECK : {}
                            \t* {}",
                        solscan!(sig.to_string()),
                        tag.clone()
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

pub async fn wait_for_confirmation(signature_str: &str, tag: String) -> Option<Signature> {
    let trimed_clean_sig = signature_str
        .trim()
        .replace("\"", "")
        .replace("'", "")
        .replace("\n", "")
        .replace("\r", "");
    let signature = match trimed_clean_sig.parse::<Signature>() {
        Ok(sig) => sig,
        Err(_) => {
            error!(
                "[FORCE_CHECK]
                \t* Check : {}
                \t* {}
                \t* States : Invalid signature",
                solscan!(signature_str),
                tag.clone()
            );

            return None;
        }
    };

    let mut attempts = 0;

    loop {
        match RPC_CLIENT.get_signature_statuses(&[signature]).await {
            Ok(statuses) => {
                if let Some(Some(status)) = statuses.value.get(0) {
                    if status.confirmations.is_none() || status.confirmations.unwrap_or(0) > 0 {
                        success!(
                            "[FORCE_CHECK]
                            \t* Check : {}
                            \t* States : Confirmed
                            \t* {}",
                            solscan!(signature),
                            tag
                        );
                        return Some(signature);
                    }
                }
            }
            Err(_) => {}
        }

        attempts += 1;
        if attempts >= 10 {
            error!(
                "[FORCE_CHECK]
                \t* Check : https://solscan.io/tx/{}
                \t* States : Failed
                \t* {}",
                signature,
                tag.clone()
            );
            return None;
        }

        sleep(Duration::from_secs(2)).await;
    }
}
