use crate::*;
use colored::*;
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    pubkey::Pubkey,
    system_instruction,
};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;
use tokio::sync::OnceCell;
use tokio::time::{Duration, sleep};

use super::nonce_file::{load_nonce_pubkeys, parse_nonce_hash_from_data};

const STATE_READY: u8 = 0;
const STATE_IN_USE: u8 = 1;
const STATE_REFRESHING: u8 = 2;

struct NonceEntry {
    pubkey: Pubkey,
    nonce_hash: Mutex<Hash>,
    state: AtomicU8,
}

pub struct NoncePool {
    entries: Vec<NonceEntry>,
}

static NONCE_POOL: OnceCell<NoncePool> = OnceCell::const_new();

pub struct AcquiredNonce {
    pub index: usize,
    pub nonce_pubkey: Pubkey,
    pub nonce_hash: Hash,
    pub advance_ix: Instruction,
}

async fn fetch_nonce_hash(pubkey: &Pubkey) -> Option<Hash> {
    let account = RPC_CLIENT.get_account(pubkey).await.ok()?;
    parse_nonce_hash_from_data(&account.data)
}

impl NoncePool {
    /// Acquire a ready nonce from the pool. Instant — no RPC calls.
    fn acquire(&self) -> Option<AcquiredNonce> {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry
                .state
                .compare_exchange(STATE_READY, STATE_IN_USE, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                let hash = *entry.nonce_hash.lock().unwrap();
                if hash == Hash::default() {
                    entry.state.store(STATE_READY, Ordering::SeqCst);
                    continue;
                }
                let advance_ix =
                    system_instruction::advance_nonce_account(&entry.pubkey, &*SIGNER_PUBKEY);
                return Some(AcquiredNonce {
                    index: i,
                    nonce_pubkey: entry.pubkey,
                    nonce_hash: hash,
                    advance_ix,
                });
            }
        }
        None
    }

    /// Spawn a background task to refresh the nonce hash, then mark as READY.
    /// Returns immediately — does NOT block the caller.
    ///
    /// Immediately invalidates the cached hash to prevent reuse of the consumed
    /// nonce, waits for the tx to land on-chain, then retries RPC fetch until a
    /// *new* hash is observed.
    fn spawn_refresh(&self, index: usize) {
        let consumed_hash = if let Some(entry) = self.entries.get(index) {
            let consumed = *entry.nonce_hash.lock().unwrap();
            *entry.nonce_hash.lock().unwrap() = Hash::default();
            entry.state.store(STATE_REFRESHING, Ordering::SeqCst);
            consumed
        } else {
            return;
        };

        tokio::spawn(async move {
            sleep(Duration::from_secs(3)).await;

            if let Some(pool) = NONCE_POOL.get() {
                if let Some(entry) = pool.entries.get(index) {
                    let mut refreshed = false;
                    for attempt in 0..5u64 {
                        if attempt > 0 {
                            sleep(Duration::from_millis(1500)).await;
                        }
                        if let Some(hash) = fetch_nonce_hash(&entry.pubkey).await {
                            if hash != consumed_hash {
                                *entry.nonce_hash.lock().unwrap() = hash;
                                refreshed = true;
                                break;
                            }
                        }
                    }
                    if !refreshed {
                        // Hash stays as Hash::default() — acquire() will skip this
                        // slot. The background recovery loop will eventually fix it.
                    }
                    entry.state.store(STATE_READY, Ordering::SeqCst);
                }
            }
        });
    }

    /// Release without refreshing (tx was never sent).
    fn release(&self, index: usize) {
        if let Some(entry) = self.entries.get(index) {
            entry.state.store(STATE_READY, Ordering::SeqCst);
        }
    }
}

// ─── Public API ──────────────────────────────────────────────────────

/// Initialize the nonce pool from saved nonce accounts.
/// Fetches current nonce values from RPC. Call once at bot startup.
pub async fn init_nonce_pool() {
    let pubkeys = load_nonce_pubkeys();
    if pubkeys.is_empty() {
        println!(
            "{}",
            "  ⚠ No nonce accounts found. Use CLI option 2 to create them first.".yellow()
        );
        let pool = NoncePool {
            entries: Vec::new(),
        };
        let _ = NONCE_POOL.set(pool);
        return;
    }

    println!(
        "{} Loading {} nonce accounts...",
        "⏳".yellow(),
        pubkeys.len()
    );

    let mut entries = Vec::with_capacity(pubkeys.len());
    for pk in &pubkeys {
        let hash = fetch_nonce_hash(pk).await.unwrap_or_default();
        if hash == Hash::default() {
            println!("  {} Nonce account {} - failed to fetch", "⚠".red(), pk);
        }
        entries.push(NonceEntry {
            pubkey: *pk,
            nonce_hash: Mutex::new(hash),
            state: AtomicU8::new(STATE_READY),
        });
    }

    let valid = entries
        .iter()
        .filter(|e| *e.nonce_hash.lock().unwrap() != Hash::default())
        .count();
    println!(
        "{} Nonce pool ready: {}/{} accounts loaded",
        "✅".green(),
        valid,
        entries.len()
    );

    let pool = NoncePool { entries };
    let _ = NONCE_POOL.set(pool);

    tokio::spawn(async {
        nonce_recovery_loop().await;
    });
}

/// Background recovery loop. Durable nonces never expire on their own — they
/// only change when consumed by an AdvanceNonceAccount instruction. So we only
/// need to recover slots where spawn_refresh failed (hash stuck at default).
async fn nonce_recovery_loop() {
    loop {
        sleep(Duration::from_secs(30)).await;
        let pool = match NONCE_POOL.get() {
            Some(p) => p,
            None => continue,
        };
        for entry in &pool.entries {
            if entry.state.load(Ordering::SeqCst) != STATE_READY {
                continue;
            }
            let needs_recovery = *entry.nonce_hash.lock().unwrap() == Hash::default();
            if !needs_recovery {
                continue;
            }
            if let Some(hash) = fetch_nonce_hash(&entry.pubkey).await {
                if entry.state.load(Ordering::SeqCst) == STATE_READY {
                    *entry.nonce_hash.lock().unwrap() = hash;
                }
            }
        }
    }
}

/// Acquire a nonce from the pool for transaction use.
/// Instant — reads from pre-cached hash, no RPC call.
pub fn acquire_nonce() -> Option<AcquiredNonce> {
    NONCE_POOL.get()?.acquire()
}

/// Kick off a background refresh for this nonce slot after tx submission.
/// Returns immediately — does NOT block the caller.
pub fn spawn_nonce_refresh(index: usize) {
    if let Some(pool) = NONCE_POOL.get() {
        pool.spawn_refresh(index);
    }
}

/// Release a nonce without refreshing (e.g., if tx was never sent).
pub fn release_nonce(index: usize) {
    if let Some(pool) = NONCE_POOL.get() {
        pool.release(index);
    }
}
