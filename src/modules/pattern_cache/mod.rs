use crate::*;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

// ── Raw JSON payload from the statistics agent ──

#[derive(Debug, Clone, Deserialize)]
struct TokenFilterRaw {
    mint_pattern: String,
    buy_pattern: Vec<String>,
    tp_threshold: Vec<f64>,
    net_profit: f64,
    token_count: u64,
    avg_profit: f64,
    sell_amounts: Vec<f64>,
    win_rate_low: bool,
    win_count: u64,
    loss_count: u64,
    win_rate: f64,
}

// ── Parsed types ──

pub type MintPattern = (u32, u64);
pub type BuyPattern = ((u32, u64), u8);

#[derive(Debug, Clone, Serialize)]
pub struct TokenFilter {
    pub mint_pattern: MintPattern,
    pub buy_pattern: Vec<BuyPattern>,
    pub tp_threshold: Vec<f64>,
    pub net_profit: f64,
    pub token_count: u64,
    pub avg_profit: f64,
    pub sell_amounts: Vec<f64>,
    pub win_rate_low: bool,
    pub win_count: u64,
    pub loss_count: u64,
    pub win_rate: f64,
}

impl TokenFilter {
    #[inline]
    pub fn primary_tp_threshold(&self) -> f64 {
        // Validated during payload parsing to be non-empty.
        self.tp_threshold[0]
    }
}

// ── Parsing ──

fn parse_mint_pattern(s: &str) -> Result<MintPattern, String> {
    let inner = s
        .trim()
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| format!("invalid mint_pattern format: {s}"))?;
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 2 {
        return Err(format!(
            "mint_pattern expects 2 values, got {}: {s}",
            parts.len()
        ));
    }
    let a = parts[0]
        .trim()
        .parse::<u32>()
        .map_err(|e| format!("mint_pattern.0: {e}"))?;
    let b = parts[1]
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("mint_pattern.1: {e}"))?;
    Ok((a, b))
}

fn parse_buy_pattern(s: &str) -> Result<BuyPattern, String> {
    let inner = s
        .trim()
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| format!("invalid buy_pattern format: {s}"))?;

    let open = inner
        .find('(')
        .ok_or_else(|| format!("missing inner tuple in buy_pattern: {s}"))?;
    let close = inner
        .find(')')
        .ok_or_else(|| format!("missing closing paren in buy_pattern: {s}"))?;

    let pair_str = &inner[open + 1..close];
    let parts: Vec<&str> = pair_str.split(',').collect();
    if parts.len() != 2 {
        return Err(format!("buy_pattern inner tuple expects 2 values: {s}"));
    }
    let a = parts[0]
        .trim()
        .parse::<u32>()
        .map_err(|e| format!("buy_pattern.0.0: {e}"))?;
    let b = parts[1]
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("buy_pattern.0.1: {e}"))?;

    let tail = inner[close + 1..].trim().trim_start_matches(',').trim();
    let c = tail
        .parse::<u8>()
        .map_err(|e| format!("buy_pattern.1: {e}"))?;

    Ok(((a, b), c))
}

impl TryFrom<TokenFilterRaw> for TokenFilter {
    type Error = String;

    fn try_from(raw: TokenFilterRaw) -> Result<Self, Self::Error> {
        if raw.tp_threshold.is_empty() {
            return Err("tp_threshold must contain at least one value".to_string());
        }
        if raw.sell_amounts.is_empty() {
            return Err("sell_amounts must contain at least one value".to_string());
        }
        if raw.tp_threshold.len() != raw.sell_amounts.len() {
            return Err(format!(
                "tp_threshold and sell_amounts must have same length (got {} vs {})",
                raw.tp_threshold.len(),
                raw.sell_amounts.len()
            ));
        }

        let mint_pattern = parse_mint_pattern(&raw.mint_pattern)?;
        let buy_pattern = raw
            .buy_pattern
            .iter()
            .map(|s| parse_buy_pattern(s))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TokenFilter {
            mint_pattern,
            buy_pattern,
            tp_threshold: raw.tp_threshold,
            net_profit: raw.net_profit,
            token_count: raw.token_count,
            avg_profit: raw.avg_profit,
            sell_amounts: raw.sell_amounts,
            win_rate_low: raw.win_rate_low,
            win_count: raw.win_count,
            loss_count: raw.loss_count,
            win_rate: raw.win_rate,
        })
    }
}

// ── In-memory cache ──
// Arc<Vec> inside RwLock: readers clone the Arc (atomic increment), not the Vec.

pub type PatternCache = Arc<RwLock<Arc<Vec<TokenFilter>>>>;

pub static PATTERN_CACHE: Lazy<PatternCache> =
    Lazy::new(|| Arc::new(RwLock::new(Arc::new(Vec::new()))));

#[inline]
pub fn get_cached_patterns() -> Arc<Vec<TokenFilter>> {
    PATTERN_CACHE.read().unwrap().clone()
}

// ── Axum handlers ──

#[derive(Debug, Deserialize)]
struct PatternPostPayload {
    results: Vec<TokenFilterRaw>,
    total_profit: f64,
    total_token_count: u64,
    total_win_count: u64,
    total_loss_count: u64,
    total_win_rate: f64,
    lookback: u64,
    fetch_interval: u64,
    timestamp: u64,
}

async fn post_patterns(
    State(cache): State<PatternCache>,
    Json(payload): Json<PatternPostPayload>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut parsed = Vec::with_capacity(payload.results.len());

    for (i, raw) in payload.results.into_iter().enumerate() {
        match TokenFilter::try_from(raw) {
            Ok(f) => parsed.push(f),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "status": "error",
                        "message": format!("filter[{i}]: {e}"),
                    })),
                );
            }
        }
    }

    let count = parsed.len();
    {
        let mut store = cache.write().unwrap();
        *store = Arc::new(parsed);
    }

    println!(
        "📦 Received {} pattern(s) | profit: {:.2} | tokens: {} | W/L: {}/{} | win_rate: {:.1}% | lookback: {}s | interval: {}s | ts: {}",
        count, payload.total_profit, payload.total_token_count,
        payload.total_win_count, payload.total_loss_count, payload.total_win_rate,
        payload.lookback, payload.fetch_interval, payload.timestamp,
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "patterns_saved": count,
        })),
    )
}

async fn health() -> &'static str {
    "ok"
}

// ── Server bootstrap ──

pub async fn run_pattern_server(port: u16) {
    let app = Router::new()
        .route("/patterns", post(post_patterns))
        .route("/health", get(health))
        .with_state(PATTERN_CACHE.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 Pattern API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind pattern server port");

    axum::serve(listener, app)
        .await
        .expect("Pattern API server crashed");
}

pub static MANUAL_MINT_PRICE_PATTERNS: Lazy<Vec<u64>> = Lazy::new(|| vec![666666]);

pub fn check_manul_entry_signal(token_data: TokenDatabaseSchema) -> bool {
    let result = if let Some(dev_buy_lamports) = token_data.dev_buy_sol_lamports
        && MANUAL_MINT_PRICE_PATTERNS.contains(&token_data.mint_budget_compute_unit_price)
        && dev_buy_lamports % 1000000 < 6000
        && dev_buy_lamports % 1000000 > 0
    {
        info!(
            "💥 Manual entry signal triggered for token: {}",
            token_data.token_mint
        );
        true
    } else {
        false
    };
    result
}
