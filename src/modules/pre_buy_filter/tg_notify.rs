/// Phase 2 — Telegram Notification Module
///
/// Sends filter decisions to a Telegram chat for real-time monitoring.
/// Uses the Telegram Bot API (sendMessage) with HTML formatting.
///
/// Config: Set TG_BOT_TOKEN and TG_CHAT_ID environment variables.

use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};

static TG_BOT_TOKEN: Lazy<String> = Lazy::new(|| {
    std::env::var("TG_BOT_TOKEN").unwrap_or_default()
});

static TG_CHAT_ID: Lazy<String> = Lazy::new(|| {
    std::env::var("TG_CHAT_ID").unwrap_or_default()
});

static TG_ENABLED: Lazy<AtomicBool> = Lazy::new(|| {
    let enabled = !TG_BOT_TOKEN.is_empty() && !TG_CHAT_ID.is_empty();
    AtomicBool::new(enabled)
});

/// Check if Telegram notifications are configured
pub fn tg_notify_enabled() -> bool {
    TG_ENABLED.load(Ordering::Relaxed)
}

/// Send a filter result notification to Telegram (non-blocking)
pub fn tg_send_filter_result(
    mint: &str,
    creator: &str,
    token_name: &str,
    passed: bool,
    risk_score: f64,
    buy_multiplier: f64,
    reasons: &[String],
) {
    if !tg_notify_enabled() {
        return;
    }

    let emoji = if !passed {
        "🔴"
    } else if risk_score > 0.0 {
        "🟡"
    } else {
        "🟢"
    };

    let status = if !passed {
        "REJECTED"
    } else if risk_score > 0.0 {
        "PASS (WARN)"
    } else {
        "PASS (CLEAN)"
    };

    let reasons_text = if reasons.is_empty() {
        "No issues".to_string()
    } else {
        reasons.iter()
            .map(|r| format!("• {}", r))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let multiplier_text = if passed && buy_multiplier < 1.0 {
        format!("\n💰 <b>Buy Multiplier:</b> {:.0}%", buy_multiplier * 100.0)
    } else {
        String::new()
    };

    let mint_short = if mint.len() > 12 {
        format!("{}...{}", &mint[..6], &mint[mint.len()-4..])
    } else {
        mint.to_string()
    };

    let message = format!(
        "{} <b>Phase 2 Filter: {}</b>\n\n\
         🪙 <b>Token:</b> {}\n\
         🔑 <b>Mint:</b> <code>{}</code>\n\
         👤 <b>Creator:</b> <code>{}</code>\n\
         ⚠️ <b>Risk Score:</b> {:.0}\n\
         {}\n\
         📋 <b>Details:</b>\n{}\n\n\
         🔗 <a href=\"https://pump.fun/{}\">Pump.fun</a> | <a href=\"https://solscan.io/token/{}\">Solscan</a>",
        emoji, status,
        if token_name.is_empty() { &mint_short } else { token_name },
        mint,
        creator,
        risk_score,
        multiplier_text,
        reasons_text,
        mint, mint,
    );

    let token = TG_BOT_TOKEN.clone();
    let chat_id = TG_CHAT_ID.clone();

    // Fire and forget — don't block the filter pipeline
    tokio::spawn(async move {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            token
        );

        let client = reqwest::Client::new();
        let _ = client.post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": message,
                "parse_mode": "HTML",
                "disable_web_page_preview": true,
            }))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;
    });
}

pub fn tg_send_trade_notification(is_buy: bool, mint: &str, tx_id: &str, amount_sol: f64) {
    if !tg_notify_enabled() {
        return;
    }

    let mint_str = mint.to_string();
    let tx_id_str = tx_id.to_string();
    let token_token = TG_BOT_TOKEN.clone();
    let chat_id = TG_CHAT_ID.clone();
    
    let action = if is_buy { "🟢 <b>BUY SUCCESS</b>" } else { "🔴 <b>SELL SUCCESS</b>" };

    tokio::spawn(async move {
        let msg = format!(
            "{}\n──────────────────\n🪙 <b>Mint:</b> <code>{}</code>\n💸 <b>Amount:</b> {:.4} SOL\n🔗 <a href=\"https://solscan.io/tx/{}\">View on Solscan</a>",
            action, mint_str, amount_sol, tx_id_str
        );

        let url = format!("https://api.telegram.org/bot{}/sendMessage", token_token);
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": msg,
            "parse_mode": "HTML",
            "disable_web_page_preview": true
        });

        let client = reqwest::Client::new();
        let _ = client.post(&url).json(&payload).send().await;
    });
}
