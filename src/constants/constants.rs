pub static PUMP_FUN_TOKEN_TOTAL_SUPPLY: u64 = 1000000000;
pub static BONDING_CURVE_TOKEN_INITIAL_BALANCE: u64 = 1003188873212;
pub static MAX_BUNDLE_BUY_LEN: usize = 4;

///////////// Relayer cross-chain swap constants
/// 
/// 
pub const SOLANA_CHAIN_ID: u64 = 792703809;
pub const BNB_CHAIN_ID: u64 = 56;
pub const SOL_NATIVE_CURRENCY: &str = "11111111111111111111111111111111";
pub const BNB_NATIVE_CURRENCY: &str = "0x0000000000000000000000000000000000000000";
pub const RELAY_QUOTE_URL: &str = "https://api.relay.link/quote/v2";
pub const RELAY_STATUS_URL: &str = "https://api.relay.link/intents/status/v3";
pub const BRIDGE_POLL_INTERVAL_MS: u64 = 3000;
pub const BRIDGE_TIMEOUT_MS: u64 = 180_000;

/// Minimum SOL balance required to rotate (0.01 SOL)
pub const MIN_SOL_LAMPORTS: u64 = 10_000_000;
/// SOL reserved for Solana tx fee (0.00005 SOL — 10x base fee of 5000 lamports/sig)
pub const FEE_RESERVE_LAMPORTS: u64 = 50_000;
/// BNB reserved for gas when bridging back to SOL (~1.2x actual gas cost from live run)
pub const BNB_GAS_RESERVE_WEI: u128 = 2_000_000_000_000;
