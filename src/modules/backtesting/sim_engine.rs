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
    pub pending_manual_pattern: Option<ManualPattern>,

    // Metadata
    pub tx_count: u64,
    pub creator: Pubkey,
    pub is_migrated: bool,
    pub exit_reason: String,

    // Fees
    pub total_fees_sol: f64,
    pub sell_count: u32,

    // Per-token overrides (from pattern or engine defaults)
    pub buy_amount_sol: f64,
    pub stop_loss_pct: f64,
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
    // Fees
    pub buy_fee_sol: f64,  // landing_fee + buy_priority_fee
    pub sell_fee_sol: f64, // sell_priority_fee only (no landing fee)
}

impl SimEngine {
    pub fn new() -> Self {
        let sim = &CONFIG.simulation_setting;

        let base_tx_fee = 0.000005; // 5000 lamports
        let landing_fee = match sim.landing_service.as_str() {
            "ZERO_SLOT" => sim.zero_slot_fee,
            _ => sim.helius_fee,
        };
        let buy_priority = sim.buy_compute_unit_limit as f64 * sim.buy_micro_lamports as f64 / 1e15;
        let sell_priority = sim.sell_compute_unit_limit as f64 * sim.sell_micro_lamports as f64 / 1e15;

        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
            completed: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
            buy_amount_sol: sim.buy_amount_sol,
            stop_loss_pct: sim.stop_loss / 100.0,
            real_tp_multiply: sim.real_tp_multiply / 100.0,
            confirmation_delay: Duration::from_millis(sim.confirmation_delay_ms),
            match_count_per_pattern: Arc::new(Mutex::new(HashMap::new())),
            total_tx_processed: Arc::new(Mutex::new(0)),
            buy_fee_sol: landing_fee + buy_priority + base_tx_fee,
            sell_fee_sol: sell_priority + base_tx_fee,
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
                let needs_bundle = manual_pat.needs_bundle_buy_confirmation();

                let token_buy_sol = manual_pat.buy_amount_sol.unwrap_or(self.buy_amount_sol);
                let token_sl_pct = manual_pat.stop_loss.map(|v| v / 100.0).unwrap_or(self.stop_loss_pct);

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
                    tp_levels: if needs_bundle { Vec::new() } else { manual_pat.take_profit.clone() },
                    sell_amounts: if needs_bundle { Vec::new() } else { manual_pat.sell_amounts.clone() },
                    next_tp_index: 0,
                    sl_triggered: false,
                    tp_triggered_at: Vec::new(),
                    outcome: SimOutcome::Pending,
                    total_sold_pct: 0.0,
                    pnl_pct: 0.0,
                    mint_cu: (unit, price),
                    buy_tx_history: Vec::new(),
                    pending_manual_pattern: if needs_bundle { Some(manual_pat.clone()) } else { None },
                    tx_count: 0,
                    creator: mint_event.creator,
                    is_migrated: false,
                    exit_reason: String::new(),
                    total_fees_sol: 0.0,
                    sell_count: 0,
                    buy_amount_sol: token_buy_sol,
                    stop_loss_pct: token_sl_pct,
                };

                let mut tokens = self.tokens.lock().expect("tokens lock");
                tokens.insert(mint, sim_token);
                minted_in_this_tx.insert(mint);

                let mut counts = self.match_count_per_pattern.lock().expect("pattern count lock");
                *counts.entry(manual_pat.label.clone()).or_insert(0) += 1;

                let mc = initial_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                if needs_bundle {
                    // No log — only log when full pattern matches (bundle buy CU confirmed)
                } else {
                    info!(
                        "\n📌 [SIM] [MANUAL_MATCH]\n\
                         │  Pattern:    {}\n\
                         │  Mint:       {}\n\
                         │  CU:         ({}, {})\n\
                         │  MC:         {:.2} SOL\n\
                         │  Buy Amt:    {:.4} SOL\n\
                         │  SL:         {:.0}%\n\
                         │  TP Levels:  {:?}%\n\
                         │  Sell Amts:  {:?}%\n\
                         └──────────────────────",
                        manual_pat.label, mint, unit, price,
                        mc,
                        token_buy_sol,
                        token_sl_pct * 100.0,
                        manual_pat.take_profit, manual_pat.sell_amounts,
                    );

                    // Spawn guaranteed buy confirmation after delay
                    let tokens_arc = self.tokens.clone();
                    let delay = self.confirmation_delay;
                    let buy_fee = self.buy_fee_sol;
                    tokio::spawn(async move {
                    tokio::time::sleep(delay).await;
                    let mut tokens = tokens_arc.lock().expect("tokens lock");
                    if let Some(sim) = tokens.get_mut(&mint) {
                        if !sim.buy_confirmed && !sim.tp_levels.is_empty() {
                            sim.buy_confirmed = true;
                            sim.buy_price = sim.current_price;
                            sim.total_fees_sol += buy_fee;
                            let mc = sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                            let mint_mc = sim.mint_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                            let price_change = ((sim.current_price / sim.mint_price) - 1.0) * 100.0;
                            info!(
                                "\n💰 [SIM] [BUY]\n\
                                 │  Pattern:    {}\n\
                                 │  Mint:       {}\n\
                                 │  Buy MC:     {:.2} SOL\n\
                                 │  Mint MC:    {:.2} SOL\n\
                                 │  Δ MC:       {:+.2}%\n\
                                 │  Amount:     {:.4} SOL\n\
                                 │  Fee:        {:.6} SOL\n\
                                 │  SL:         {:.0}%\n\
                                 │  TP:         {:?}%\n\
                                 └──────────────────────",
                                sim.pattern_label, sim.mint,
                                mc, mint_mc, price_change,
                                sim.buy_amount_sol, buy_fee,
                                sim.stop_loss_pct * 100.0,
                                sim.tp_levels,
                            );
                        }
                    }
                });
                } // else (no bundle)
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
                    pending_manual_pattern: None,
                    tx_count: 0,
                    creator: mint_event.creator,
                    is_migrated: false,
                    exit_reason: String::new(),
                    total_fees_sol: 0.0,
                    sell_count: 0,
                    buy_amount_sol: self.buy_amount_sol,
                    stop_loss_pct: self.stop_loss_pct,
                };

                let mut tokens = self.tokens.lock().expect("tokens lock");
                tokens.insert(mint, sim_token);
                minted_in_this_tx.insert(mint);
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

                    let mut counts = self.match_count_per_pattern.lock().expect("pattern count lock");
                    *counts.entry(sim.pattern_label.clone()).or_insert(0) += 1;
                    drop(counts);

                    let mc = sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                    info!(
                        "\n🎯 [SIM] [BUNDLE_MATCH]\n\
                         │  Pattern:    {}\n\
                         │  Mint:       {}\n\
                         │  Mint CU:    ({}, {})\n\
                         │  Buy Bundle: {:?}\n\
                         │  MC:         {:.2} SOL\n\
                         │  Buy Amt:    {:.4} SOL\n\
                         │  SL:         {:.0}%\n\
                         │  TP Levels:  {:?}%\n\
                         │  Sell Amts:  {:?}%\n\
                         └──────────────────────",
                        sim.pattern_label, mint,
                        mint_pat.0, mint_pat.1,
                        pattern.buy_pattern,
                        mc,
                        sim.buy_amount_sol,
                        sim.stop_loss_pct * 100.0,
                        sim.tp_levels, sim.sell_amounts,
                    );

                    // Spawn guaranteed buy confirmation after delay
                    let tokens_arc = self.tokens.clone();
                    let delay = self.confirmation_delay;
                    let buy_fee = self.buy_fee_sol;
                    let mint_key = *mint;
                    tokio::spawn(async move {
                        tokio::time::sleep(delay).await;
                        let mut tokens = tokens_arc.lock().expect("tokens lock");
                        if let Some(sim) = tokens.get_mut(&mint_key) {
                            if !sim.buy_confirmed && !sim.tp_levels.is_empty() {
                                sim.buy_confirmed = true;
                                sim.buy_price = sim.current_price;
                                sim.total_fees_sol += buy_fee;
                                let mc = sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                                let mint_mc = sim.mint_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                                let price_change = ((sim.current_price / sim.mint_price) - 1.0) * 100.0;
                                info!(
                                    "\n💰 [SIM] [BUY]\n\
                                     │  Pattern:    {}\n\
                                     │  Mint:       {}\n\
                                     │  Buy MC:     {:.2} SOL\n\
                                     │  Mint MC:    {:.2} SOL\n\
                                     │  Δ MC:       {:+.2}%\n\
                                     │  Amount:     {:.4} SOL\n\
                                     │  Fee:        {:.6} SOL\n\
                                     │  SL:         {:.0}%\n\
                                     │  TP:         {:?}%\n\
                                     └──────────────────────",
                                    sim.pattern_label, sim.mint,
                                    mc, mint_mc, price_change,
                                    sim.buy_amount_sol, buy_fee,
                                    sim.stop_loss_pct * 100.0,
                                    sim.tp_levels,
                                );
                            }
                        }
                    });
                }

                // Check pending manual pattern bundle buy CU
                if let Some(manual_pat) = sim.pending_manual_pattern.clone() {
                    if sim.tp_levels.is_empty() && manual_pat.matches_bundle_buy_cu(unit, price) {
                        sim.tp_levels = manual_pat.take_profit.clone();
                        sim.sell_amounts = manual_pat.sell_amounts.clone();
                        if let Some(pat_buy) = manual_pat.buy_amount_sol {
                            sim.buy_amount_sol = pat_buy;
                        }
                        if let Some(pat_sl) = manual_pat.stop_loss {
                            sim.stop_loss_pct = pat_sl / 100.0;
                        }
                        sim.matched_at = Instant::now();
                        let label = manual_pat.label.clone();

                        let mc = sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                        info!(
                            "\n🎯 [SIM] [MANUAL_BUNDLE_MATCH]\n\
                             │  Pattern:    {}\n\
                             │  Mint:       {}\n\
                             │  Dev CU:     ({}, {})\n\
                             │  Bundle CU:  ({}, {})\n\
                             │  MC:         {:.2} SOL\n\
                             │  Buy Amt:    {:.4} SOL\n\
                             │  SL:         {:.0}%\n\
                             │  TP Levels:  {:?}%\n\
                             │  Sell Amts:  {:?}%\n\
                             └──────────────────────",
                            label, mint,
                            sim.mint_cu.0, sim.mint_cu.1,
                            unit, price, mc,
                            sim.buy_amount_sol,
                            sim.stop_loss_pct * 100.0,
                            sim.tp_levels, sim.sell_amounts,
                        );

                        sim.pending_manual_pattern = None;

                        let tokens_arc = self.tokens.clone();
                        let delay = self.confirmation_delay;
                        let buy_fee = self.buy_fee_sol;
                        let mint_key = *mint;
                        tokio::spawn(async move {
                            tokio::time::sleep(delay).await;
                            let mut tokens = tokens_arc.lock().expect("tokens lock");
                            if let Some(sim) = tokens.get_mut(&mint_key) {
                                if !sim.buy_confirmed && !sim.tp_levels.is_empty() {
                                    sim.buy_confirmed = true;
                                    sim.buy_price = sim.current_price;
                                    sim.total_fees_sol += buy_fee;
                                    let mc = sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                                    let mint_mc = sim.mint_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                                    let price_change = ((sim.current_price / sim.mint_price) - 1.0) * 100.0;
                                    info!(
                                        "\n💰 [SIM] [BUY]\n\
                                         │  Pattern:    {}\n\
                                         │  Mint:       {}\n\
                                         │  Buy MC:     {:.2} SOL\n\
                                         │  Mint MC:    {:.2} SOL\n\
                                         │  Δ MC:       {:+.2}%\n\
                                         │  Amount:     {:.4} SOL\n\
                                         │  Fee:        {:.6} SOL\n\
                                         │  SL:         {:.0}%\n\
                                         │  TP:         {:?}%\n\
                                         └──────────────────────",
                                        label, mint_key,
                                        mc, mint_mc, price_change,
                                        sim.buy_amount_sol, buy_fee,
                                        sim.stop_loss_pct * 100.0,
                                        sim.tp_levels,
                                    );
                                }
                            }
                        });
                    }
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
        if sim.current_price < sim.buy_price * sim.stop_loss_pct {
            sim.sl_triggered = true;
            sim.exit_price = sim.current_price;
            sim.sell_count += 1;
            sim.total_fees_sol += self.sell_fee_sol;
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

            let fee_pct = (sim.total_fees_sol / sim.buy_amount_sol) * 100.0;
            sim.pnl_pct = (pnl_from_sold + pnl_from_remaining) / 100.0 * 100.0 - fee_pct;
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
            let sol_pnl = sim.buy_amount_sol * sim.pnl_pct / 100.0;
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
                sim.sell_count += 1;
                sim.total_fees_sol += self.sell_fee_sol;

                let mc = sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                let buy_mc = sim.buy_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                let price_mult = sim.current_price / sim.buy_price;
                let this_tp_pnl_pct = (price_mult - 1.0) * 100.0;
                let this_tp_sol = sim.buy_amount_sol * (sell_pct / 100.0) * price_mult;
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
                    let fee_pct = (sim.total_fees_sol / sim.buy_amount_sol) * 100.0;
                    sim.pnl_pct = pnl / 100.0 * 100.0 - fee_pct;
                    sim.outcome = SimOutcome::TpHit;
                    sim.exit_reason = format!(
                        "All TPs hit | final MC: {:.2} SOL | PnL: {:.2}%",
                        sim.current_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64, sim.pnl_pct
                    );

                    let sol_pnl = sim.buy_amount_sol * sim.pnl_pct / 100.0;
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
                    let fee_pct = (sim.total_fees_sol / sim.buy_amount_sol) * 100.0;
                    sim.pnl_pct = (pnl_from_sold + pnl_from_remaining) / 100.0 * 100.0 - fee_pct;
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
            // Only include tokens that were fully matched (have tp_levels)
            if !sim.tp_levels.is_empty() {
                completed.push(sim);
            }
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
