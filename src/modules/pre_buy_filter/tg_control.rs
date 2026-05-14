use crate::*;
use serde_json::{json, Value};
use std::env;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::time::{sleep, Duration};

pub static WARN_ONLY_MODE: AtomicBool = AtomicBool::new(false);
pub static ENABLE_M1_HOLDER: AtomicBool = AtomicBool::new(true);
pub static ENABLE_M2_PANIC: AtomicBool = AtomicBool::new(true);
pub static ENABLE_M3_DEV: AtomicBool = AtomicBool::new(true);
pub static ENABLE_M4_GENESIS: AtomicBool = AtomicBool::new(true);
pub static ENABLE_M5_METADATA: AtomicBool = AtomicBool::new(true);

// Global State
pub static BOT_IS_RUNNING: AtomicBool = AtomicBool::new(false);

// Global Stats (Mocked for now, wired up later)
pub static STAT_SCANNED: AtomicU64 = AtomicU64::new(0);
pub static STAT_PASSED: AtomicU64 = AtomicU64::new(0);
pub static STAT_REJECTED: AtomicU64 = AtomicU64::new(0);
pub static STAT_WARNED: AtomicU64 = AtomicU64::new(0);
pub static STAT_SKIPPED: AtomicU64 = AtomicU64::new(0);

pub async fn start_telegram_control_bot() {
    let token = match env::var("TG_BOT_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => return,
    };
    let chat_id = match env::var("TG_CHAT_ID") {
        Ok(c) if !c.is_empty() => c,
        _ => return,
    };

    let client = reqwest::Client::new();
    let mut offset: i64 = 0;

    info!("🚀 Telegram Control Bot Task Started...");

    loop {
        let url = format!("https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=30", token, offset);
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(results) = json["result"].as_array() {
                    for result in results {
                        if let Some(update_id) = result["update_id"].as_i64() {
                            offset = update_id + 1;
                        }

                        // Handle message
                        if let Some(message) = result.get("message") {
                            let text = message["text"].as_str().unwrap_or("");
                            let sender_chat_id = message["chat"]["id"].as_i64().unwrap_or(0).to_string();
                            
                            info!("📥 [TG_CONTROL] Received msg: '{}' from chat_id: {}", text, sender_chat_id);
                            
                            if sender_chat_id == chat_id {
                                handle_text_message(&client, &token, &chat_id, text).await;
                            } else {
                                info!("⚠️ [TG_CONTROL] Ignored msg from unauthorized chat_id: {} (expected: {})", sender_chat_id, chat_id);
                            }
                        }

                        // Handle callback query
                        if let Some(callback) = result.get("callback_query") {
                            let data = callback["data"].as_str().unwrap_or("");
                            let callback_id = callback["id"].as_str().unwrap_or("");
                            let sender_chat_id = callback["message"]["chat"]["id"].as_i64().unwrap_or(0).to_string();
                            let message_id = callback["message"]["message_id"].as_i64().unwrap_or(0);

                            if sender_chat_id == chat_id {
                                handle_callback(&client, &token, &chat_id, message_id, data).await;
                                let answer_url = format!("https://api.telegram.org/bot{}/answerCallbackQuery", token);
                                let _ = client.post(&answer_url).json(&json!({"callback_query_id": callback_id})).send().await;
                            }
                        }
                    }
                } else if let Some(desc) = json.get("description") {
                    error!("❌ [TG_CONTROL] API Error: {}", desc);
                }
            } else {
                error!("❌ [TG_CONTROL] Failed to parse JSON response");
            }
        } else {
            error!("❌ [TG_CONTROL] HTTP request failed");
        }
        sleep(Duration::from_secs(1)).await;
    }
}

async fn handle_text_message(client: &reqwest::Client, token: &str, chat_id: &str, text: &str) {
    if text.starts_with("/start") || text.contains("Dashboard") {
        send_dashboard(client, token, chat_id, "Today").await;
    } else if text.contains("Wallet management") {
        let balance = crate::RPC_CLIENT.get_balance(&*crate::SIGNER_PUBKEY).await.unwrap_or(0);
        let sol_balance = balance as f64 / 1_000_000_000.0;
        let msg = format!("💰 **Wallet Management**\n──────────────────\n🔑 **Address:** `{}`\n💵 **Balance:** {:.4} SOL\n\n_Note: To change wallets, update the `private_key` in your .env file and restart the bot._", *crate::SIGNER_PUBKEY, sol_balance);
        send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "Markdown").await;
    } else if text.contains("Trading parameters") {
        let msg = format!("⚙️ **Trading Parameters**\n──────────────────\n💸 **Base Buy Amount:** {} SOL\n🛑 **Stop Loss:** {:.0}%\n📈 **Dynamic Sizing:** {}\n🛡️ **Max Risk Allowed:** {}\n\n_Note: To modify parameters, please edit `Config.toml`._",
            *crate::BUY_AMOUNT_SOL,
            *crate::STOP_LOSS * 100.0,
            if *crate::ENABLE_DYNAMIC_SIZING { "✅ ON" } else { "❌ OFF" },
            *crate::MAX_TOTAL_RISK_SCORE
        );
        send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "Markdown").await;
    } else if text.contains("Anti-Rug") {
        send_settings_menu(client, token, chat_id).await;
    } else if text.contains("Start") {
        BOT_IS_RUNNING.store(true, Ordering::Relaxed);
        send_simple_msg_with_keyboard(client, token, chat_id, "✅ Bot is STARTED. Ready to snipe!").await;
    } else if text.contains("Stop") || text.starts_with("/stop") {
        BOT_IS_RUNNING.store(false, Ordering::Relaxed);
        send_simple_msg_with_keyboard(client, token, chat_id, "🛑 Bot is STOPPED. Will not buy any new tokens.").await;
    }
}

async fn send_simple_msg(client: &reqwest::Client, token: &str, chat_id: &str, msg: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let payload = json!({ "chat_id": chat_id, "text": msg });
    let _ = client.post(&url).json(&payload).send().await;
}

async fn send_simple_msg_with_parse_mode(client: &reqwest::Client, token: &str, chat_id: &str, msg: &str, parse_mode: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": parse_mode });
    let _ = client.post(&url).json(&payload).send().await;
}

async fn send_simple_msg_with_keyboard(client: &reqwest::Client, token: &str, chat_id: &str, msg: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let payload = json!({
        "chat_id": chat_id,
        "text": msg,
        "reply_markup": build_reply_keyboard()
    });
    let _ = client.post(&url).json(&payload).send().await;
}

fn build_reply_keyboard() -> Value {
    let run_btn = if BOT_IS_RUNNING.load(Ordering::Relaxed) { "⏹️ Stop" } else { "▶️ Start" };
    json!({
        "keyboard": [
            [{"text": "💰 Wallet management"}, {"text": "⚙️ Trading parameters"}],
            [{"text": "🛡️ Anti-Rug"}, {"text": run_btn}]
        ],
        "resize_keyboard": true,
        "is_persistent": true
    })
}

fn build_dashboard_text(period: &str) -> String {
    let scanned = STAT_SCANNED.load(Ordering::Relaxed);
    let passed = STAT_PASSED.load(Ordering::Relaxed);
    let rejected = STAT_REJECTED.load(Ordering::Relaxed);
    let warned = STAT_WARNED.load(Ordering::Relaxed);
    let skipped = STAT_SKIPPED.load(Ordering::Relaxed);
    let pass_rate = if scanned > 0 { (passed as f64 / scanned as f64) * 100.0 } else { 0.0 };

    format!("📊 {}\n\
    ──────────────────\n\
    💰 *PNL Summary*\n\
    ├ Realized PNL: +0.0000 SOL\n\
    ├ Total spent: 0.0000 SOL\n\
    ├ Total received: 0.0000 SOL\n\
    ├ Win rate: 0.0% (0/0)\n\
    ├ ✅ Wins: 0\n\
    └ ❌ Losses: 0\n\n\
    💹 *Trade Activity*\n\
    ├ Total buys: 0 (✅ 0 / ❌ 0)\n\
    ├ Buy success rate: 0.0%\n\
    └ Total sells: 0\n\n\
    🛡️ *Anti-Rug Filter*\n\
    ├ Scanned: {}\n\
    ├ ✅ Passed: {}\n\
    ├ ❌ Rejected: {}\n\
    ├ ⚠️ Warned: {}\n\
    ├ 🚫 Skipped: {}\n\
    └ Pass rate: {:.1}%\n", period, scanned, passed, rejected, warned, skipped, pass_rate)
}

fn build_dashboard_inline_keyboard() -> Value {
    json!({
        "inline_keyboard": [
            [{"text": "📊 Select time period for stats:", "callback_data": "ignore"}],
            [{"text": "📅 Today", "callback_data": "time_today"}, {"text": "📈 7 Days", "callback_data": "time_7d"}],
            [{"text": "📅 30 Days", "callback_data": "time_30d"}, {"text": "🌐 All Time", "callback_data": "time_all"}]
        ]
    })
}

async fn send_dashboard(client: &reqwest::Client, token: &str, chat_id: &str, period: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let text = build_dashboard_text(period);
    
    let payload = json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "Markdown",
        "reply_markup": build_dashboard_inline_keyboard()
    });
    
    // Send dashboard and simultaneously send the reply keyboard
    let _ = client.post(&url).json(&payload).send().await;
    
    // Ensure reply keyboard is attached
    let payload2 = json!({
        "chat_id": chat_id,
        "text": "Options loaded below 👇",
        "reply_markup": build_reply_keyboard()
    });
    let _ = client.post(&url).json(&payload2).send().await;
}

async fn update_dashboard(client: &reqwest::Client, token: &str, chat_id: &str, message_id: i64, period: &str) {
    let url = format!("https://api.telegram.org/bot{}/editMessageText", token);
    let text = build_dashboard_text(period);
    
    let payload = json!({
        "chat_id": chat_id,
        "message_id": message_id,
        "text": text,
        "parse_mode": "Markdown",
        "reply_markup": build_dashboard_inline_keyboard()
    });
    let _ = client.post(&url).json(&payload).send().await;
}

// ── Anti-Rug Settings Logic ──

async fn handle_callback(client: &reqwest::Client, token: &str, chat_id: &str, message_id: i64, data: &str) {
    match data {
        "time_today" => update_dashboard(client, token, chat_id, message_id, "Today").await,
        "time_7d" => update_dashboard(client, token, chat_id, message_id, "7 Days").await,
        "time_30d" => update_dashboard(client, token, chat_id, message_id, "30 Days").await,
        "time_all" => update_dashboard(client, token, chat_id, message_id, "All Time").await,
        "toggle_warn" => {
            let current = WARN_ONLY_MODE.load(Ordering::Relaxed);
            WARN_ONLY_MODE.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        "toggle_m1" => {
            let current = ENABLE_M1_HOLDER.load(Ordering::Relaxed);
            ENABLE_M1_HOLDER.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        "toggle_m2" => {
            let current = ENABLE_M2_PANIC.load(Ordering::Relaxed);
            ENABLE_M2_PANIC.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        "toggle_m3" => {
            let current = ENABLE_M3_DEV.load(Ordering::Relaxed);
            ENABLE_M3_DEV.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        "toggle_m4" => {
            let current = ENABLE_M4_GENESIS.load(Ordering::Relaxed);
            ENABLE_M4_GENESIS.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        "toggle_m5" => {
            let current = ENABLE_M5_METADATA.load(Ordering::Relaxed);
            ENABLE_M5_METADATA.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        _ => {}
    }
}

fn build_settings_keyboard() -> Value {
    let w = if WARN_ONLY_MODE.load(Ordering::Relaxed) { "⚠️ Warn-Only Mode (no block)" } else { "❌ Warn-Only Mode (no block)" };
    let m1 = if ENABLE_M1_HOLDER.load(Ordering::Relaxed) { "✅ M1: Holder Analyzer" } else { "❌ M1: Holder Analyzer" };
    let m2 = if ENABLE_M2_PANIC.load(Ordering::Relaxed) { "✅ M2: Panic-Sell Monitor" } else { "❌ M2: Panic-Sell Monitor" };
    let m3 = if ENABLE_M3_DEV.load(Ordering::Relaxed) { "✅ M3: Dev Wallet Profiler" } else { "❌ M3: Dev Wallet Profiler" };
    let m4 = if ENABLE_M4_GENESIS.load(Ordering::Relaxed) { "✅ M4: Genesis Detector" } else { "❌ M4: Genesis Detector" };
    let m5 = if ENABLE_M5_METADATA.load(Ordering::Relaxed) { "✅ M5: Metadata Checker" } else { "❌ M5: Metadata Checker" };

    json!({
        "inline_keyboard": [
            [{"text": "🛡️ Anti-Rug Intelligence Settings", "callback_data": "ignore"}],
            [{"text": "Tap to toggle each module:", "callback_data": "ignore"}],
            [{"text": "🔄 Anti-Rug Master Switch", "callback_data": "ignore"}],
            [{"text": w, "callback_data": "toggle_warn"}],
            [{"text": m1, "callback_data": "toggle_m1"}],
            [{"text": m2, "callback_data": "toggle_m2"}],
            [{"text": m3, "callback_data": "toggle_m3"}],
            [{"text": m4, "callback_data": "toggle_m4"}],
            [{"text": m5, "callback_data": "toggle_m5"}],
            [{"text": "🔙 Close", "callback_data": "ignore"}]
        ]
    })
}

async fn send_settings_menu(client: &reqwest::Client, token: &str, chat_id: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let payload = json!({
        "chat_id": chat_id,
        "text": "⚙️ **Anti-Rug Intelligence Settings**",
        "parse_mode": "Markdown",
        "reply_markup": build_settings_keyboard()
    });
    let _ = client.post(&url).json(&payload).send().await;
}

async fn update_settings_menu(client: &reqwest::Client, token: &str, chat_id: &str, message_id: i64) {
    let url = format!("https://api.telegram.org/bot{}/editMessageReplyMarkup", token);
    let payload = json!({
        "chat_id": chat_id,
        "message_id": message_id,
        "reply_markup": build_settings_keyboard()
    });
    let _ = client.post(&url).json(&payload).send().await;
}
