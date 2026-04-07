mod bridge;
mod save;
mod transfer_tx;

use crate::*;
use bridge::{bridge_bnb_to_sol, bridge_sol_to_bnb};
use save::save_rotation_json;

use solana_sdk::{native_token::LAMPORTS_PER_SOL, signer::Signer};

pub async fn rotate_wallet() -> Result<(), Box<dyn std::error::Error>> {
    info!("═══════════════════════════════════════════");
    info!("          WALLET ROTATION START");
    info!("═══════════════════════════════════════════");

    // ── Config ──
    let bnb_rpc_url = BNB_RPC_ENDPOINT.clone();
    if bnb_rpc_url.is_empty() {
        error!("[ROTATE] bnb_rpc_endpoint not configured in Config.toml");
        return Err("bnb_rpc_endpoint not configured".into());
    }

    // ── Old wallet ──
    let old_keypair = SIGNER_KEYPAIR.insecure_clone();
    let old_pubkey = old_keypair.pubkey();
    let old_pubkey_str = old_pubkey.to_string();
    let old_private_key = bs58::encode(old_keypair.to_bytes()).into_string();
    let old_balance = RPC_CLIENT.get_balance(&old_pubkey).await?;
    let old_balance_sol = old_balance as f64 / LAMPORTS_PER_SOL as f64;

    info!("[ROTATE] Old Wallet:      {}", old_pubkey);
    info!("[ROTATE] Old Private Key: {}", old_private_key);
    info!("[ROTATE] Old Balance:     {:.9} SOL", old_balance_sol);

    if old_balance < MIN_SOL_LAMPORTS {
        error!("[ROTATE] Insufficient SOL balance: {:.9} SOL (min 0.01)", old_balance_sol);
        return Err("Insufficient SOL balance".into());
    }

    let transfer_lamports = old_balance - FEE_RESERVE_LAMPORTS;
    info!("[ROTATE] Bridging:        {:.9} SOL", transfer_lamports as f64 / LAMPORTS_PER_SOL as f64);

    // ── Step 1: SOL → BNB ──
    info!("───────────────────────────────────────────");
    info!("  SOL → BNB");
    info!("───────────────────────────────────────────");

    let sol_to_bnb = match bridge_sol_to_bnb(&old_keypair, transfer_lamports).await {
        Ok(result) => result,
        Err(e) => {
            error!("[ROTATE] SOL → BNB failed: {}", e);
            let _ = save_rotation_json(
                "failed_sol_to_bnb",
                Some(&e.to_string()),
                &old_pubkey_str,
                &old_private_key,
                None, None, None, None,
            );
            return Err(e);
        }
    };

    let bnb_addr_str = format!("{}", sol_to_bnb.bnb_address);

    info!("[ROTATE] BNB Address:     {}", bnb_addr_str);
    info!("[ROTATE] BNB Private Key: 0x{}", sol_to_bnb.bnb_private_key);
    for sig in &sol_to_bnb.sol_signatures {
        info!("[ROTATE] SOL TX:          {}", sig);
    }
    info!("[ROTATE] SOL → BNB complete ✓");

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // ── Step 2: BNB → SOL ──
    info!("───────────────────────────────────────────");
    info!("  BNB → SOL");
    info!("───────────────────────────────────────────");

    let bnb_to_sol = match bridge_bnb_to_sol(
        &sol_to_bnb.bnb_signer,
        sol_to_bnb.bnb_address,
        &bnb_rpc_url,
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            error!("[ROTATE] BNB → SOL failed: {}", e);
            let _ = save_rotation_json(
                "failed_bnb_to_sol",
                Some(&e.to_string()),
                &old_pubkey_str,
                &old_private_key,
                Some(&bnb_addr_str),
                Some(&sol_to_bnb.bnb_private_key),
                None, None,
            );
            return Err(e);
        }
    };

    info!("[ROTATE] BNB Balance:     {:.6} BNB", bnb_to_sol.bnb_balance_wei.to::<u128>() as f64 / 1e18);
    info!("[ROTATE] New Wallet:      {}", bnb_to_sol.new_sol_pubkey);
    info!("[ROTATE] New Private Key: {}", bnb_to_sol.new_sol_private_key);
    for (hash, block) in &bnb_to_sol.bnb_tx_results {
        info!("[ROTATE] BNB TX:          {} (block {:?})", hash, block);
    }
    info!("[ROTATE] BNB → SOL complete ✓");

    // ── Final balance ──
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let new_balance = RPC_CLIENT
        .get_balance(&bnb_to_sol.new_sol_pubkey)
        .await
        .unwrap_or(0);
    let new_balance_sol = new_balance as f64 / LAMPORTS_PER_SOL as f64;

    info!("═══════════════════════════════════════════");
    info!("          ROTATION COMPLETE");
    info!("═══════════════════════════════════════════");
    info!("[ROTATE] Old Wallet:      {}", old_pubkey);
    info!("[ROTATE] New Wallet:      {}", bnb_to_sol.new_sol_pubkey);
    info!("[ROTATE] New Private Key: {}", bnb_to_sol.new_sol_private_key);
    info!("[ROTATE] New Balance:     {:.9} SOL", new_balance_sol);
    info!("");
    info!("⚠ Update Config.toml → private_key = \"{}\"", bnb_to_sol.new_sol_private_key);

    // ── Save JSON (success) ──
    save_rotation_json(
        "success",
        None,
        &old_pubkey_str,
        &old_private_key,
        Some(&bnb_addr_str),
        Some(&sol_to_bnb.bnb_private_key),
        Some(&bnb_to_sol.new_sol_pubkey.to_string()),
        Some(&bnb_to_sol.new_sol_private_key),
    )?;

    Ok(())
}
