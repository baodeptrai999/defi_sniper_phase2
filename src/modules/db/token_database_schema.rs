use crate::*;
use colored::*;
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub struct TokenDatabaseSchema {
    pub token_mint: Pubkey,
    pub token_creator: Pubkey,
    pub token_total_supply: u64,
    pub token_price: f64,
    pub token_peak_price: f64,
    pub token_holders: Vec<String>,
    pub token_is_purchased: bool,
    pub token_balance: u64,
    pub token_buying_point_price: f64,
    pub token_marketcap: f64,
    pub token_volume: Option<f64>,
    pub tp_state: TPMode,
    pub tracked_tp_state: TPMode,
    pub ts_state: TSMode,
    pub tracked_ts_state: TSMode,
    pub ts_stop_selling_plan: TSStopSellingPlan,
    pub tp_selling_plan: TPSellingPlan,
    pub pump_fun_swap_accounts: PumpFunSwapAccounts,
    pub last_event: LastEvent,
    pub token_sniper_status: TokenSniperStatus,
    pub token_copy_trade_status: TokenCopyTradeStatus,
    pub target_buy_amount: Option<u64>,
    pub target_sell_amount: Option<u64>,
    pub token_sell_status: TokenSellStatus,
    pub bundle_tx_counter: i32,
    pub token_is_blacklisted: TokenBlacklistInfo,
}

impl TokenDatabaseSchema {
    pub fn new_from_mint(
        mint_event: MintEvent,
        mint_instruction_accounts: MintInstructionAccounts,
        tx_id: String,
    ) -> Self {
        info!(
            "[{}]\t\t\t*Mint: {}",
            "Mint".blue(),
            mint_event.mint.to_string(),
        );

        let initial_token_price = (mint_event.virtual_sol_reserves as f64 / 10f64.powi(9))
            / (mint_event.virtual_token_reserves as f64 / 10f64.powi(6));
        let initial_token_marketcap = initial_token_price * mint_event.token_total_supply as f64;
        let initial_token_holders = Vec::new();

        let token_data = Self {
            token_mint: mint_event.mint,
            token_creator: mint_event.creator,
            token_total_supply: mint_event.token_total_supply / 10u64.pow(6),
            token_balance: 0,
            token_price: initial_token_price,
            token_peak_price: initial_token_price,
            token_holders: initial_token_holders,
            token_is_purchased: false,
            token_marketcap: initial_token_marketcap,
            token_volume: Some(0.0),
            token_buying_point_price: 0.0,
            tp_state: TPMode::None,
            tracked_tp_state: TPMode::None,
            ts_state: TSMode::None,
            tracked_ts_state: TSMode::None,
            tp_selling_plan: TPSellingPlan {
                tp_1: 0,
                tp_2: 0,
                tp_3: 0,
                tp_4: 0,
                tp_5: 0,
            },
            ts_stop_selling_plan: TSStopSellingPlan {
                ts_1_stop: 0,
                ts_2_stop: 0,
                ts_3_stop: 0,
                ts_4_stop: 0,
                ts_5_stop: 0,
            },
            pump_fun_swap_accounts: PumpFunSwapAccounts::from_mint(
                &mint_instruction_accounts,
                &mint_event,
            ),
            last_event: LastEvent {
                tx_hash: tx_id,
                last_tracked_event: TokenEvent::MintTokenEvent,
                last_activity_timestamp: mint_event.timestamp,
            },
            token_sniper_status: TokenSniperStatus::TokenMinted,
            token_copy_trade_status: TokenCopyTradeStatus::None,
            target_buy_amount: None,
            target_sell_amount: None,
            token_sell_status: TokenSellStatus::None,
            bundle_tx_counter: 0,
            token_is_blacklisted: TokenBlacklistInfo::None,
        };
        let _ = TOKEN_DB.upsert(mint_event.mint.clone(), token_data.clone());
        token_data
    }

    pub fn new_from_target_buy(
        buy_event: BuyEvent,
        buy_instruction_accounts: BuyInstructionAccounts,
        tx_id: String,
    ) -> Self {
        let token_price = (buy_event.virtual_sol_reserves as f64 / 10f64.powi(9))
            / (buy_event.virtual_token_reserves as f64 / 10f64.powi(6));
        let token_marketcap = token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
        let target_amount: u64 = buy_event.sol_amount;
        let monitored_token_holders = vec![buy_event.user.to_string()];

        let token_data = Self {
            token_mint: buy_event.mint,
            token_creator: buy_event.creator,
            token_total_supply: PUMP_FUN_TOKEN_TOTAL_SUPPLY,
            token_price: token_price,
            token_peak_price: token_price,
            token_holders: monitored_token_holders,
            token_is_purchased: false,
            token_balance: 0,
            token_buying_point_price: 0.0,
            token_marketcap: token_marketcap,
            token_volume: None,
            tp_state: TPMode::None,
            tracked_tp_state: TPMode::None,
            ts_state: TSMode::None,
            tracked_ts_state: TSMode::None,
            ts_stop_selling_plan: TSStopSellingPlan {
                ts_1_stop: 0,
                ts_2_stop: 0,
                ts_3_stop: 0,
                ts_4_stop: 0,
                ts_5_stop: 0,
            },
            tp_selling_plan: TPSellingPlan {
                tp_1: 0,
                tp_2: 0,
                tp_3: 0,
                tp_4: 0,
                tp_5: 0,
            },
            pump_fun_swap_accounts: PumpFunSwapAccounts::from_target_buy(buy_instruction_accounts),
            last_event: LastEvent {
                tx_hash: tx_id,
                last_tracked_event: TokenEvent::BuyTokenEvent,
                last_activity_timestamp: buy_event.timestamp,
            },
            token_sniper_status: TokenSniperStatus::None,
            token_copy_trade_status: TokenCopyTradeStatus::TargetBought,
            target_buy_amount: Some(target_amount),
            target_sell_amount: None,
            token_sell_status: TokenSellStatus::None,
            bundle_tx_counter: 0,
            token_is_blacklisted: TokenBlacklistInfo::None,
        };
        let _ = TOKEN_DB.upsert(buy_event.mint.clone(), token_data.clone());
        token_data
    }

    pub fn update_sell_state_flag(&mut self, tx_id: String) {
        if self.token_balance > 0 {
            self.tp_state = if self.token_price > self.token_buying_point_price * *TAKE_PROFIT_5
                && self.tp_state < TPMode::TP5
            {
                update!(
                    "[TP_UPDATED]\t*MINT: {}
                    \t*TP STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_state,
                    TPMode::TP5,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                );
                TPMode::TP5
            } else if self.token_price > self.token_buying_point_price * *TAKE_PROFIT_4
                && self.tp_state < TPMode::TP4
            {
                update!(
                    "[TP_UPDATED]\t*MINT: {}
                    \t*TP STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_state,
                    TPMode::TP4,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                );
                TPMode::TP4
            } else if self.token_price > self.token_buying_point_price * *TAKE_PROFIT_3
                && self.tp_state < TPMode::TP3
            {
                update!(
                    "[TP_UPDATED]\t*MINT: {}
                    \t*TP STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_state,
                    TPMode::TP4,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                );
                TPMode::TP3
            } else if self.token_price > self.token_buying_point_price * *TAKE_PROFIT_2
                && self.tp_state < TPMode::TP2
            {
                update!(
                    "[TP_UPDATED]\t*MINT: {}
                    \t*TP STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_state,
                    TPMode::TP2,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                );
                TPMode::TP2
            } else if self.token_price > self.token_buying_point_price * *TAKE_PROFIT_1
                && self.tp_state < TPMode::TP1
            {
                update!(
                    "[TP_UPDATED]\t*MINT: {}
                    \t*TP STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_state,
                    TPMode::TP1,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                );
                TPMode::TP1
            } else if self.token_price < self.token_buying_point_price * *STOP_LOSS
                && self.tp_state < TPMode::SL
            {
                update!(
                    "[TP_UPDATED]\t*MINT: {}
                    \t*TP STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_state,
                    TPMode::SL,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                );
                TPMode::SL
            } else {
                self.tp_state.clone()
            };

            self.ts_state = if self.ts_state == TSMode::TS5Triggered
                && self.token_price < self.token_peak_price * (1.0 - *TS_5_STOP)
            {
                update!(
                    "[TS_UPDATED]\t*MINT: {}
                    \t*TS STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_state,
                    TSMode::TS5Stop,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64
                );
                TSMode::TS5Stop
            } else if self.ts_state == TSMode::TS4Triggered
                && self.token_price < self.token_peak_price * (1.0 - *TS_4_STOP)
            {
                update!(
                    "[TS_UPDATED]\t*MINT: {}
                    \t*TS STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_state,
                    TSMode::TS4Stop,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64
                );
                TSMode::TS4Stop
            } else if self.ts_state == TSMode::TS3Triggered
                && self.token_price < self.token_peak_price * (1.0 - *TS_3_STOP)
            {
                update!(
                    "[TS_UPDATED]\t*MINT: {}
                    \t*TS STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_state,
                    TSMode::TS3Stop,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64
                );
                TSMode::TS3Stop
            } else if self.ts_state == TSMode::TS2Triggered
                && self.token_price < self.token_peak_price * (1.0 - *TS_2_STOP)
            {
                update!(
                    "[TS_UPDATED]\t*MINT: {}
                    \t*TS STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_state,
                    TSMode::TS2Stop,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64
                );
                TSMode::TS2Stop
            } else if self.ts_state == TSMode::TS1Triggered
                && self.token_price < self.token_peak_price * (1.0 - *TS_1_STOP)
            {
                update!(
                    "[TS_UPDATED]\t*MINT: {}
                    \t*TS STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_state,
                    TSMode::TS1Stop,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64
                );
                TSMode::TS1Stop
            } else if self.token_price > self.token_buying_point_price * *TS_5
                && self.ts_state < TSMode::TS5Triggered
            {
                update!(
                    "[TS_UPDATED]\t*MINT: {}
                    \t*TS STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_state,
                    TSMode::TS5Triggered,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64
                );
                TSMode::TS5Triggered
            } else if self.token_price > self.token_buying_point_price * *TS_4
                && self.ts_state < TSMode::TS4Triggered
            {
                update!(
                    "[TS_UPDATED]\t*MINT: {}
                    \t*TS STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_state,
                    TSMode::TS4Triggered,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64
                );
                TSMode::TS4Triggered
            } else if self.token_price > self.token_buying_point_price * *TS_3
                && self.ts_state < TSMode::TS3Triggered
            {
                update!(
                    "[TS_UPDATED]\t*MINT: {}
                    \t*TS STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_state,
                    TSMode::TS3Triggered,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64
                );
                TSMode::TS3Triggered
            } else if self.token_price > self.token_buying_point_price * *TS_2
                && self.ts_state < TSMode::TS2Triggered
            {
                update!(
                    "[TS_UPDATED]\t*MINT: {}
                    \t*TS STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_state,
                    TSMode::TS2Triggered,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64
                );
                TSMode::TS2Triggered
            } else if self.token_price > self.token_buying_point_price * *TS_1
                && self.ts_state < TSMode::TS1Triggered
            {
                update!(
                    "[TS_UPDATED]\t*MINT: {}
                    \t*TS STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_state,
                    TSMode::TS1Triggered,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64
                );
                TSMode::TS1Triggered
            } else {
                self.ts_state.clone()
            };

            dev_log!(
                "[POOL STATE UPDATE]\t*MINT {:<12} ,
                \t*TX HASH: {}
                \t*CURRENT MC: {:.5} SOL , PEAK MC: {:.5} SOL, BUYING POINT MC: {:.5} SOL
                \t*PRICE VARIANT PCNT: {:3.5} % , FALL PCNT: {:3.5} %
                \t*ts_state: {:?} , tp_state: {:?}",
                &self.pump_fun_swap_accounts.mint.to_string(),
                solscan!(tx_id),
                &self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                &self.token_peak_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                &self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                &self.token_price * 100.0 / &self.token_buying_point_price,
                100.0 * (&self.token_peak_price - &self.token_price) / &self.token_peak_price,
                self.ts_state,
                self.tp_state,
            );
        }
    }

    pub fn update_sell_state_flag_copy_mode(&mut self, tx_id: String) {
        if self.token_balance > 0 {
            self.tp_state = if self.token_price
                > self.token_buying_point_price * *COPY_MODE_TAKE_PROFIT
                && self.tp_state < TPMode::CopyModeTp
            {
                update!(
                    "[TP_UPDATED]\t*MINT: {}
                    \t*TP STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_state,
                    TPMode::CopyModeTp,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                );
                TPMode::CopyModeTp
            } else if self.token_price < self.token_buying_point_price * *STOP_LOSS
                && self.tp_state < TPMode::SL
            {
                update!(
                    "[TP_UPDATED]\t*MINT: {}
                    \t*TP STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_state,
                    TPMode::SL,
                    self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                );
                TPMode::SL
            } else {
                self.tp_state.clone()
            };

            dev_log!(
                "[POOL STATE UPDATE]\t*MINT {:<12} ,
                \t*TX HASH: {}
                \t*CURRENT MC: {:.5} SOL , PEAK MC: {:.5} SOL, BUYING POINT MC: {:.5} SOL
                \t*PRICE VARIANT PCNT: {:3.5} % , FALL PCNT: {:3.5} %
                \t*ts_state: {:?} , tp_state: {:?}",
                &self.pump_fun_swap_accounts.mint.to_string(),
                solscan!(tx_id),
                &self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                &self.token_peak_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                &self.token_buying_point_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                &self.token_price * 100.0 / &self.token_buying_point_price,
                100.0 * (&self.token_peak_price - &self.token_price) / &self.token_peak_price,
                self.ts_state,
                self.tp_state,
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TSMode {
    None,
    TS1Triggered,
    TS1Stop,
    TS2Triggered,
    TS2Stop,
    TS3Triggered,
    TS3Stop,
    TS4Triggered,
    TS4Stop,
    TS5Triggered,
    TS5Stop,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TPMode {
    None,
    TP1,
    TP2,
    TP3,
    TP4,
    TP5,
    CopyModeTp,
    SL,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenSniperStatus {
    None,
    TokenMinted,
    SniperTradeSubmitted,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenCopyTradeStatus {
    None,
    TargetBought,
    TargetSold,
    CopyTradeSubmitted,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]

pub enum TokenBlacklistInfo {
    None,
    NotBlacklistedToken,
    BlacklistedToken,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenSellStatus {
    None,
    SellTradeSubmitted,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenEvent {
    MintTokenEvent,
    BuyTokenEvent,
    SellTokenEvent,
}

#[derive(Debug, Clone, Copy)]
pub struct TSStopSellingPlan {
    pub ts_1_stop: u64,
    pub ts_2_stop: u64,
    pub ts_3_stop: u64,
    pub ts_4_stop: u64,
    pub ts_5_stop: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct TPSellingPlan {
    pub tp_1: u64,
    pub tp_2: u64,
    pub tp_3: u64,
    pub tp_4: u64,
    pub tp_5: u64,
}

#[derive(Debug, Clone)]
pub struct LastEvent {
    pub tx_hash: String,
    pub last_tracked_event: TokenEvent,
    pub last_activity_timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct TokenHoldersInfo {
    pub holder_accounts: Vec<Pubkey>,
    pub max_holder: Option<Pubkey>,
    pub max_holder_percent: f64,
}
