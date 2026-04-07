use crate::*;
use solana_sdk::pubkey::Pubkey;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct TokenDatabaseSchema {
    pub token_mint: Pubkey,
    pub token_mint_time: Instant,
    pub token_creator: Pubkey,
    pub token_price: f64,
    pub token_max_price: f64,
    pub token_buying_point_price: f64,
    pub token_is_purchased: bool,
    pub token_is_migrated: bool,
    pub token_balance: u64,
    pub sl_state: SLMode,
    pub tracked_sl_state: SLMode,
    pub pumpfun_struct: PumpfunStruct,
    pub pumpswap_struct: Option<PumpSwapStruct>,
    pub token_trade_signal: TokenTradeSignal,
    pub token_sell_status: TokenSellStatus,
    pub mint_budget_compute_unit_limit: u32,
    pub mint_budget_compute_unit_price: u64,
    pub dev_buy_sol_lamports: Option<u64>,
    pub buy_tx_history: Vec<((u32, u64), u8)>,
    pub pending_manual_pattern: Option<ManualPattern>,
    pub token_tp_levels: Vec<f64>,
    pub token_sell_amount_percents: Vec<f64>,
    pub token_sell_plan_amounts: Vec<u64>,
    pub next_tp_index_to_sell: usize,
    pub pending_tp_sell_index: Option<usize>,
    pub pending_tp_sell_amount: u64,
    pub is_cashback_enabled: bool,
    pub override_buy_amount_sol: Option<f64>,
    pub override_stop_loss: Option<f64>,
}

impl TokenDatabaseSchema {
    pub fn new_from_mint(
        mint_event: MintEvent,
        mint_instruction_accounts: MintInstructionAccounts,
        budget_compute_data: (u32, u64),
        _tx_id: String,
    ) -> Self {
        let initial_token_price = (mint_event.virtual_sol_reserves as f64 / 10f64.powi(9))
            / (mint_event.virtual_token_reserves as f64 / 10f64.powi(6));

        let token_data = Self {
            token_mint: mint_event.mint,
            token_mint_time: Instant::now(),
            token_creator: mint_event.creator,
            token_balance: 0,
            token_price: initial_token_price,
            token_max_price: initial_token_price,
            token_is_purchased: false,
            token_is_migrated: false,
            token_buying_point_price: 0.0,
            sl_state: SLMode::None,
            tracked_sl_state: SLMode::None,
            pumpfun_struct: PumpfunStruct::from_mint(&mint_instruction_accounts, &mint_event),
            pumpswap_struct: None,
            token_trade_signal: TokenTradeSignal::None,
            mint_budget_compute_unit_limit: budget_compute_data.0,
            mint_budget_compute_unit_price: budget_compute_data.1,
            dev_buy_sol_lamports: None,
            token_sell_status: TokenSellStatus::None,
            buy_tx_history: Vec::new(),
            pending_manual_pattern: None,
            token_tp_levels: Vec::new(),
            token_sell_amount_percents: Vec::new(),
            token_sell_plan_amounts: Vec::new(),
            next_tp_index_to_sell: 0,
            pending_tp_sell_index: None,
            pending_tp_sell_amount: 0,
            is_cashback_enabled: mint_event.is_cashback_enabled,
            override_buy_amount_sol: None,
            override_stop_loss: None,
        };

        let _ = TOKEN_DB.upsert(mint_event.mint.clone(), token_data.clone());
        token_data
    }

    pub fn update_sell_state_flag(&mut self, _tx_id: String) {
        if self.token_balance == 0 {
            return;
        }

        if self.token_price < self.token_buying_point_price * self.override_stop_loss.unwrap_or(*STOP_LOSS)
            && self.sl_state != SLMode::Triggered
        {
            update!(
                "[SL_REACHED]\t*MINT: {}
                \t*MC VARIANT: {:.3} SOL (BUY) -> {:.3} SOL (NOW)",
                self.pumpfun_struct.mint,
                self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
            );
            self.sl_state = SLMode::Triggered;
        }

        if self.pending_tp_sell_index.is_none() {
            if let Some((tp_idx, threshold_pct)) = self
                .token_tp_levels
                .get(self.next_tp_index_to_sell)
                .map(|v| (self.next_tp_index_to_sell, *v))
            {
                let threshold_multiplier = threshold_pct / 100.0 * *REAL_TP_MULTIPLY;

                if self.token_price > self.token_buying_point_price * threshold_multiplier {
                    let planned_amount = self
                        .token_sell_plan_amounts
                        .get(tp_idx)
                        .copied()
                        .unwrap_or(0)
                        .min(self.token_balance);

                    if planned_amount > 0 {
                        self.pending_tp_sell_index = Some(tp_idx);
                        self.pending_tp_sell_amount = planned_amount;

                        update!(
                            "[TP{}_REACHED]\t*MINT: {}\n\t*TARGET: {}%\n\t*SELL_AMOUNT: {}",
                            tp_idx + 1,
                            self.pumpfun_struct.mint,
                            threshold_pct,
                            planned_amount,
                        );
                    } else {
                        self.next_tp_index_to_sell += 1;
                    }
                }
            }
        }
    }

    pub fn set_tp_sell_strategy(&mut self, tp_levels: Vec<f64>, sell_amount_percents: Vec<f64>) {
        self.token_tp_levels = tp_levels;
        self.token_sell_amount_percents = sell_amount_percents;
        self.token_sell_plan_amounts.clear();
        self.next_tp_index_to_sell = 0;
        self.pending_tp_sell_index = None;
        self.pending_tp_sell_amount = 0;
        self.initialize_sell_plan_if_needed();
    }

    pub fn initialize_sell_plan_if_needed(&mut self) {
        if !self.token_is_purchased
            || self.token_balance == 0
            || !self.token_sell_plan_amounts.is_empty()
            || self.token_tp_levels.is_empty()
            || self.token_sell_amount_percents.is_empty()
            || self.token_tp_levels.len() != self.token_sell_amount_percents.len()
        {
            return;
        }

        let mut plan = Vec::with_capacity(self.token_sell_amount_percents.len());
        let mut remaining = self.token_balance;

        for (i, sell_pct) in self.token_sell_amount_percents.iter().enumerate() {
            let amount = if i + 1 == self.token_sell_amount_percents.len() {
                remaining
            } else {
                let target = ((self.token_balance as f64) * (*sell_pct / 100.0)).floor() as u64;
                target.min(remaining)
            };

            plan.push(amount);
            remaining = remaining.saturating_sub(amount);
        }

        self.token_sell_plan_amounts = plan;
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum SLMode {
    None,
    Triggered,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenSellStatus {
    None,
    SellTradeSubmitted,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenTradeSignal {
    None,
    IsEntryPoint,
    EntrySubmitted,
    IsExitPoint,
    ExitSubmitted,
}
