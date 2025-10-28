use crate::*;
use base64;
use serde_json::json;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, pubkey::Pubkey,
    system_instruction, transaction::Transaction,
};
use std::str::FromStr;
use std::time::Instant;

pub fn init_http_client() {
    let _client = &HTTP_CLIENT;
}

pub async fn send_zero_slot_transaction(
    raw_instructions: Vec<Instruction>,
    tag: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let (cu, priority_fee_micro_lamport, _third_party_fee) = *PRIORITY_FEE;

    let mut total_instruction = Vec::new();
    //budget compute unit limit
    total_instruction.push(ComputeBudgetInstruction::set_compute_unit_limit(cu as u32));
    //compute unit price
    total_instruction.push(ComputeBudgetInstruction::set_compute_unit_price(
        priority_fee_micro_lamport,
    ));
    //pure ix
    total_instruction.extend(raw_instructions);
    //tip ix
    let tip_receiver = Pubkey::from_str("TpdxgNJBWZRL8UXF5mrEsyWxDWx9HQexA9P1eTWQ42p").unwrap();
    let tip_transfer_instruction = system_instruction::transfer(
        &SIGNER_PUBKEY, // Sender's public key
        &tip_receiver,  // Tip receiver's public key
        100000,         // Amount to transfer as a tip (0.001 SOL in this case)
    );
    total_instruction.push(tip_transfer_instruction);
    let mut transaction = Transaction::new_with_payer(&total_instruction, Some(&SIGNER_PUBKEY));
    info!("Total ix build took: {:?}", start_time.elapsed());
    // Sign the transaction with the sender's keypair
    transaction
        .try_sign(&[SIGNER_KEYPAIR.insecure_clone()], get_slot())
        .expect("Failed to sign transaction");

    let serialized_transaction = bincode::serialize(&transaction).unwrap();
    let base64_encoded_transaction = base64::encode(serialized_transaction);

    info!("Signing and serializing took: {:?}", start_time.elapsed());

    // Build the JSON-RPC request
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
    info!("TX making: {:?}", start_time.elapsed());
    let tx_submission_start = Instant::now();
    let response = HTTP_CLIENT
        .post("http://la1.0slot.trade?api-key=335e371309b6492584368e9dc553622d")
        .json(&request_body)
        .send()
        .await?;
    let response_json: serde_json::Value = response.json().await?;
    if let Some(result) = response_json.get("result") {
        println!(
            "Transaction(zero slot) submission took: {:?}",
            tx_submission_start.elapsed()
        );
        info!(
            "[SUBMIT]
                \t* Service: ZERO_SLOT
                \t* Hash: {:?}
                \t* {}",
            result,
            tag.clone()
        );
    } else if let Some(error) = response_json.get("error") {
        eprintln!("Failed to send transaction: {}", error);
    }

    Ok(())
}
