use crate::*;
use solana_sdk::pubkey::Pubkey;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ── Simulated token state ──

#[derive(Debug, Clone)]
pub struct SimToken {
    pub mint: Pubkey,
    pub pattern_label: String,
    pub matched_at: Instant,
    pub buy_confirmed: bool,

    // Prices
    pub mint_price: f64,
    pub buy_price: f64,
    pub current_price: f64,
    pub max_price: f64,
    pub exit_price: f64,

    // TP / SL
    pub tp_levels: Vec<f64>,
    pub sell_amounts: Vec<f64>,
    pub next_tp_index: usize,
    pub sl_triggered: bool,
    pub tp_triggered_at: Vec<f64>, // prices where each TP hit

    // Outcome
    pub outcome: SimOutcome,
    pub total_sold_pct: f64,
    pub pnl_pct: f64, // final P&L %

    // Bundle tracking (server patterns)
    pub mint_cu: (u32, u64),
    pub buy_tx_history: Vec<((u32, u64), u8)>,

    // Metadata
    pub tx_count: u64,
    pub creator: Pubkey,
    pub is_migrated: bool,
    pub exit_reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimOutcome {
    Pending,    // still in position
    TpHit,      // all TP levels hit
    SlHit,      // stop loss triggered
    Timeout,    // no activity, position expired
    Partial,    // some TPs hit, then SL or timeout
}

impl std::fmt::Display for SimOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimOutcome::Pending => write!(f, "PENDING"),
            SimOutcome::TpHit => write!(f, "TP_HIT"),
            SimOutcome::SlHit => write!(f, "SL_HIT"),
            SimOutcome::Timeout => write!(f, "HOLDING"),
            SimOutcome::Partial => write!(f, "PARTIAL"),
        }
    }
}

// ── Simulation engine ──

pub struct SimEngine {
    pub tokens: Arc<Mutex<HashMap<Pubkey, SimToken>>>,
    pub completed: Arc<Mutex<Vec<SimToken>>>,
    pub start_time: Instant,
    pub buy_amount_sol: f64,
    pub stop_loss_pct: f64,
    pub real_tp_multiply: f64,
    pub confirmation_delay: Duration,
    pub match_count_per_pattern: Arc<Mutex<HashMap<String, u64>>>,
    pub total_tx_processed: Arc<Mutex<u64>>,
}

impl SimEngine {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
            completed: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
            buy_amount_sol: CONFIG.simulation_setting.buy_amount_sol,
            stop_loss_pct: CONFIG.simulation_setting.stop_loss / 100.0,
            real_tp_multiply: CONFIG.simulation_setting.real_tp_multiply / 100.0,
            confirmation_delay: Duration::from_millis(CONFIG.simulation_setting.confirmation_delay_ms),
            match_count_per_pattern: Arc::new(Mutex::new(HashMap::new())),
            total_tx_processed: Arc::new(Mutex::new(0)),
        }
    }

    /// Process one transaction's parsed data (simulation equivalent of handle_trade_events)
    pub fn process_transaction(
        &self,
        budget_compute_data: (u32, u64),
        pumpfun_trade_data: &(
            Vec<MintContext>,
            Vec<PumpfunBuyEvent>,
            Vec<PumpfunSellEvent>,
            Vec<MintInstructionAccounts>,
            Vec<PumpfunBuyInstructionAccounts>,
            Vec<PumpfunSellInstructionAccounts>,
        ),
        migration_data: &(
            Vec<MigrateInstructionAccounts>,
            Vec<CreatePoolInstructionAccounts>,
            Vec<CreatePoolEventData>,
        ),
        pumpswap_trade_data: &(
            Vec<PumpswapBuyEvent>,
            Vec<PumpswapSellEvent>,
            Vec<PumpswapBuyInstructionAccounts>,
            Vec<PumpswapSellInstructionAccounts>,
        ),
        _tx_id: &str,
    ) {
        {
            let mut count = self.total_tx_processed.lock().expect("tx counter lock");
            *count += 1;
        }

        let (unit, price) = budget_compute_data;
        let (mint_contexts, buy_events, sell_events, _, _, _) = pumpfun_trade_data;
        let (_, create_pool_accounts, create_pool_events) = migration_data;
        let (pumpswap_buy_events, pumpswap_sell_events, pumpswap_buy_accs, pumpswap_sell_accs) =
            pumpswap_trade_data;

        let manual_patterns = get_manual_patterns();
        let server_patterns = get_cached_patterns();

        // ── Mint matching ──
        let mut minted_in_this_tx: HashSet<Pubkey> = HashSet::new();

        for mint_ctx in mint_contexts.iter() {
            let mint_event = &mint_ctx.mint_event;
            let mint_tx_ctx = &mint_ctx.mint_transaction_context;
            let mint = mint_event.mint;

            let initial_price = (mint_event.virtual_sol_reserves as f64 / 1e9)
                / (mint_event.virtual_token_reserves as f64 / 1e6);

            // Check server patterns
            let server_matched = server_patterns.iter().any(|p| p.mint_pattern == (unit, price));

            // Check manual patterns
            let matched_manual = manual_patterns
                .iter()
                .find(|p| p.matches(unit, price, mint_tx_ctx));

            if let Some(manual_pat) = matched_manual {
                let sim_token = SimToken {
                    mint,
                    pattern_label: manual_pat.label.clone(),
                    matched_at: Instant::now(),
                    buy_confirmed: false,
                    mint_price: initial_price,
                    buy_price: initial_price,
                    current_price: initial_price,
                    max_price: initial_price,
                    exit_price: 0.0,
                    tp_levels: manual_pat.take_profit.clone(),
                    sell_amounts: manual_pat.sell_amounts.clone(),
                    next_tp_index: 0,
                    sl_triggered: false,
                    tp_triggered_at: Vec::new(),
                    outcome: SimOutcome::Pending,
                    total_sold_pct: 0.0,
                    pnl_pct: 0.0,
                    mint_cu: (unit, price),
                    buy_tx_history: Vec::new(),
                    tx_count: 0,
                    creator: mint_event.creator,
                    is_migrated: false,
                    exit_reason: String::new(),
                };

                let mut tokens = self.tokens.lock().expect("tokens lock");
                tokens.insert(mint, sim_token);
                minted_in_this_tx.insert(mint);

                let mut counts = self.match_count_per_pattern.lock().expect("pattern count lock");
                *counts.entry(manual_pat.label.clone()).or_insert(0) += 1;

                let mc = initial_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                info!(
                    "\n📌 [SIM] [MANUAL_MATCH]\n\
                     │  Pattern:    {}\n\
                     │  Mint:       {}\n\
                     │  CU:         ({}, {})\n\
                     │  MC:         {:.2} SOL\n\
                     │  TP Levels:  {:?}%\n\
                     │  Sell Amts:  {:?}%\n\
                     └──────────────────────",
                    manual_pat.label, mint, unit, price,
                    mc,
                    manual_pat.take_profit, manual_pat.sell_amounts,
                );
            } else if server_matched {
                // Server pattern — track but no immediate entry (needs bundle match)
                let sim_token = SimToken {
                    mint,
                    pattern_label: format!("SERVER_CU({},{})", unit, price),
                    matched_at: Instant::now(),
                    buy_confirmed: false,
                    mint_price: initial_price,
                    buy_price: initial_price,
                    current_price: initial_price,
                    max_price: initial_price,
                    exit_price: 0.0,
                    tp_levels: Vec::new(),
                    sell_amounts: Vec::new(),
                    next_tp_index: 0,
                    sl_triggered: false,
                    tp_triggered_at: Vec::new(),
                    outcome: SimOutcome::Pending,
                    total_sold_pct: 0.0,
                    pnl_pct: 0.0,
                    mint_cu: (unit, price),
                    buy_tx_history: Vec::new(),
                    tx_count: 0,
                    creator: mint_event.creator,
                    is_migrated: false,
                    exit_reason: String::new(),
                };

                let mut tokens = self.tokens.lock().expect("tokens lock");
                tokens.insert(mint, sim_token);
                minted_in_this_tx.insert(mint);

                let mut counts = self.match_count_per_pattern.lock().expect("pattern count lock");
                *counts.entry(format!("SERVER_CU({},{})", unit, price)).or_insert(0) += 1;

                let mc = initial_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                info!(
                    "\n🔍 [SIM] [SERVER_MINT_MATCH]\n\
                     │  Mint:       {}\n\
                     │  CU:         ({}, {})\n\
                     │  MC:         {:.2} SOL\n\
                     │  Status:     Waiting for bundle buy match...\n\
                     └──────────────────────",
                    mint, unit, price, mc,
                );
            }
        }

        // ── Pumpfun Buy events → update price + count buys + simulate confirmation ──
        let mut tokens = self.tokens.lock().expect("tokens lock");
        let mut buy_counts: HashMap<Pubkey, u8> = HashMap::new();

        for buy_event in buy_events.iter() {
            let mint = buy_event.mint;

            if let Some(sim) = tokens.get_mut(&mint) {
                let new_price = (buy_event.virtual_sol_reserves as f64 / 1e9)
                    / (buy_event.virtual_token_reserves as f64 / 1e6);
                sim.current_price = new_price;
                sim.max_price = sim.max_price.max(new_price);
                sim.tx_count += 1;

                // Count buys for bundle matching (mirrors real sniper logic)
                if !minted_in_this_tx.contains(&mint) {
                    if let Some(c) = buy_counts.get_mut(&mint) {
                        *c += 1;
                    } else if sim.buy_tx_history.len() < MAX_BUNDLE_BUY_LEN
                        && !sim.buy_confirmed
                    {
                        buy_counts.insert(mint, 1);
                    }
                }

                // Simulate buy confirmation after delay
                if !sim.buy_confirmed
                    && !sim.tp_levels.is_empty()
                    && sim.matched_at.elapsed() >= self.confirmation_delay
                {
                    sim.buy_confirmed = true;
                    sim.buy_price = new_price;
                    let mc = new_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                    let mint_mc = sim.mint_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                    let price_change = ((new_price / sim.mint_price) - 1.0) * 100.0;
                    info!(
                        "\n💰 [SIM] [BUY]\n\
                         │  Pattern:    {}\n\
                         │  Mint:       {}\n\
                         │  Buy MC:     {:.2} SOL\n\
                         │  Mint MC:    {:.2} SOL\n\
                         │  Δ MC:       {:+.2}%\n\
                         │  Amount:     {:.4} SOL\n\
                         │  SL:         {:.0}%\n\
                         │  TP:         {:?}%\n\
                         └──────────────────────",
                        sim.pattern_label, sim.mint,
                        mc, mint_mc, price_change,
                        self.buy_amount_sol,
                        self.stop_loss_pct * 100.0,
                        sim.tp_levels,
                    );
                }

                // Check TP/SL only after buy confirmed
                if sim.buy_confirmed {
                    self.check_tp_sl(sim);
                }
            }
        }

        // ── Bundle buy pattern matching for server patterns ──
        for (mint, buy_count) in buy_counts.iter() {
            if let Some(sim) = tokens.get_mut(mint) {
                sim.buy_tx_history.push(((unit, price), *buy_count));

                let mint_pat = sim.mint_cu;
                let history = &sim.buy_tx_history;

                let mut matched_pattern: Option<&TokenFilter> = None;
                for pattern in server_patterns.iter() {
                    if pattern.mint_pattern != mint_pat {
                        continue;
                    }
                    if *history == pattern.buy_pattern {
                        matched_pattern = Some(pattern);
                        break;
                    }
                }

                if let Some(pattern) = matched_pattern {
                    sim.tp_levels = pattern.tp_threshold.clone();
                    sim.sell_amounts = pattern.sell_amounts.clone();
                    sim.matched_at = Instant::now(); // reset for confirmation delay
                    sim.pattern_label = format!(
                        "SERVER_BUNDLE({},{},len={})",
                        mint_pat.0, mint_pat.1, pattern.buy_pattern.len()
                    );

                    let mc = sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                    info!(
                        "\n🎯 [SIM] [BUNDLE_MATCH]\n\
                         │  Pattern:    {}\n\
                         │  Mint:       {}\n\
                         │  Mint CU:    ({}, {})\n\
                         │  Buy Bundle: {:?}\n\
                         │  MC:         {:.2} SOL\n\
                         │  TP Levels:  {:?}%\n\
                         │  Sell Amts:  {:?}%\n\
                         └──────────────────────",
                        sim.pattern_label, mint,
                        mint_pat.0, mint_pat.1,
                        pattern.buy_pattern,
                        mc,
                        sim.tp_levels, sim.sell_amounts,
                    );
                }
            }
        }

        // ── Pumpfun Sell events → update price ──
        for sell_event in sell_events.iter() {
            if let Some(sim) = tokens.get_mut(&sell_event.mint) {
                let new_price = (sell_event.virtual_sol_reserves as f64 / 1e9)
                    / (sell_event.virtual_token_reserves as f64 / 1e6);
                sim.current_price = new_price;
                sim.max_price = sim.max_price.max(new_price);
                sim.tx_count += 1;

                if sim.buy_confirmed {
                    self.check_tp_sl(sim);
                }
            }
        }

        // ── Migration events ──
        for (pool_accounts, _pool_event) in
            create_pool_accounts.iter().zip(create_pool_events.iter())
        {
            if let Some(sim) = tokens.get_mut(&pool_accounts.base_mint) {
                sim.is_migrated = true;
                info!(
                    "\n🔄 [SIM] [MIGRATED]\n\
                     │  Pattern:  {}\n\
                     │  Mint:     {}\n\
                     │  MC:       {:.2} SOL\n\
                     └──────────────────────",
                    sim.pattern_label, sim.mint,
                    sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                );
            }
        }

        // ── Pumpswap buy/sell events ──
        for (i, ps_buy) in pumpswap_buy_events.iter().enumerate() {
            let mint = pumpswap_buy_accs[i].base_mint;
            if let Some(sim) = tokens.get_mut(&mint) {
                let new_price = (ps_buy.pool_quote_token_reserves as f64 / 1e9)
                    / (ps_buy.pool_base_token_reserves as f64 / 1e6);
                sim.current_price = new_price;
                sim.max_price = sim.max_price.max(new_price);
                sim.tx_count += 1;

                if !sim.buy_confirmed
                    && !sim.tp_levels.is_empty()
                    && sim.matched_at.elapsed() >= self.confirmation_delay
                {
                    sim.buy_confirmed = true;
                    sim.buy_price = new_price;
                    let mc = new_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                    let price_change = ((new_price / sim.mint_price) - 1.0) * 100.0;
                    info!(
                        "\n💰 [SIM] [BUY] (Pumpswap)\n\
                         │  Pattern:    {}\n\
                         │  Mint:       {}\n\
                         │  Buy MC:     {:.2} SOL\n\
                         │  Δ MC:       {:+.2}%\n\
                         │  Amount:     {:.4} SOL\n\
                         └──────────────────────",
                        sim.pattern_label, sim.mint,
                        mc, price_change,
                        self.buy_amount_sol,
                    );
                }

                if sim.buy_confirmed {
                    self.check_tp_sl(sim);
                }
            }
        }

        for (i, ps_sell) in pumpswap_sell_events.iter().enumerate() {
            let mint = pumpswap_sell_accs[i].base_mint;
            if let Some(sim) = tokens.get_mut(&mint) {
                let new_price = (ps_sell.pool_quote_token_reserves as f64 / 1e9)
                    / (ps_sell.pool_base_token_reserves as f64 / 1e6);
                sim.current_price = new_price;
                sim.max_price = sim.max_price.max(new_price);
                sim.tx_count += 1;

                if sim.buy_confirmed {
                    self.check_tp_sl(sim);
                }
            }
        }

        // Move completed tokens out of active tracking
        let completed_mints: Vec<Pubkey> = tokens
            .iter()
            .filter(|(_, s)| s.outcome != SimOutcome::Pending)
            .map(|(k, _)| *k)
            .collect();

        let mut completed = self.completed.lock().expect("completed lock");
        for mint in completed_mints {
            if let Some(sim) = tokens.remove(&mint) {
                completed.push(sim);
            }
        }
    }

    fn check_tp_sl(&self, sim: &mut SimToken) {
        if sim.outcome != SimOutcome::Pending {
            return;
        }

        // ── Stop Loss check ──
        if sim.current_price < sim.buy_price * self.stop_loss_pct {
            sim.sl_triggered = true;
            sim.exit_price = sim.current_price;
            let remaining_pct = 100.0 - sim.total_sold_pct;
            let pnl_from_remaining =
                remaining_pct * (sim.current_price / sim.buy_price - 1.0);

            // P&L from already-sold portions
            let pnl_from_sold: f64 = sim
                .tp_triggered_at
                .iter()
                .zip(sim.sell_amounts.iter())
                .map(|(tp_price, sell_pct)| sell_pct * (tp_price / sim.buy_price - 1.0))
                .sum();

            sim.pnl_pct = (pnl_from_sold + pnl_from_remaining) / 100.0 * 100.0;
            sim.outcome = if sim.total_sold_pct > 0.0 {
                SimOutcome::Partial
            } else {
                SimOutcome::SlHit
            };
            sim.exit_reason = format!(
                "SL at MC {:.2} (buy MC: {:.2}, loss: {:.2}%)",
                sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                sim.buy_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                sim.pnl_pct
            );

            let mc = sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
            let buy_mc = sim.buy_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
            let sol_pnl = self.buy_amount_sol * sim.pnl_pct / 100.0;
            info!(
                "\n🔴 [SIM] [SELL] [SL]\n\
                 │  Pattern:    {}\n\
                 │  Mint:       {}\n\
                 │  Buy MC:     {:.2} SOL\n\
                 │  Sell MC:    {:.2} SOL\n\
                 │  PnL:        {:.2}% ({:+.6} SOL)\n\
                 │  Sold Prior: {:.0}%\n\
                 └──────────────────────",
                sim.pattern_label, sim.mint,
                buy_mc, mc,
                sim.pnl_pct, sol_pnl,
                sim.total_sold_pct,
            );
            return;
        }

        // ── Take Profit check ──
        if sim.next_tp_index < sim.tp_levels.len() {
            let tp_pct = sim.tp_levels[sim.next_tp_index];
            let threshold = tp_pct / 100.0 * self.real_tp_multiply;

            if sim.current_price > sim.buy_price * threshold {
                let sell_pct = sim.sell_amounts[sim.next_tp_index];
                sim.tp_triggered_at.push(sim.current_price);
                sim.total_sold_pct += sell_pct;
                sim.next_tp_index += 1;

                let mc = sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                let buy_mc = sim.buy_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                let price_mult = sim.current_price / sim.buy_price;
                let this_tp_pnl_pct = (price_mult - 1.0) * 100.0;
                let this_tp_sol = self.buy_amount_sol * (sell_pct / 100.0) * price_mult;
                info!(
                    "\n🟢 [SIM] [SELL] [TP{}]\n\
                     │  Pattern:    {}\n\
                     │  Mint:       {}\n\
                     │  Buy MC:     {:.2} SOL\n\
                     │  Sell MC:    {:.2} SOL  ({:.2}x)\n\
                     │  This TP:    {:.0}% sold → {:.6} SOL return\n\
                     │  Price PnL:  {:+.2}%\n\
                     │  Total Sold: {:.0}%\n\
                     └──────────────────────",
                    sim.next_tp_index,
                    sim.pattern_label, sim.mint,
                    buy_mc, mc, price_mult,
                    sell_pct, this_tp_sol,
                    this_tp_pnl_pct,
                    sim.total_sold_pct,
                );

                // All TPs hit → close position
                if sim.next_tp_index >= sim.tp_levels.len() || sim.total_sold_pct >= 100.0 {
                    sim.exit_price = sim.current_price;
                    let pnl: f64 = sim
                        .tp_triggered_at
                        .iter()
                        .zip(sim.sell_amounts.iter())
                        .map(|(tp_price, sell_pct)| sell_pct * (tp_price / sim.buy_price - 1.0))
                        .sum();
                    sim.pnl_pct = pnl / 100.0 * 100.0;
                    sim.outcome = SimOutcome::TpHit;
                    sim.exit_reason = format!(
                        "All TPs hit | final MC: {:.2} SOL | PnL: {:.2}%",
                        sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64, sim.pnl_pct
                    );

                    let sol_pnl = self.buy_amount_sol * sim.pnl_pct / 100.0;
                    info!(
                        "\n✅ [SIM] [CLOSED] All TPs hit\n\
                         │  Pattern:    {}\n\
                         │  Mint:       {}\n\
                         │  Net PnL:    {:.2}% ({:+.6} SOL)\n\
                         └──────────────────────",
                        sim.pattern_label, sim.mint,
                        sim.pnl_pct, sol_pnl,
                    );
                }
            }
        }
    }

    /// Finalize all pending tokens (called when simulation ends)
    pub fn finalize(&self) {
        let mut tokens = self.tokens.lock().expect("tokens lock");
        let mut completed = self.completed.lock().expect("completed lock");

        for (_, mut sim) in tokens.drain() {
            if sim.outcome == SimOutcome::Pending {
                if sim.buy_confirmed {
                    sim.exit_price = sim.current_price;
                    let remaining_pct = 100.0 - sim.total_sold_pct;
                    let pnl_from_remaining =
                        remaining_pct * (sim.current_price / sim.buy_price - 1.0);
                    let pnl_from_sold: f64 = sim
                        .tp_triggered_at
                        .iter()
                        .zip(sim.sell_amounts.iter())
                        .map(|(tp_price, sell_pct)| sell_pct * (tp_price / sim.buy_price - 1.0))
                        .sum();
                    sim.pnl_pct = (pnl_from_sold + pnl_from_remaining) / 100.0 * 100.0;
                    sim.outcome = SimOutcome::Timeout;
                    sim.exit_reason = format!(
                        "Holding | last MC: {:.2} SOL | PnL: {:.2}%",
                        sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64, sim.pnl_pct
                    );
                } else {
                    sim.outcome = SimOutcome::Timeout;
                    sim.exit_reason = "Never confirmed (no trades after match)".to_string();
                }
            }
            completed.push(sim);
        }
    }

    pub fn get_results(&self) -> Vec<SimToken> {
        self.completed.lock().expect("completed lock").clone()
    }

    pub fn get_active_count(&self) -> usize {
        self.tokens.lock().expect("tokens lock").len()
    }

    pub fn get_total_tx(&self) -> u64 {
        *self.total_tx_processed.lock().expect("tx counter lock")
    }

    pub fn get_pattern_counts(&self) -> HashMap<String, u64> {
        self.match_count_per_pattern
            .lock()
            .expect("pattern count lock")
            .clone()
    }

    pub fn get_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}
