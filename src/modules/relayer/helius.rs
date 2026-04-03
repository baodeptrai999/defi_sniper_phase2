use crate::*;
use base64;
use rand::RngExt;
use serde_json::json;
#[allow(deprecated)]
use solana_sdk::system_instruction;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, transaction::Transaction,
};
use std::time::Instant;

pub async fn send_helius_transaction(
    raw_instructions: Vec<Instruction>,
    tag: String,
) -> Option<String> {
    let nonce = match acquire_nonce() {
        Some(n) => n,
        None => {
            error!("[HELIUS] No nonce available | {}", tag);
            return None;
        }
    };

    let mut total_instruction = Vec::new();

    // Advance nonce MUST be the first instruction for durable nonce transactions
    total_instruction.push(nonce.advance_ix);

    total_instruction.push(ComputeBudgetInstruction::set_compute_unit_limit(
        *BUY_COMPUTE_UNIT_LIMIT as u32,
    ));
    total_instruction.push(ComputeBudgetInstruction::set_compute_unit_price(
        *BUY_MICRO_LAMPORTS,
    ));
    total_instruction.extend(raw_instructions);

    //tip ix
    let idx = rand::rng().random_range(0..HELIUS_TIP_ACCOUNTS.len());
    let tip_receiver = HELIUS_TIP_ACCOUNTS[idx];
    let tip_transfer_instruction = system_instruction::transfer(
        &SIGNER_PUBKEY,
        &tip_receiver,
        (*HELIUS_FEE * 10f64.powi(9)) as u64,
    );
    total_instruction.push(tip_transfer_instruction);

    let mut transaction = Transaction::new_with_payer(&total_instruction, Some(&SIGNER_PUBKEY));
    transaction
        .try_sign(&[SIGNER_KEYPAIR.insecure_clone()], nonce.nonce_hash)
        .expect("Failed to sign transaction");

    let serialized_transaction = bincode::serialize(&transaction).unwrap();
    let base64_encoded_transaction = base64::encode(serialized_transaction);

    let request_body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sendTransaction",
        "params": [
            base64_encoded_transaction,
            {
                "encoding": "base64",
                "skipPreflight": true,
            }
        ]
    });

    let tx_submission_start = Instant::now();
    let response = HTTP_CLIENT
        .post(&*HELIUS_ENDPOINT)
        .json(&request_body)
        .send()
        .await;

    match response {
        Ok(response_data) => {
            // TX bytes were sent to the endpoint — nonce may have been consumed
            spawn_nonce_refresh(nonce.index);

            let response_json: serde_json::Value = response_data.json().await.unwrap();
            if let Some(result) = response_json.get("result").and_then(|v| v.as_str()) {
                info!(
                    "[SUBMIT] HELIUS | {} | took {:?} | {}",
                    result,
                    tx_submission_start.elapsed(),
                    tag.clone()
                );
                return Some(result.to_string());
            } else {
                error!("[SUBMIT] HELIUS | no result | {:?}", response_json);
                return None;
            }
        }
        Err(e) => {
            // HTTP error — tx was never sent, nonce not consumed
            release_nonce(nonce.index);
            error!("[SUBMIT] HELIUS | HTTP error: {}", e);
            return None;
        }
    }
}
