use super::pattern_translator::{BuyIxRaw, ManualPatternRaw};

/// Define your manual patterns here. Each entry is translated into a `ManualPattern`
/// at startup and matched against incoming mint transactions.
///
/// Only define fields you want to check — undefined (None) fields are skipped.
///
/// ┌─────────────────────────────────────────────────────────────────────┐
/// │                    SAMPLE PATTERN (all fields)                      │
/// ├─────────────────────────────────────────────────────────────────────┤
/// │  ManualPatternRaw {                                                 │
/// │      label: Some("MY_PATTERN".to_string()),                         │
/// │                                                                     │
/// │      // ── CU filters ──                                            │
/// │      // "NULL" (== 0) | "NOT_NULL" (> 0) | exact e.g. "160000"      │
/// │      dev_cu_price: Some("NOT_NULL".to_string()),                    │
/// │      dev_cu_limit: Some("600000".to_string()),                      │
/// │                                                                     │
/// │      // ── Instruction sequence ──                                  │
/// │      // Program:Instruction separated by ">>"                       │
/// │      // CB:       SetComputeUnitLimit, SetComputeUnitPrice          │
/// │      // Pumpfun:  Create, CreateV2, Buy, BuyExactSolIn,             │
/// │      //           ExtendAccount, Sell, Migrate, ...                 │
/// │      // ATA:      Create, CreateIdempotent                          │
/// │      // System:   Transfer, CreateAccount,                          │
/// │      //           AdvanceNonceAccount, ...                          │
/// │      // Token2022: InitializeMint, Transfer, MintTo, ...            │
/// │      mint_instructions: Some(                                       │
/// │          "CB:SetComputeUnitLimit>>CB:SetComputeUnitPrice\           │
/// │           >>Pumpfun:CreateV2>>ATA:CreateIdempotent\                 │
/// │           >>Pumpfun:BuyExactSolIn"                                  │
/// │              .to_string()),                                         │
/// │                                                                     │
/// │      // ── Buy instruction data filter ──                           │
/// │      // name:   "Pumpfun:Buy" or "Pumpfun:BuyExactSolIn"            │
/// │      // amount: "ANY" | "DIVIDED>>divisor" | exact u64 string       │
/// │      dev_buy_instruction_data: Some(BuyIxRaw {                      │
/// │          name: "Pumpfun:BuyExactSolIn".to_string(),                 │
/// │          amount: "DIVIDED>>1000000000".to_string(),                 │
/// │      }),                                                            │
/// │                                                                     │
/// │      // ── Trade settings ──                                        │
/// │      stop_loss: Some(70.0),            // or None for config default│
/// │      take_profit: vec![200.0, 400.0],                               │
/// │      sell_amounts: Some(vec![50.0, 50.0]),                          │
/// │      buy_amount_sol: Some(0.03),       // or None for config default│
/// │                                                                     │
/// │      // ── Tx metadata filters ──                                   │
/// │      token_version: Some("V2".to_string()),  // "V1" or "V2"        │
/// │      alt_addresses: Some(vec![                                      │
/// │          "7mFD2m...".to_string(),                                   │
/// │      ]),                                                            │
/// │      mint_tx_version: Some("V0".to_string()),// "Legacy" or "V0"    │
/// │                                                                     │
/// │      // ── Bundle buy CU filter (optional) ──                       │
/// │      // If set, entry is DEFERRED until next buy tx CU matches      │
/// │      // If None, entry signal fires immediately at mint match       │
/// │      // Same syntax: "NULL" | "NOT_NULL" | exact e.g. "200000"      │
/// │      bundle_buy_cu_limit: Some("200000".to_string()),               │
/// │      bundle_buy_cu_price: Some("NOT_NULL".to_string()),             │
/// │  }                                                                  │
/// └─────────────────────────────────────────────────────────────────────┘

pub fn get_raw_manual_patterns() -> Vec<ManualPatternRaw> {
    vec![
        // ── PATTERN 1 ──
        ManualPatternRaw {
            label: Some("PATTERN_1".to_string()),
            dev_cu_limit: Some("NOT_NULL".to_string()),
            dev_cu_price: Some("NULL".to_string()),
            mint_instructions: Some("CB:SetComputeUnitLimit>>Pumpfun:CreateV2>>Pumpfun:ExtendAccount>>ATA:CreateIdempotent>>Pumpfun:BuyExactSolIn".to_string()),
            dev_buy_instruction_data: Some(BuyIxRaw {
                name: "Pumpfun:BuyExactSolIn".to_string(),
                amount: "DIVIDED>>1000000000".to_string(),
            }),
            bundle_buy_cu_limit: None,
            bundle_buy_cu_price: None,
            stop_loss: None,
            take_profit: vec![500.0],
            sell_amounts: Some(vec![100.0]),
            token_version: None,
            alt_addresses: Some(vec!["7mFD2mUtRS65XstiSAvCJuYmdesZoQwCwRJhq1p3eRMe".to_string()]),
            mint_tx_version: Some("V0".to_string()),
            buy_amount_sol: None,
        },

        ManualPatternRaw {
            label: Some("PATTERN_2".to_string()),
            dev_cu_price: Some("160000".to_string()),
            dev_cu_limit: Some("600000".to_string()),
            mint_instructions: Some("CB:SetComputeUnitLimit>>CB:SetComputeUnitPrice>>Pumpfun:Create>>Pumpfun:ExtendAccount>>System:Transfer".to_string()),
            dev_buy_instruction_data: None,
            bundle_buy_cu_limit: None,
            bundle_buy_cu_price: None,
            stop_loss: None,
            take_profit: vec![600.0],
            sell_amounts: Some(vec![100.0]),
            token_version: Some("V1".to_string()),
            alt_addresses: Some(vec!["3uJ6k2iQvehx8AwHDLMjCeFzksaxNWgF9DFNERMHxJXw".to_string()]),
            mint_tx_version: Some("V0".to_string()),
            buy_amount_sol: None,
        },

        ManualPatternRaw {
            label: Some("PATTERN_3".to_string()),
            dev_cu_price: Some("NULL".to_string()),
            dev_cu_limit: Some("NULL".to_string()),
            mint_instructions: None,
            dev_buy_instruction_data: None,
            bundle_buy_cu_limit: Some("140000".to_string()),
            bundle_buy_cu_price: Some("320000".to_string()),
            stop_loss: None,
            take_profit: vec![235.0],
            sell_amounts: Some(vec![100.0]),
            token_version: Some("V2".to_string()),
            alt_addresses: None,
            mint_tx_version: None,
            buy_amount_sol: Some(0.05),
        },

        ManualPatternRaw {
            label: Some("PATTERN_4".to_string()),
            dev_cu_price: Some("1000".to_string()),
            dev_cu_limit: Some("NOT_NULL".to_string()),
            mint_instructions: None,
            dev_buy_instruction_data: None,
            bundle_buy_cu_limit: None,
            bundle_buy_cu_price: None,
            stop_loss: None,
            take_profit: vec![135.0],
            sell_amounts: Some(vec![100.0]),
            token_version: Some("V2".to_string()),
            alt_addresses: Some(vec!["61zTJzmPuCticMHiFgXr2ohPupKqRey5SufNwEnqhsTx".to_string()]),
            mint_tx_version: None,
            buy_amount_sol: None,
        },

        ManualPatternRaw {
            label: Some("PATTERN_5".to_string()),
            dev_cu_price: Some("300000".to_string()),
            dev_cu_limit: Some("600000".to_string()),
            mint_instructions: None,
            dev_buy_instruction_data: None,
            bundle_buy_cu_limit: Some("600000".to_string()),
            bundle_buy_cu_price: Some("300000".to_string()),
            stop_loss: None,
            take_profit: vec![136.0, 192.0],
            sell_amounts: Some(vec![50.0, 50.0]),
            token_version: Some("V1".to_string()),
            alt_addresses: None,
            mint_tx_version: Some("Legacy".to_string()),
            buy_amount_sol: None,
        },

        ManualPatternRaw {
            label: Some("PATTERN_6".to_string()),
            dev_cu_price: Some("1000".to_string()),
            dev_cu_limit: Some("NOT_NULL".to_string()),
            mint_instructions: Some("CB:SetComputeUnitLimit>>CB:SetComputeUnitPrice>>System:Transfer>>Pumpfun:CreateV2>>Pumpfun:ExtendAccount>>ATA:CreateIdempotent>>Pumpfun:Buy".to_string()),
            dev_buy_instruction_data: None,
            bundle_buy_cu_limit: Some("NULL".to_string()),
            bundle_buy_cu_price: Some("NULL".to_string()),
            stop_loss: None,
            take_profit: vec![250.0],
            sell_amounts: Some(vec![100.0]),
            token_version: Some("V2".to_string()),
            alt_addresses: Some(vec!["61zTJzmPuCticMHiFgXr2ohPupKqRey5SufNwEnqhsTx".to_string()]),
            mint_tx_version: Some("V0".to_string()),
            buy_amount_sol: None,
        },

        ManualPatternRaw {
            label: Some("PATTERN_7".to_string()),
            dev_cu_price: Some("1000000".to_string()),
            dev_cu_limit: Some("300000".to_string()),
            mint_instructions: Some("CB:SetComputeUnitLimit>>CB:SetComputeUnitPrice>>System:Transfer>>System:Transfer>>Pumpfun:CreateV2>>ATA:CreateIdempotent>>Pumpfun:Buy".to_string()),
            dev_buy_instruction_data: None,
            bundle_buy_cu_limit: Some("120000".to_string()),
            bundle_buy_cu_price: Some("1000000".to_string()),
            stop_loss: Some(82.0),
            take_profit: vec![270.0],
            sell_amounts: Some(vec![100.0]),
            token_version: Some("V2".to_string()),
            alt_addresses: Some(vec!["beaaXjkvwyQ8cC9G7aExy81ygAZgc7vdrB7oif8poL2".to_string()]),
            mint_tx_version: Some("V0".to_string()),
            buy_amount_sol: None,
        },


    ]
}
