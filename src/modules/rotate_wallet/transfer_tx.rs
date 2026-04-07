use crate::*;

#[allow(deprecated)]
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    instruction::{AccountMeta, Instruction},
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::VersionedTransaction,
};
use std::str::FromStr;
use alloy::{
    primitives::{Address, U256},
    providers::Provider,
};

/// Build and submit all Solana transactions from relay quote steps.
/// Returns the list of tx signatures.
pub async fn execute_solana_steps(
    quote: &serde_json::Value,
    signer: &Keypair,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let steps = quote["steps"]
        .as_array()
        .ok_or("No steps in quote")?;

    let mut signatures = Vec::new();
    for step in steps {
        let items = step["items"]
            .as_array()
            .ok_or("No items in step")?;

        for item in items {
            let instructions = parse_instructions(item)?;
            let lookup_tables = fetch_lookup_tables(item).await?;

            let recent_blockhash = RPC_CLIENT.get_latest_blockhash().await?;
            let msg = v0::Message::try_compile(
                &signer.pubkey(),
                &instructions,
                &lookup_tables,
                recent_blockhash,
            )?;
            let tx = VersionedTransaction::try_new(
                VersionedMessage::V0(msg),
                &[signer],
            )?;

            let sig = RPC_CLIENT.send_and_confirm_transaction(&tx).await?;
            signatures.push(sig.to_string());
        }
    }

    Ok(signatures)
}

/// Parse Solana instructions from a relay quote item JSON.
fn parse_instructions(
    item: &serde_json::Value,
) -> Result<Vec<Instruction>, Box<dyn std::error::Error>> {
    let instructions_json = item["data"]["instructions"]
        .as_array()
        .ok_or("No instructions in quote item data")?;

    let mut instructions = Vec::new();
    for ix_json in instructions_json {
        let program_id = Pubkey::from_str(
            ix_json["programId"]
                .as_str()
                .ok_or("No programId in instruction")?,
        )?;

        let keys_json = ix_json["keys"]
            .as_array()
            .ok_or("No keys in instruction")?;

        let mut accounts = Vec::new();
        for key in keys_json {
            let pubkey = Pubkey::from_str(
                key["pubkey"].as_str().ok_or("No pubkey in key")?,
            )?;
            let is_signer = key["isSigner"].as_bool().unwrap_or(false);
            let is_writable = key["isWritable"].as_bool().unwrap_or(false);
            if is_writable {
                accounts.push(AccountMeta::new(pubkey, is_signer));
            } else {
                accounts.push(AccountMeta::new_readonly(pubkey, is_signer));
            }
        }

        let data_hex = ix_json["data"]
            .as_str()
            .ok_or("No data in instruction")?;
        let data = hex::decode(data_hex.trim_start_matches("0x"))?;

        instructions.push(Instruction {
            program_id,
            accounts,
            data,
        });
    }

    Ok(instructions)
}

/// Fetch address lookup tables referenced in a relay quote item (concurrent).
async fn fetch_lookup_tables(
    item: &serde_json::Value,
) -> Result<Vec<AddressLookupTableAccount>, Box<dyn std::error::Error>> {
    let alt_addrs = match item["data"]["addressLookupTableAddresses"].as_array() {
        Some(arr) => arr,
        None => return Ok(vec![]),
    };

    let pubkeys: Vec<Pubkey> = alt_addrs
        .iter()
        .map(|v| Pubkey::from_str(v.as_str().unwrap_or_default()))
        .collect::<Result<_, _>>()?;

    let futures: Vec<_> = pubkeys
        .iter()
        .map(|pk| async move {
            let account = RPC_CLIENT.get_account(pk).await?;
            let lookup_table =
                solana_sdk::address_lookup_table::state::AddressLookupTable::deserialize(
                    &account.data,
                )?;
            Ok::<_, Box<dyn std::error::Error>>(AddressLookupTableAccount {
                key: *pk,
                addresses: lookup_table.addresses.to_vec(),
            })
        })
        .collect();

    futures::future::try_join_all(futures).await
}

/// Submit all BNB transactions from relay quote steps.
/// Returns list of (tx_hash, block_number) pairs.
pub async fn execute_bnb_steps<P: Provider>(
    quote: &serde_json::Value,
    provider: &P,
) -> Result<Vec<(String, Option<u64>)>, Box<dyn std::error::Error>> {
    let steps = quote["steps"]
        .as_array()
        .ok_or("No steps in quote")?;

    let mut results = Vec::new();
    for step in steps {
        let items = step["items"]
            .as_array()
            .ok_or("No items in step")?;

        for item in items {
            let data = &item["data"];
            let to = Address::from_str(
                data["to"].as_str().ok_or("No 'to' in tx data")?,
            )?;

            let value = match data["value"].as_str() {
                Some(v) => U256::from_str(v)?,
                None => match data["value"].as_u64() {
                    Some(v) => U256::from(v),
                    None => U256::ZERO,
                },
            };

            let mut tx_req = alloy::rpc::types::TransactionRequest::default()
                .to(to)
                .value(value);

            if let Some(tx_data_hex) = data["data"].as_str() {
                let call_data = hex::decode(tx_data_hex.trim_start_matches("0x"))?;
                if !call_data.is_empty() {
                    tx_req = tx_req.input(call_data.into());
                }
            }

            // Gas params from relay (handles both string and number JSON types)
            if let Some(gas) = parse_json_u64(&data["gas"]) {
                tx_req = tx_req.gas_limit(gas);
            }
            if let Some(v) = parse_json_u128(&data["maxFeePerGas"]) {
                tx_req = tx_req.max_fee_per_gas(v);
            }
            if let Some(v) = parse_json_u128(&data["maxPriorityFeePerGas"]) {
                tx_req = tx_req.max_priority_fee_per_gas(v);
            }
            if let Some(v) = parse_json_u128(&data["gasPrice"]) {
                tx_req = tx_req.gas_price(v);
            }

            let pending = provider.send_transaction(tx_req).await?;
            let tx_hash = pending.tx_hash().to_string();
            let receipt = pending.get_receipt().await?;
            results.push((tx_hash, receipt.block_number));
        }
    }

    Ok(results)
}

fn parse_json_u64(val: &serde_json::Value) -> Option<u64> {
    val.as_u64()
        .or_else(|| val.as_str().and_then(|s| s.parse().ok()))
}

fn parse_json_u128(val: &serde_json::Value) -> Option<u128> {
    val.as_u64()
        .map(|v| v as u128)
        .or_else(|| val.as_str().and_then(|s| s.parse().ok()))
}
