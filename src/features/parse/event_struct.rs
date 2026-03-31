use crate::*;
use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct InstructionRawData {
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
    pub program_id_index: u32,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct BudgetComputeUnitLimit {
    pub unit: u32,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct BudgetComputeUnitPrice {
    pub micro_lamports: u64,
}

////////Struct pumpfun
///
///
#[derive(Debug, Clone, BorshDeserialize)]
pub struct MintEvent {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub user: Pubkey,
    pub creator: Pubkey,
    pub timestamp: i64,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,
    pub token_program: Pubkey,
    pub is_mayhem_mode: bool,
    pub is_cashback_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct MintInstructionAccounts {
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub associated_bonding_curve: Pubkey,
    pub user: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub event_authority: Pubkey,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct PumpfunTradeEvent {
    pub mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub is_buy: bool,
    pub user: Pubkey,
    pub timestamp: i64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub fee_recipient: Pubkey,
    pub fee_basis_points: u64,
    pub fee: u64,
    pub creator: Pubkey,
    pub creator_fee_basis_points: u64,
    pub creator_fee: u64,
}

#[derive(Debug, Clone)]
pub struct PumpfunBuyEvent {
    pub mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub user: Pubkey,
    pub timestamp: i64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub fee_recipient: Pubkey,
    pub fee_basis_points: u64,
    pub fee: u64,
    pub creator: Pubkey,
    pub creator_fee_basis_points: u64,
    pub creator_fee: u64,
}

#[derive(Debug, Clone)]
pub struct PumpfunSellEvent {
    pub mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub user: Pubkey,
    pub timestamp: i64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub fee_recipient: Pubkey,
    pub fee_basis_points: u64,
    pub fee: u64,
    pub creator: Pubkey,
    pub creator_fee_basis_points: u64,
    pub creator_fee: u64,
}

#[derive(Debug, Clone)]
pub struct PumpfunBuyInstructionAccounts {
    pub global: Pubkey,
    pub fee_recipient: Pubkey,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub associated_bonding_curve: Pubkey,
    pub associated_user: Pubkey,
    pub user: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
    pub creator_vault: Pubkey,
    pub event_authority: Pubkey,
    pub program: Pubkey,
    pub global_volume_accumulator: Pubkey,
    pub user_volume_accumulator: Pubkey,
    pub fee_config: Pubkey,
    pub fee_program: Pubkey,
}

#[derive(Debug, Clone)]
pub struct PumpfunSellInstructionAccounts {
    pub global: Pubkey,
    pub fee_recipient: Pubkey,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub associated_bonding_curve: Pubkey,
    pub associated_user: Pubkey,
    pub user: Pubkey,
    pub system_program: Pubkey,
    pub creator_vault: Pubkey,
    pub token_program: Pubkey,
    pub event_authority: Pubkey,
    pub program: Pubkey,
    pub fee_config: Pubkey,
    pub fee_program: Pubkey,
}

/////////////Struct pumpswap
#[derive(Debug, Clone, BorshDeserialize)]
pub struct PumpswapBuyEvent {
    pub timestamp: i64,
    pub base_amount_out: u64,
    pub max_quote_amount_in: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub quote_amount_in: u64,
    pub lp_fee_basis_points: u64,
    pub lp_fee: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee: u64,
    pub quote_amount_in_with_lp_fee: u64,
    pub user_quote_amount_in: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub coin_creator_fee_basis_points: u64,
    pub coin_creator_fee: u64,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct PumpswapSellEvent {
    pub timestamp: i64,
    pub base_amount_in: u64,
    pub min_quote_amount_out: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub quote_amount_out: u64,
    pub lp_fee_basis_points: u64,
    pub lp_fee: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee: u64,
    pub quote_amount_out_without_lp_fee: u64,
    pub user_quote_amount_out: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub coin_creator_fee_basis_points: u64,
    pub coin_creator_fee: u64,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct PumpswapBuyInstructionAccounts {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub global_config: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
    pub system_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub event_authority: Pubkey,
    pub program: Pubkey,
    pub coin_creator_vault_ata: Pubkey,
    pub coin_creator_vault_authority: Pubkey,
    pub global_volume_accumulator: Pubkey,
    pub user_volume_accumulator: Pubkey,
    pub fee_config: Pubkey,
    pub fee_program: Pubkey,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct PumpswapSellInstructionAccounts {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub global_config: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
    pub system_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub event_authority: Pubkey,
    pub program: Pubkey,
    pub coin_creator_vault_ata: Pubkey,
    pub coin_creator_vault_authority: Pubkey,
    pub fee_config: Pubkey,
    pub fee_program: Pubkey,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct MigrateInstructionAccounts {
    pub global: Pubkey,
    pub withdraw_authority: Pubkey,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub associated_bonding_curve: Pubkey,
    pub user: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
    pub pump_amm_program: Pubkey,
    pub pool: Pubkey,
    pub pool_authority: Pubkey,
    pub pool_authority_mint_account: Pubkey,
    pub pool_authority_wsol_account: Pubkey,
    pub amm_global_config: Pubkey,
    pub wsol_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub user_pool_token_account: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub token_2022_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub pump_amm_event_authority: Pubkey,
    pub event_authority: Pubkey,
    pub pump_fun_program: Pubkey,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct CreatePoolInstructionAccounts {
    pub pool: Pubkey,                     // #1 - Pool
    pub global_config: Pubkey,            // #2 - Global Config
    pub creator: Pubkey,                  // #3 - Creator
    pub base_mint: Pubkey,                // #4 - Base Mint (GigaHouse)
    pub quote_mint: Pubkey,               // #5 - Quote Mint (WSOL)
    pub lp_mint: Pubkey,                  // #6 - LP Mint
    pub user_base_token_account: Pubkey,  // #7 - User Base Token Account
    pub user_quote_token_account: Pubkey, // #8 - User Quote Token Account
    pub user_pool_token_account: Pubkey,  // #9 - User Pool Token Account
    pub pool_base_token_account: Pubkey,  // #10 - Pool Base Token Account
    pub pool_quote_token_account: Pubkey, // #11 - Pool Quote Token Account
    pub system_program: Pubkey,           // #12 - System Program
    pub token_2022_program: Pubkey,       // #13 - Token 2022 Program
    pub base_token_program: Pubkey,       // #14 - Base Token Program
    pub quote_token_program: Pubkey,      // #15 - Quote Token Program
    pub associated_token_program: Pubkey, // #16 - Associated Token Program
    pub event_authority: Pubkey,          // #17 - Event Authority
    pub pump_amm_program: Pubkey,         // #18 - Program (Pump.fun AMM)
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct CreatePoolEventData {
    pub timestamp: i64,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_mint_decimals: u8,
    pub quote_mint_decimals: u8,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub pool_base_amount: u64,
    pub pool_quote_amount: u64,
    pub minimum_liquidity: u64,
    pub initial_liquidity: u64,
    pub lp_token_amount_out: u64,
    pub pool_bump: u8,
    pub pool: Pubkey,
    pub lp_mint: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub is_mayhem_mode: bool,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct PumpfunBondingCurve {
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
    pub creator: Pubkey,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct PumpAMMPool {
    pub pool_bump: u8,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub lp_supply: u64,
    pub coin_creator: Pubkey,
    pub is_mayhem_mode: bool,
}

pub async fn fetch_and_decode_pool(pool: Pubkey) -> Option<PumpAMMPool> {
    match RPC_CLIENT.get_account_data(&pool).await {
        Ok(pool_data) => {
            println!("Fetched account data for pool: {}", pool);
            match PumpAMMPool::try_from_slice(&pool_data[8..244]) {
                Ok(pool_state) => {
                    println!("Successfully decoded pool state for: {}", pool);
                    Some(pool_state)
                }
                Err(e) => {
                    println!("Failed to decode pool state for {}: {:?}", pool, e);
                    None
                }
            }
        }
        Err(e) => {
            println!("Failed to fetch account data for {}: {:?}", pool, e);
            None
        }
    }
}
