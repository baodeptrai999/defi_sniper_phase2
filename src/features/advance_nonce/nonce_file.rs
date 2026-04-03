use serde::{Deserialize, Serialize};
use solana_sdk::{hash::Hash, pubkey::Pubkey};
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub const NONCE_ACCOUNTS_PATH: &str = "nonce_accounts.json";
pub const NONCE_RENT_LAMPORTS: u64 = 1_447_680;

#[derive(Serialize, Deserialize)]
struct NonceAccountsFile {
    accounts: Vec<String>,
}

pub fn load_nonce_pubkeys() -> Vec<Pubkey> {
    let path = Path::new(NONCE_ACCOUNTS_PATH);
    if !path.exists() {
        return Vec::new();
    }
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let file: NonceAccountsFile = match serde_json::from_str(&content) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    file.accounts
        .iter()
        .filter_map(|s| Pubkey::from_str(s).ok())
        .collect()
}

pub fn save_nonce_pubkeys(pubkeys: &[Pubkey]) {
    let file = NonceAccountsFile {
        accounts: pubkeys.iter().map(|p| p.to_string()).collect(),
    };
    let json = serde_json::to_string_pretty(&file).expect("Failed to serialize nonce accounts");
    fs::write(NONCE_ACCOUNTS_PATH, json).expect("Failed to write nonce_accounts.json");
}

/// Parse nonce hash from raw nonce account data.
///
/// Layout (bincode):
///   [0..4]   u32  Versions enum variant (1 = Current)
///   [4..8]   u32  State enum variant (1 = Initialized)
///   [8..40]  [u8;32] Authority pubkey
///   [40..72] [u8;32] Durable nonce hash
///   [72..80] u64  lamports_per_signature
pub fn parse_nonce_hash_from_data(data: &[u8]) -> Option<Hash> {
    if data.len() < 80 {
        return None;
    }
    let state = u32::from_le_bytes(data[4..8].try_into().ok()?);
    if state != 1 {
        return None;
    }
    let hash_bytes: [u8; 32] = data[40..72].try_into().ok()?;
    Some(Hash::new_from_array(hash_bytes))
}
