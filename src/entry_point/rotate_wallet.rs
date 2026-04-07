use colored::*;
use pumpfun_sniper::*;

#[tokio::main]
async fn main() {
    println!("{}", "\n  🔄 Wallet Rotation - Cross Chain Swap".yellow().bold());
    println!("{}", "  ─────────────────────────────────────".bright_black());
    match rotate_wallet().await {
        Ok(_) => println!("{}", "\n  ✅ Wallet rotation complete.".green()),
        Err(e) => println!("{}", format!("\n  ❌ Wallet rotation failed: {}", e).red()),
    }
}
