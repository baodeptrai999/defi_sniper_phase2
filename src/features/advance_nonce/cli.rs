use crate::*;
use colored::*;
#[allow(deprecated)]
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::io::{self, Write};

use super::nonce_file::*;

pub async fn nonce_management_menu() {
    loop {
        println!();
        println!("{}", "═══════════════════════════════════════════".cyan());
        println!("  {}", "Advance Nonce Management".cyan().bold());
        println!("{}", "═══════════════════════════════════════════".cyan());
        println!("  {} Create nonce accounts", "[ 1. ]".green());
        println!("  {} View nonce status", "[ 2. ]".green());
        println!("  {} Close nonce accounts (reclaim SOL)", "[ 3. ]".green());
        println!("  {} Back", "[ 0. ]".red());
        println!("{}", "═══════════════════════════════════════════".cyan());
        print!("\n  {} ", "Select option >>".yellow());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        match input {
            "1" => create_nonce_accounts_cli().await,
            "2" => show_nonce_status().await,
            "3" => close_nonce_accounts_cli().await,
            "0" => break,
            _ => println!("{}", "  ⚠ Invalid option.".red()),
        }
    }
}

async fn create_nonce_accounts_cli() {
    print!(
        "\n  {} ",
        "How many nonce accounts to create? (Suggested: 20) >>".yellow()
    );
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let count: usize = match input.trim().parse() {
        Ok(n) if n > 0 && n <= 50 => n,
        _ => {
            println!("{}", "  ⚠ Enter a number between 1 and 50.".red());
            return;
        }
    };

    let total_cost_lamports = count as u64 * NONCE_RENT_LAMPORTS;
    let total_cost_sol = total_cost_lamports as f64 / 1e9;

    println!(
        "\n  {} Creating {} nonce accounts (cost: ~{:.6} SOL for rent)",
        "📝",
        count,
        total_cost_sol
    );

    // Check wallet balance
    match RPC_CLIENT.get_balance(&*SIGNER_PUBKEY).await {
        Ok(balance) => {
            let balance_sol = balance as f64 / 1e9;
            println!("  {} Wallet balance: {:.6} SOL", "💰", balance_sol);
            if balance < total_cost_lamports + 10_000 * count as u64 {
                println!(
                    "{}",
                    "  ⚠ Insufficient balance for nonce account creation.".red()
                );
                return;
            }
        }
        Err(e) => {
            println!("{}", format!("  ⚠ Failed to check balance: {}", e).red());
            return;
        }
    }

    let mut existing = load_nonce_pubkeys();
    let mut created = 0usize;

    for i in 0..count {
        let nonce_keypair = Keypair::new();
        let nonce_pubkey = nonce_keypair.pubkey();

        let create_ixs = system_instruction::create_nonce_account(
            &*SIGNER_PUBKEY,
            &nonce_pubkey,
            &*SIGNER_PUBKEY,
            NONCE_RENT_LAMPORTS,
        );

        let blockhash = match RPC_CLIENT.get_latest_blockhash().await {
            Ok(h) => h,
            Err(e) => {
                println!(
                    "{}",
                    format!("  ⚠ Failed to get blockhash: {}. Stopping.", e).red()
                );
                break;
            }
        };

        let tx = Transaction::new_signed_with_payer(
            &create_ixs,
            Some(&*SIGNER_PUBKEY),
            &[&SIGNER_KEYPAIR.insecure_clone(), &nonce_keypair],
            blockhash,
        );

        match RPC_CLIENT.send_and_confirm_transaction(&tx).await {
            Ok(sig) => {
                existing.push(nonce_pubkey);
                created += 1;
                println!(
                    "  {} [{}/{}] Created nonce account: {} (sig: {})",
                    "✅".green(),
                    i + 1,
                    count,
                    nonce_pubkey,
                    sig
                );
            }
            Err(e) => {
                println!(
                    "  {} [{}/{}] Failed to create nonce account: {}",
                    "❌".red(),
                    i + 1,
                    count,
                    e
                );
            }
        }
    }

    if created > 0 {
        save_nonce_pubkeys(&existing);
        println!(
            "\n  {} Created {}/{} nonce accounts. Saved to {}",
            "✅".green(),
            created,
            count,
            NONCE_ACCOUNTS_PATH
        );
    }
}

pub async fn show_nonce_status() {
    let pubkeys = load_nonce_pubkeys();
    if pubkeys.is_empty() {
        println!("{}", "\n  ⚠ No nonce accounts found.".yellow());
        return;
    }

    println!(
        "\n  {} Nonce accounts ({} total):",
        "📋",
        pubkeys.len()
    );
    println!("  {}", "─".repeat(80));

    for (i, pk) in pubkeys.iter().enumerate() {
        match RPC_CLIENT.get_account(pk).await {
            Ok(account) => {
                let balance_sol = account.lamports as f64 / 1e9;
                let hash = parse_nonce_hash_from_data(&account.data);
                let hash_str = hash
                    .map(|h| format!("{}", h))
                    .unwrap_or_else(|| "INVALID".red().to_string());
                println!(
                    "  [{}] {} | {:.6} SOL | Nonce: {}",
                    i + 1,
                    pk,
                    balance_sol,
                    &hash_str[..16.min(hash_str.len())]
                );
            }
            Err(_) => {
                println!(
                    "  [{}] {} | {}",
                    i + 1,
                    pk,
                    "UNREACHABLE".red()
                );
            }
        }
    }
    println!("  {}", "─".repeat(80));
}

async fn close_nonce_accounts_cli() {
    let pubkeys = load_nonce_pubkeys();
    if pubkeys.is_empty() {
        println!("{}", "\n  ⚠ No nonce accounts to close.".yellow());
        return;
    }

    println!(
        "\n  {} This will close ALL {} nonce accounts and reclaim SOL.",
        "⚠".yellow(),
        pubkeys.len()
    );
    print!("  {} ", "Type 'yes' to confirm >>".red());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim() != "yes" {
        println!("  Cancelled.");
        return;
    }

    let mut remaining: Vec<Pubkey> = Vec::new();

    for pk in &pubkeys {
        let withdraw_ix = system_instruction::withdraw_nonce_account(
            pk,
            &*SIGNER_PUBKEY,
            &*SIGNER_PUBKEY,
            NONCE_RENT_LAMPORTS,
        );

        let blockhash = match RPC_CLIENT.get_latest_blockhash().await {
            Ok(h) => h,
            Err(_) => {
                remaining.push(*pk);
                continue;
            }
        };

        let tx = Transaction::new_signed_with_payer(
            &[withdraw_ix],
            Some(&*SIGNER_PUBKEY),
            &[&SIGNER_KEYPAIR.insecure_clone()],
            blockhash,
        );

        match RPC_CLIENT.send_and_confirm_transaction(&tx).await {
            Ok(sig) => {
                println!("  {} Closed {} (sig: {})", "✅".green(), pk, sig);
            }
            Err(e) => {
                println!("  {} Failed to close {}: {}", "❌".red(), pk, e);
                remaining.push(*pk);
            }
        }
    }

    save_nonce_pubkeys(&remaining);
    println!(
        "\n  {} Done. {} accounts remaining.",
        "✅".green(),
        remaining.len()
    );
}
