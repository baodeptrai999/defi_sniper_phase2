use crate::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub static DYNAMIC_BUY: Lazy<DynamicBuyTracker> = Lazy::new(|| DynamicBuyTracker::new());

pub struct DynamicBuyTracker {
    states: Mutex<HashMap<String, DynamicBuyState>>,
}

struct DynamicBuyState {
    consecutive_losses: u32,
    consecutive_wins: u32,
    multiplier: f64,
}

impl DynamicBuyState {
    fn new() -> Self {
        Self {
            consecutive_losses: 0,
            consecutive_wins: 0,
            multiplier: 1.0,
        }
    }
}

impl DynamicBuyTracker {
    fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }

    /// Record a completed trade outcome per pattern.
    /// `is_profit`: true if sell_price > buy_price
    pub fn record_outcome(&self, pattern_label: &str, mint: &str, is_profit: bool) {
        if !*DYNAMIC_BUY_AMOUNT_MODE || pattern_label.is_empty() {
            return;
        }

        let mut states = self.states.lock().expect("dynamic_buy lock");
        let state = states.entry(pattern_label.to_string()).or_insert_with(DynamicBuyState::new);
        let old_mult = state.multiplier;

        if is_profit {
            state.consecutive_losses = 0;
            state.consecutive_wins += 1;

            if state.consecutive_wins >= *PROFIT_SEQUENCE {
                state.multiplier = (state.multiplier * *PROFIT_MULTIPLY).min(*MAX_BUY_AMOUNT_MULTIPLY);
                state.consecutive_wins = 0;
                info!(
                    "\n📈 [DYNAMIC_BUY] {} consecutive wins\n\
                     │  Pattern:      {}\n\
                     │  Mint:         {}\n\
                     │  Multiplier:   {:.3} → {:.3}\n\
                     └──────────────────────",
                    *PROFIT_SEQUENCE, pattern_label, mint, old_mult, state.multiplier,
                );
            }
        } else {
            state.consecutive_wins = 0;
            state.consecutive_losses += 1;

            if state.consecutive_losses >= *LOSS_SEQUENCE {
                state.multiplier *= *LOSS_MULTIPLY;
                state.consecutive_losses = 0;
                info!(
                    "\n📉 [DYNAMIC_BUY] {} consecutive losses\n\
                     │  Pattern:      {}\n\
                     │  Mint:         {}\n\
                     │  Multiplier:   {:.3} → {:.3}\n\
                     └──────────────────────",
                    *LOSS_SEQUENCE, pattern_label, mint, old_mult, state.multiplier,
                );
            }
        }
    }

    /// Get the adjusted buy amount for a specific pattern.
    /// Caps between `initial_buy_sol * MIN_BUY_AMOUNT_MULTIPLY` and `initial_buy_sol * MAX_BUY_AMOUNT_MULTIPLY`.
    pub fn adjusted_buy_amount(&self, pattern_label: &str, initial_buy_sol: f64) -> f64 {
        if !*DYNAMIC_BUY_AMOUNT_MODE || pattern_label.is_empty() {
            return initial_buy_sol;
        }

        let states = self.states.lock().expect("dynamic_buy lock");
        let multiplier = states.get(pattern_label).map(|s| s.multiplier).unwrap_or(1.0);
        let max = initial_buy_sol * *MAX_BUY_AMOUNT_MULTIPLY;
        let min = initial_buy_sol * *MIN_BUY_AMOUNT_MULTIPLY;
        (initial_buy_sol * multiplier).clamp(min, max)
    }
}
