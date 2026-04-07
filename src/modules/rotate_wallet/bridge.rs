use super::transfer_tx::{execute_bnb_steps, execute_solana_steps};
use crate::*;

use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
};
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};

pub struct SolToBnbResult {
    pub bnb_signer: PrivateKeySigner,
    pub bnb_address: Address,
    pub bnb_private_key: String,
    pub sol_signatures: Vec<String>,
}

pub struct BnbToSolResult {
    pub new_sol_pubkey: Pubkey,
    pub new_sol_private_key: String,
    pub bnb_balance_wei: U256,
    pub bnb_tx_results: Vec<(String, Option<u64>)>,
}

/// Bridge SOL → native BNB via relay.link
pub async fn bridge_sol_to_bnb(
    old_keypair: &Keypair,
    transfer_lamports: u64,
) -> Result<SolToBnbResult, Box<dyn std::error::Error>> {
    let bnb_signer = PrivateKeySigner::random();
    let bnb_address = bnb_signer.address();
    let bnb_private_key = hex::encode(bnb_signer.credential().to_bytes());

    let quote = get_relay_quote(
        &old_keypair.pubkey().to_string(),
        SOLANA_CHAIN_ID,
        BNB_CHAIN_ID,
        SOL_NATIVE_CURRENCY,
        BNB_NATIVE_CURRENCY,
        &transfer_lamports.to_string(),
        &format!("{}", bnb_address),
    )
    .await?;

    let request_id = quote["steps"][0]["requestId"]
        .as_str()
        .ok_or("No requestId in SOL→BNB quote")?
        .to_string();

    let sol_signatures = execute_solana_steps(&quote, old_keypair).await?;

    if !wait_for_bridge(&request_id, "SOL→BNB").await? {
        return Err("SOL→BNB bridge failed or timed out".into());
    }

    Ok(SolToBnbResult {
        bnb_signer,
        bnb_address,
        bnb_private_key,
        sol_signatures,
    })
}

/// Bridge native BNB → SOL via relay.link
pub async fn bridge_bnb_to_sol(
    bnb_signer: &PrivateKeySigner,
    bnb_address: Address,
    bnb_rpc_url: &str,
) -> Result<BnbToSolResult, Box<dyn std::error::Error>> {
    let bnb_wallet = EthereumWallet::from(bnb_signer.clone());
    let bnb_provider = ProviderBuilder::new()
        .wallet(bnb_wallet)
        .connect_http(bnb_rpc_url.parse()?);

    let bnb_balance = bnb_provider.get_balance(bnb_address).await?;
    if bnb_balance == U256::ZERO {
        return Err("No BNB received from bridge".into());
    }

    let gas_reserve = U256::from(BNB_GAS_RESERVE_WEI);
    if bnb_balance <= gas_reserve {
        return Err("BNB balance too low to cover gas reserve".into());
    }
    let quote_amount = bnb_balance - gas_reserve;

    let new_sol_keypair = Keypair::new();
    let new_sol_pubkey = new_sol_keypair.pubkey();
    let new_sol_private_key = bs58::encode(new_sol_keypair.to_bytes()).into_string();

    let quote = get_relay_quote(
        &format!("{}", bnb_address),
        BNB_CHAIN_ID,
        SOLANA_CHAIN_ID,
        BNB_NATIVE_CURRENCY,
        SOL_NATIVE_CURRENCY,
        &quote_amount.to_string(),
        &new_sol_pubkey.to_string(),
    )
    .await?;

    let request_id = quote["steps"][0]["requestId"]
        .as_str()
        .ok_or("No requestId in BNB→SOL quote")?
        .to_string();

    let bnb_tx_results = execute_bnb_steps(&quote, &bnb_provider).await?;

    if !wait_for_bridge(&request_id, "BNB→SOL").await? {
        return Err("BNB→SOL bridge failed or timed out".into());
    }

    Ok(BnbToSolResult {
        new_sol_pubkey,
        new_sol_private_key,
        bnb_balance_wei: bnb_balance,
        bnb_tx_results,
    })
}

/// Get a bridge quote from relay.link v2
async fn get_relay_quote(
    user: &str,
    origin_chain: u64,
    dest_chain: u64,
    origin_currency: &str,
    dest_currency: &str,
    amount: &str,
    recipient: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let body = serde_json::json!({
        "user": user,
        "originChainId": origin_chain,
        "destinationChainId": dest_chain,
        "originCurrency": origin_currency,
        "destinationCurrency": dest_currency,
        "amount": amount,
        "tradeType": "EXACT_INPUT",
        "recipient": recipient
    });

    let resp = reqwest::Client::new()
        .post(RELAY_QUOTE_URL)
        .json(&body)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(resp)
}

/// Poll relay.link until bridge is "success", "failed", or timeout.
async fn wait_for_bridge(
    request_id: &str,
    label: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();

    loop {
        if start.elapsed().as_millis() as u64 > BRIDGE_TIMEOUT_MS {
            error!("[ROTATE] {} TIMEOUT after {}s", label, BRIDGE_TIMEOUT_MS / 1000);
            return Ok(false);
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(BRIDGE_POLL_INTERVAL_MS)).await;

        let url = format!("{}?requestId={}", RELAY_STATUS_URL, request_id);
        match client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    let status = json["status"].as_str().unwrap_or("unknown");
                    if status == "success" {
                        return Ok(true);
                    }
                    if status == "failed" || status == "refund" {
                        error!("[ROTATE] {} bridge failed: {}", label, status);
                        return Ok(false);
                    }
                }
            }
            Err(e) => {
                error!("[ROTATE] {} poll error: {}", label, e);
            }
        }
    }
}
