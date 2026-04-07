use colored::*;
use pumpfun_sniper::*;

#[tokio::main]
async fn main() {
    init_nonce_pool().await;
    println!("{}", "\n  🔄 Executing All Sell...".yellow());
    match all_sell().await {
        Ok(_) => println!("{}", "  ✅ All Sell complete.".green()),
        Err(e) => println!("{}", format!("  ❌ All Sell failed: {}", e).red()),
    }
}
