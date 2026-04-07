use crate::*;
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

// ── Value condition for CU fields ──

#[derive(Debug, Clone, PartialEq)]
pub enum ValueCondition {
    Null,
    NotNull,
    Exact(u64),
}

impl ValueCondition {
    pub fn matches_u32(&self, val: u32) -> bool {
        match self {
            ValueCondition::Null => val == 0,
            ValueCondition::NotNull => val != 0,
            ValueCondition::Exact(expected) => val as u64 == *expected,
        }
    }

    pub fn matches_u64(&self, val: u64) -> bool {
        match self {
            ValueCondition::Null => val == 0,
            ValueCondition::NotNull => val != 0,
            ValueCondition::Exact(expected) => val == *expected,
        }
    }
}

fn parse_value_condition(raw: &str) -> ValueCondition {
    let upper = raw.trim().to_uppercase();
    match upper.as_str() {
        "NULL" => ValueCondition::Null,
        "NOT_NULL" => ValueCondition::NotNull,
        _ => {
            if let Ok(val) = raw.trim().parse::<u64>() {
                ValueCondition::Exact(val)
            } else {
                ValueCondition::Null
            }
        }
    }
}

// ── Buy instruction amount condition ──

#[derive(Debug, Clone, PartialEq)]
pub enum AmountCondition {
    Any,
    DivisibleBy(u64),
    Exact(u64),
}

impl AmountCondition {
    pub fn matches(&self, val: u64) -> bool {
        match self {
            AmountCondition::Any => true,
            AmountCondition::DivisibleBy(divisor) => *divisor != 0 && val % divisor == 0,
            AmountCondition::Exact(expected) => val == *expected,
        }
    }
}

// ── Buy instruction condition ──

#[derive(Debug, Clone)]
pub struct BuyIxCondition {
    pub name: String,
    pub amount_condition: AmountCondition,
}

// ── Manual pattern definition (parsed, ready for matching) ──

#[derive(Debug, Clone)]
pub struct ManualPattern {
    pub label: String,
    pub cu_price: Option<ValueCondition>,
    pub cu_limit: Option<ValueCondition>,
    pub mint_instructions: Option<Vec<String>>,
    pub buy_ix_condition: Option<BuyIxCondition>,
    pub bundle_buy_cu_limit: Option<ValueCondition>,
    pub bundle_buy_cu_price: Option<ValueCondition>,
    pub stop_loss: Option<f64>,
    pub take_profit: Vec<f64>,
    pub sell_amounts: Vec<f64>,
    pub token_version: Option<String>,       // "V1", "V2"
    pub alt_addresses: Option<Vec<Pubkey>>,
    pub mint_tx_version: Option<String>,     // "Legacy", "V0"
    pub buy_amount_sol: Option<f64>,
}

// ── Raw DSL input — all fields optional except take_profit ──

#[derive(Debug, Clone)]
pub struct ManualPatternRaw {
    pub label: Option<String>,
    pub dev_cu_limit: Option<String>,
    pub dev_cu_price: Option<String>,
    pub mint_instructions: Option<String>,
    pub dev_buy_instruction_data: Option<BuyIxRaw>,
    pub bundle_buy_cu_limit: Option<String>,
    pub bundle_buy_cu_price: Option<String>,
    pub stop_loss: Option<f64>,
    pub take_profit: Vec<f64>,
    pub sell_amounts: Option<Vec<f64>>,
    pub token_version: Option<String>,
    pub alt_addresses: Option<Vec<String>>,
    pub mint_tx_version: Option<String>,
    pub buy_amount_sol: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuyIxRaw {
    pub name: String,
    pub amount: String,
}



// ── Parse amount condition from DSL string ──

fn parse_amount_condition(raw: &str) -> AmountCondition {
    let upper = raw.to_uppercase();
    if upper == "NULL" || upper == "ANY" {
        return AmountCondition::Any;
    }

    // "SpendableSolIn>>DIVIDED>>1000000000" or "AMOUNT>>DIVIDED>>1000000000"
    if upper.contains(">>DIVIDED>>") {
        let parts: Vec<&str> = raw.split(">>").collect();
        if parts.len() >= 3 {
            if let Ok(divisor) = parts[2].trim().parse::<u64>() {
                return AmountCondition::DivisibleBy(divisor);
            }
        }
    }

    if let Ok(val) = raw.parse::<u64>() {
        return AmountCondition::Exact(val);
    }

    AmountCondition::Any
}

// ── Convert raw DSL → ManualPattern ──

impl ManualPatternRaw {
    pub fn parse(self, index: usize) -> Result<ManualPattern, String> {
        let label = self
            .label
            .unwrap_or_else(|| format!("PATTERN_{}", index + 1));

        let cu_price = self.dev_cu_price.as_deref().map(parse_value_condition);
        let cu_limit = self.dev_cu_limit.as_deref().map(parse_value_condition);

        let bundle_buy_cu_limit = self.bundle_buy_cu_limit.as_deref().map(parse_value_condition);
        let bundle_buy_cu_price = self.bundle_buy_cu_price.as_deref().map(parse_value_condition);

        let mint_instructions = self.mint_instructions.map(|s| {
            s.split(">>")
                .map(|part| part.trim().to_string())
                .collect::<Vec<String>>()
        });

        let buy_ix_condition = self.dev_buy_instruction_data.map(|raw| {
            let amount_condition = parse_amount_condition(&raw.amount);
            BuyIxCondition {
                name: raw.name,
                amount_condition,
            }
        });

        let stop_loss = self.stop_loss.filter(|&v| v != 0.0);
        let buy_amount_sol = self.buy_amount_sol.filter(|&v| v > 0.0);

        let take_profit = self.take_profit;
        if take_profit.is_empty() {
            return Err(format!("{}: take_profit must not be empty", label));
        }

        let sell_amounts = self.sell_amounts.unwrap_or_else(|| {
            vec![100.0 / take_profit.len() as f64; take_profit.len()]
        });

        if sell_amounts.len() != take_profit.len() {
            return Err(format!(
                "{}: sell_amounts length ({}) != take_profit length ({})",
                label,
                sell_amounts.len(),
                take_profit.len()
            ));
        }

        let token_version = self.token_version;

        let alt_addresses = self.alt_addresses.map(|addrs| {
            addrs
                .iter()
                .filter_map(|s| Pubkey::from_str(s.trim()).ok())
                .collect::<Vec<Pubkey>>()
        });

        let mint_tx_version = self.mint_tx_version;

        Ok(ManualPattern {
            label,
            cu_price,
            cu_limit,
            mint_instructions,
            buy_ix_condition,
            bundle_buy_cu_limit,
            bundle_buy_cu_price,
            stop_loss,
            take_profit,
            sell_amounts,
            token_version,
            alt_addresses,
            mint_tx_version,
            buy_amount_sol,
        })
    }
}

// ── Matching: only check fields that are defined ──

impl ManualPattern {
    pub fn matches(
        &self,
        cu_limit: u32,
        cu_price: u64,
        ctx: &MintTransactionContext,
    ) -> bool {
        if let Some(cond) = &self.cu_limit {
            if !cond.matches_u32(cu_limit) {
                return false;
            }
        }

        if let Some(cond) = &self.cu_price {
            if !cond.matches_u64(cu_price) {
                return false;
            }
        }

        if let Some(expected_ixs) = &self.mint_instructions {
            if ctx.all_instruction_names != *expected_ixs {
                return false;
            }
        }

        if let Some(buy_cond) = &self.buy_ix_condition {
            let buy_name = match &ctx.buy_ix_name {
                Some(name) => name.as_str(),
                None => return false,
            };
            if buy_name != buy_cond.name {
                return false;
            }
            if buy_cond.amount_condition != AmountCondition::Any {
                let amount = extract_buy_amount(buy_name, &ctx.buy_ix_data);
                if let Some(val) = amount {
                    if !buy_cond.amount_condition.matches(val) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        if let Some(expected_ver) = &self.token_version {
            let actual = match &ctx.token_version {
                TokenVersion::V1 => "V1",
                TokenVersion::V2 => "V2",
            };
            if !expected_ver.eq_ignore_ascii_case(actual) {
                return false;
            }
        }

        if let Some(expected_alts) = &self.alt_addresses {
            if !expected_alts.is_empty() && ctx.alt_addresses != *expected_alts {
                return false;
            }
        }

        if let Some(expected_tx_ver) = &self.mint_tx_version {
            let actual = match &ctx.tx_type {
                TxType::Legacy => "Legacy",
                TxType::V0 => "V0",
            };
            if !expected_tx_ver.eq_ignore_ascii_case(actual) {
                return false;
            }
        }

        true
    }

    /// Returns true if this pattern requires checking the next buy tx's CU data
    /// before giving an entry signal.
    pub fn needs_bundle_buy_confirmation(&self) -> bool {
        self.bundle_buy_cu_limit.is_some() || self.bundle_buy_cu_price.is_some()
    }

    /// Check if a buy tx's CU limit/price matches the bundle_buy filter.
    pub fn matches_bundle_buy_cu(&self, cu_limit: u32, cu_price: u64) -> bool {
        if let Some(cond) = &self.bundle_buy_cu_limit {
            if !cond.matches_u32(cu_limit) {
                return false;
            }
        }
        if let Some(cond) = &self.bundle_buy_cu_price {
            if !cond.matches_u64(cu_price) {
                return false;
            }
        }
        true
    }
}

/// Extract the primary amount from the JSON buy instruction data.
fn extract_buy_amount(buy_name: &str, buy_ix_data: &Option<String>) -> Option<u64> {
    let json_str = buy_ix_data.as_ref()?;
    let json: serde_json::Value = serde_json::from_str(json_str).ok()?;

    match buy_name {
        "Pumpfun:Buy" => json.get("amount")?.as_u64(),
        "Pumpfun:BuyExactSolIn" => json.get("spendable_sol_in")?.as_u64(),
        _ => None,
    }
}

// ── Global manual pattern cache ──

use once_cell::sync::Lazy;
use std::sync::Arc;

pub static MANUAL_PATTERNS: Lazy<Arc<Vec<ManualPattern>>> = Lazy::new(|| {
    let raw_patterns = get_raw_manual_patterns();
    let mut patterns = Vec::with_capacity(raw_patterns.len());

    for (i, raw) in raw_patterns.into_iter().enumerate() {
        match raw.parse(i) {
            Ok(pattern) => {
                println!(
                    "✅ Manual pattern loaded: {} | instructions: {:?} | TP: {:?}%",
                    pattern.label, pattern.mint_instructions, pattern.take_profit,
                );
                patterns.push(pattern);
            }
            Err(e) => {
                eprintln!("❌ Failed to parse manual pattern {}: {}", i + 1, e);
            }
        }
    }

    Arc::new(patterns)
});

pub fn get_manual_patterns() -> Arc<Vec<ManualPattern>> {
    MANUAL_PATTERNS.clone()
}
