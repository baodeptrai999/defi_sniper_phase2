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
        // ── CATCH-ALL: Buy mọi token mới qua Anti-Rug filter ──
        // Không đặt CU/instruction filter → khớp tất cả token
        // Anti-Rug 5 module sẽ bảo vệ khỏi scam/rug-pull
        ManualPatternRaw {
            label: Some("ALL_PUMPFUN_FILTERED".to_string()),
            dev_cu_limit: None,                    // Không lọc CU
            dev_cu_price: None,                    // Không lọc CU
            mint_instructions: None,               // Không lọc instruction
            dev_buy_instruction_data: None,        // Không lọc buy data
            bundle_buy_cu_limit: None,
            bundle_buy_cu_price: None,
            stop_loss: Some(50.0),                 // Cắt lỗ 50%
            take_profit: vec![150.0, 300.0],       // Chốt lời 150% và 300%
            sell_amounts: Some(vec![50.0, 50.0]),  // Bán 50% ở mỗi mốc
            token_version: None,                   // Cả V1 và V2
            alt_addresses: None,
            mint_tx_version: None,
            buy_amount_sol: Some(0.001),           // Mua 0.001 SOL (an toàn)
        },
    ]
}
