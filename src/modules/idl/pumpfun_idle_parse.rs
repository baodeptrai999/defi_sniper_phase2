//! Parsed from pump_idl.json - PumpFun Anchor IDL
//! Program ID: 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P

use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

// ============================================================
// Shared IDL Types
// ============================================================

/// Mirrors PumpFun's Anchor `OptionBool` struct: a single `bool` field (1 byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshDeserialize)]
pub struct OptionBool(pub bool);

impl OptionBool {
    pub fn is_true(&self) -> bool {
        self.0
    }
}

// ============================================================
// Instruction Discriminators (first 8 bytes of ix data)
// ============================================================

pub mod ix_discriminator {
    pub const ADMIN_SET_CREATOR: [u8; 8] = [69, 25, 171, 142, 57, 239, 13, 4];
    pub const ADMIN_SET_IDL_AUTHORITY: [u8; 8] = [8, 217, 96, 231, 144, 104, 192, 5];
    pub const ADMIN_UPDATE_TOKEN_INCENTIVES: [u8; 8] = [209, 11, 115, 87, 213, 23, 124, 204];
    pub const BUY: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
    pub const BUY_EXACT_SOL_IN: [u8; 8] = [56, 252, 116, 8, 158, 223, 205, 95];
    pub const CLAIM_CASHBACK: [u8; 8] = [37, 58, 35, 126, 190, 53, 228, 197];
    pub const CLAIM_TOKEN_INCENTIVES: [u8; 8] = [16, 4, 71, 28, 204, 1, 40, 27];
    pub const CLOSE_USER_VOLUME_ACCUMULATOR: [u8; 8] = [249, 69, 164, 218, 150, 103, 84, 138];
    pub const COLLECT_CREATOR_FEE: [u8; 8] = [20, 22, 86, 123, 198, 28, 219, 132];
    pub const CREATE: [u8; 8] = [24, 30, 200, 40, 5, 28, 7, 119];
    pub const CREATE_V2: [u8; 8] = [214, 144, 76, 236, 95, 139, 49, 180];
    pub const DISTRIBUTE_CREATOR_FEES: [u8; 8] = [165, 114, 103, 0, 121, 206, 247, 81];
    pub const EXTEND_ACCOUNT: [u8; 8] = [234, 102, 194, 203, 150, 72, 62, 229];
    pub const GET_MINIMUM_DISTRIBUTABLE_FEE: [u8; 8] = [117, 225, 127, 202, 134, 95, 68, 35];
    pub const INIT_USER_VOLUME_ACCUMULATOR: [u8; 8] = [94, 6, 202, 115, 255, 96, 232, 183];
    pub const INITIALIZE: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237];
    pub const MIGRATE: [u8; 8] = [155, 234, 231, 146, 236, 158, 162, 30];
    pub const MIGRATE_BONDING_CURVE_CREATOR: [u8; 8] = [87, 124, 52, 191, 52, 38, 214, 232];
    pub const SELL: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
    pub const SET_CREATOR: [u8; 8] = [254, 148, 255, 112, 207, 142, 170, 165];
    pub const SET_MAYHEM_VIRTUAL_PARAMS: [u8; 8] = [61, 169, 188, 191, 153, 149, 42, 97];
    pub const SET_METAPLEX_CREATOR: [u8; 8] = [138, 96, 174, 217, 48, 85, 197, 246];
    pub const SET_PARAMS: [u8; 8] = [27, 234, 178, 52, 147, 2, 187, 141];
    pub const SET_RESERVED_FEE_RECIPIENTS: [u8; 8] = [111, 172, 162, 232, 114, 89, 213, 142];
    pub const SYNC_USER_VOLUME_ACCUMULATOR: [u8; 8] = [86, 31, 192, 87, 163, 87, 79, 238];
    pub const TOGGLE_CASHBACK_ENABLED: [u8; 8] = [115, 103, 224, 255, 189, 89, 86, 195];
    pub const TOGGLE_CREATE_V2: [u8; 8] = [28, 255, 230, 240, 172, 107, 203, 171];
    pub const TOGGLE_MAYHEM_MODE: [u8; 8] = [1, 9, 111, 208, 100, 31, 255, 163];
    pub const UPDATE_GLOBAL_AUTHORITY: [u8; 8] = [227, 181, 74, 196, 208, 21, 97, 213];

    pub const ALL: &[(&str, [u8; 8])] = &[
        ("Pumpfun:AdminSetCreator", ADMIN_SET_CREATOR),
        ("Pumpfun:AdminSetIdlAuthority", ADMIN_SET_IDL_AUTHORITY),
        ("Pumpfun:AdminUpdateTokenIncentives", ADMIN_UPDATE_TOKEN_INCENTIVES),
        ("Pumpfun:Buy", BUY),
        ("Pumpfun:BuyExactSolIn", BUY_EXACT_SOL_IN),
        ("Pumpfun:ClaimCashback", CLAIM_CASHBACK),
        ("Pumpfun:ClaimTokenIncentives", CLAIM_TOKEN_INCENTIVES),
        ("Pumpfun:CloseUserVolumeAccumulator", CLOSE_USER_VOLUME_ACCUMULATOR),
        ("Pumpfun:CollectCreatorFee", COLLECT_CREATOR_FEE),
        ("Pumpfun:Create", CREATE),
        ("Pumpfun:CreateV2", CREATE_V2),
        ("Pumpfun:DistributeCreatorFees", DISTRIBUTE_CREATOR_FEES),
        ("Pumpfun:ExtendAccount", EXTEND_ACCOUNT),
        ("Pumpfun:GetMinimumDistributableFee", GET_MINIMUM_DISTRIBUTABLE_FEE),
        ("Pumpfun:InitUserVolumeAccumulator", INIT_USER_VOLUME_ACCUMULATOR),
        ("Pumpfun:Initialize", INITIALIZE),
        ("Pumpfun:Migrate", MIGRATE),
        ("Pumpfun:MigrateBondingCurveCreator", MIGRATE_BONDING_CURVE_CREATOR),
        ("Pumpfun:Sell", SELL),
        ("Pumpfun:SetCreator", SET_CREATOR),
        ("Pumpfun:SetMayhemVirtualParams", SET_MAYHEM_VIRTUAL_PARAMS),
        ("Pumpfun:SetMetaplexCreator", SET_METAPLEX_CREATOR),
        ("Pumpfun:SetParams", SET_PARAMS),
        ("Pumpfun:SetReservedFeeRecipients", SET_RESERVED_FEE_RECIPIENTS),
        ("Pumpfun:SyncUserVolumeAccumulator", SYNC_USER_VOLUME_ACCUMULATOR),
        ("Pumpfun:ToggleCashbackEnabled", TOGGLE_CASHBACK_ENABLED),
        ("Pumpfun:ToggleCreateV2", TOGGLE_CREATE_V2),
        ("Pumpfun:ToggleMayhemMode", TOGGLE_MAYHEM_MODE),
        ("Pumpfun:UpdateGlobalAuthority", UPDATE_GLOBAL_AUTHORITY),
    ];
}

// ============================================================
// Event Discriminators (first 8 bytes of event data after
// the 8-byte Anchor event log prefix)
// ============================================================

pub mod event_discriminator {
    pub const ANCHOR_EVENT_LOG: [u8; 8] = [228, 69, 165, 46, 81, 203, 154, 29];
    pub const ADMIN_SET_CREATOR_EVENT: [u8; 8] = [64, 69, 192, 104, 29, 30, 25, 107];
    pub const ADMIN_SET_IDL_AUTHORITY_EVENT: [u8; 8] = [245, 59, 70, 34, 75, 185, 109, 92];
    pub const ADMIN_UPDATE_TOKEN_INCENTIVES_EVENT: [u8; 8] = [147, 250, 108, 120, 247, 29, 67, 222];
    pub const CLAIM_CASHBACK_EVENT: [u8; 8] = [226, 214, 246, 33, 7, 242, 147, 229];
    pub const CLAIM_TOKEN_INCENTIVES_EVENT: [u8; 8] = [79, 172, 246, 49, 205, 91, 206, 232];
    pub const CLOSE_USER_VOLUME_ACCUMULATOR_EVENT: [u8; 8] = [146, 159, 189, 172, 146, 88, 56, 244];
    pub const COLLECT_CREATOR_FEE_EVENT: [u8; 8] = [122, 2, 127, 1, 14, 191, 12, 175];
    pub const COMPLETE_EVENT: [u8; 8] = [95, 114, 97, 156, 212, 46, 152, 8];
    pub const COMPLETE_PUMP_AMM_MIGRATION_EVENT: [u8; 8] = [189, 233, 93, 185, 92, 148, 234, 148];
    pub const CREATE_EVENT: [u8; 8] = [27, 114, 169, 77, 222, 235, 99, 118];
    pub const DISTRIBUTE_CREATOR_FEES_EVENT: [u8; 8] = [165, 55, 129, 112, 4, 179, 202, 40];
    pub const EXTEND_ACCOUNT_EVENT: [u8; 8] = [97, 97, 215, 144, 93, 146, 22, 124];
    pub const INIT_USER_VOLUME_ACCUMULATOR_EVENT: [u8; 8] = [134, 36, 13, 72, 232, 101, 130, 216];
    pub const MIGRATE_BONDING_CURVE_CREATOR_EVENT: [u8; 8] = [155, 167, 104, 220, 213, 108, 243, 3];
    pub const MINIMUM_DISTRIBUTABLE_FEE_EVENT: [u8; 8] = [168, 216, 132, 239, 235, 182, 49, 52];
    pub const RESERVED_FEE_RECIPIENTS_EVENT: [u8; 8] = [43, 188, 250, 18, 221, 75, 187, 95];
    pub const SET_CREATOR_EVENT: [u8; 8] = [237, 52, 123, 37, 245, 251, 72, 210];
    pub const SET_METAPLEX_CREATOR_EVENT: [u8; 8] = [142, 203, 6, 32, 127, 105, 191, 162];
    pub const SET_PARAMS_EVENT: [u8; 8] = [223, 195, 159, 246, 62, 48, 143, 131];
    pub const SYNC_USER_VOLUME_ACCUMULATOR_EVENT: [u8; 8] = [197, 122, 167, 124, 116, 81, 91, 255];
    pub const TRADE_EVENT: [u8; 8] = [189, 219, 127, 211, 78, 230, 97, 238];
    pub const UPDATE_GLOBAL_AUTHORITY_EVENT: [u8; 8] = [182, 195, 137, 42, 35, 206, 207, 247];
    pub const UPDATE_MAYHEM_VIRTUAL_PARAMS_EVENT: [u8; 8] = [117, 123, 228, 182, 161, 168, 220, 214];
}

// ============================================================
// Instruction Args Structs (BorshDeserialize)
// Deserialize from ix.data[8..] (after 8-byte discriminator)
// Instructions with no args are omitted.
// ============================================================

/// `admin_set_creator` instruction args
#[derive(Debug, Clone, BorshDeserialize)]
pub struct AdminSetCreatorArgs {
    pub creator: Pubkey,
}

/// `admin_set_idl_authority` instruction args
#[derive(Debug, Clone, BorshDeserialize)]
pub struct AdminSetIdlAuthorityArgs {
    pub idl_authority: Pubkey,
}

/// `admin_update_token_incentives` instruction args
#[derive(Debug, Clone, BorshDeserialize)]
pub struct AdminUpdateTokenIncentivesArgs {
    pub start_time: i64,
    pub end_time: i64,
    pub seconds_in_a_day: i64,
    pub day_number: u64,
    pub pump_token_supply_per_day: u64,
}

/// `buy` instruction args
#[derive(Debug, Clone)]
pub struct BuyArgs {
    pub amount: u64,
    pub max_sol_cost: u64,
    pub track_volume: bool,
}

impl BuyArgs {
    pub fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self, std::io::Error> {
        let amount = u64::deserialize(data)?;
        let max_sol_cost = u64::deserialize(data)?;
        let track_volume = if data.is_empty() {
            false
        } else {
            OptionBool::deserialize(data)?.is_true()
        };
        Ok(Self { amount, max_sol_cost, track_volume })
    }
}

/// `buy_exact_sol_in` instruction args
#[derive(Debug, Clone)]
pub struct BuyExactSolInArgs {
    pub spendable_sol_in: u64,
    pub min_tokens_out: u64,
    pub track_volume: bool,
}

impl BuyExactSolInArgs {
    pub fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self, std::io::Error> {
        let spendable_sol_in = u64::deserialize(data)?;
        let min_tokens_out = u64::deserialize(data)?;
        let track_volume = if data.is_empty() {
            false
        } else {
            OptionBool::deserialize(data)?.is_true()
        };
        Ok(Self { spendable_sol_in, min_tokens_out, track_volume })
    }
}

/// `create` (v1) instruction args
#[derive(Debug, Clone, BorshDeserialize)]
pub struct CreateArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub creator: Pubkey,
}

/// `create_v2` instruction args
#[derive(Debug, Clone)]
pub struct CreateV2Args {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub creator: Pubkey,
    pub is_mayhem_mode: bool,
    pub is_cashback_enabled: bool,
}

impl CreateV2Args {
    pub fn deserialize_from_slice(data: &mut &[u8]) -> Result<Self, std::io::Error> {
        let name = String::deserialize(data)?;
        let symbol = String::deserialize(data)?;
        let uri = String::deserialize(data)?;
        let creator = Pubkey::deserialize(data)?;
        let is_mayhem_mode = bool::deserialize(data)?;
        let is_cashback_enabled = if data.is_empty() {
            false
        } else {
            OptionBool::deserialize(data)?.is_true()
        };
        Ok(Self { name, symbol, uri, creator, is_mayhem_mode, is_cashback_enabled })
    }
}

/// `sell` instruction args
#[derive(Debug, Clone, BorshDeserialize)]
pub struct SellArgs {
    pub amount: u64,
    pub min_sol_output: u64,
}

/// `set_creator` instruction args
#[derive(Debug, Clone, BorshDeserialize)]
pub struct SetCreatorArgs {
    pub creator: Pubkey,
}

/// `set_params` instruction args
#[derive(Debug, Clone, BorshDeserialize)]
pub struct SetParamsArgs {
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub token_total_supply: u64,
    pub fee_basis_points: u64,
    pub withdraw_authority: Pubkey,
    pub enable_migrate: bool,
    pub pool_migration_fee: u64,
    pub creator_fee_basis_points: u64,
    pub set_creator_authority: Pubkey,
    pub admin_set_creator_authority: Pubkey,
}

/// `set_reserved_fee_recipients` instruction args
#[derive(Debug, Clone, BorshDeserialize)]
pub struct SetReservedFeeRecipientsArgs {
    pub whitelist_pda: Pubkey,
}

/// `toggle_cashback_enabled` instruction args
#[derive(Debug, Clone, BorshDeserialize)]
pub struct ToggleCashbackEnabledArgs {
    pub enabled: bool,
}

/// `toggle_create_v2` instruction args
#[derive(Debug, Clone, BorshDeserialize)]
pub struct ToggleCreateV2Args {
    pub enabled: bool,
}

/// `toggle_mayhem_mode` instruction args
#[derive(Debug, Clone, BorshDeserialize)]
pub struct ToggleMayhemModeArgs {
    pub enabled: bool,
}

// ============================================================
// Instruction Account Indices
// Use: account_keys[info.accounts[INDEX] as usize]
// ============================================================

pub mod ix_accounts {
    /// `admin_set_creator` accounts (6 accounts)
    pub mod admin_set_creator {
        pub const ADMIN_SET_CREATOR_AUTHORITY: usize = 0;
        pub const GLOBAL: usize = 1;
        pub const MINT: usize = 2;
        pub const BONDING_CURVE: usize = 3;
        pub const EVENT_AUTHORITY: usize = 4;
        pub const PROGRAM: usize = 5;
    }

    /// `admin_set_idl_authority` accounts (7 accounts)
    pub mod admin_set_idl_authority {
        pub const AUTHORITY: usize = 0;
        pub const GLOBAL: usize = 1;
        pub const IDL_ACCOUNT: usize = 2;
        pub const SYSTEM_PROGRAM: usize = 3;
        pub const PROGRAM_SIGNER: usize = 4;
        pub const EVENT_AUTHORITY: usize = 5;
        pub const PROGRAM: usize = 6;
    }

    /// `admin_update_token_incentives` accounts (10 accounts)
    pub mod admin_update_token_incentives {
        pub const AUTHORITY: usize = 0;
        pub const GLOBAL: usize = 1;
        pub const GLOBAL_VOLUME_ACCUMULATOR: usize = 2;
        pub const MINT: usize = 3;
        pub const GLOBAL_INCENTIVE_TOKEN_ACCOUNT: usize = 4;
        pub const ASSOCIATED_TOKEN_PROGRAM: usize = 5;
        pub const SYSTEM_PROGRAM: usize = 6;
        pub const TOKEN_PROGRAM: usize = 7;
        pub const EVENT_AUTHORITY: usize = 8;
        pub const PROGRAM: usize = 9;
    }

    /// `buy` accounts (16 accounts)
    pub mod buy {
        pub const GLOBAL: usize = 0;
        pub const FEE_RECIPIENT: usize = 1;
        pub const MINT: usize = 2;
        pub const BONDING_CURVE: usize = 3;
        pub const ASSOCIATED_BONDING_CURVE: usize = 4;
        pub const ASSOCIATED_USER: usize = 5;
        pub const USER: usize = 6;
        pub const SYSTEM_PROGRAM: usize = 7;
        pub const TOKEN_PROGRAM: usize = 8;
        pub const CREATOR_VAULT: usize = 9;
        pub const EVENT_AUTHORITY: usize = 10;
        pub const PROGRAM: usize = 11;
        pub const GLOBAL_VOLUME_ACCUMULATOR: usize = 12;
        pub const USER_VOLUME_ACCUMULATOR: usize = 13;
        pub const FEE_CONFIG: usize = 14;
        pub const FEE_PROGRAM: usize = 15;
    }

    /// `buy_exact_sol_in` accounts (16 accounts) - same layout as `buy`
    pub mod buy_exact_sol_in {
        pub const GLOBAL: usize = 0;
        pub const FEE_RECIPIENT: usize = 1;
        pub const MINT: usize = 2;
        pub const BONDING_CURVE: usize = 3;
        pub const ASSOCIATED_BONDING_CURVE: usize = 4;
        pub const ASSOCIATED_USER: usize = 5;
        pub const USER: usize = 6;
        pub const SYSTEM_PROGRAM: usize = 7;
        pub const TOKEN_PROGRAM: usize = 8;
        pub const CREATOR_VAULT: usize = 9;
        pub const EVENT_AUTHORITY: usize = 10;
        pub const PROGRAM: usize = 11;
        pub const GLOBAL_VOLUME_ACCUMULATOR: usize = 12;
        pub const USER_VOLUME_ACCUMULATOR: usize = 13;
        pub const FEE_CONFIG: usize = 14;
        pub const FEE_PROGRAM: usize = 15;
    }

    /// `claim_cashback` accounts (5 accounts)
    pub mod claim_cashback {
        pub const USER: usize = 0;
        pub const USER_VOLUME_ACCUMULATOR: usize = 1;
        pub const SYSTEM_PROGRAM: usize = 2;
        pub const EVENT_AUTHORITY: usize = 3;
        pub const PROGRAM: usize = 4;
    }

    /// `claim_token_incentives` accounts (12 accounts)
    pub mod claim_token_incentives {
        pub const USER: usize = 0;
        pub const USER_ATA: usize = 1;
        pub const GLOBAL_VOLUME_ACCUMULATOR: usize = 2;
        pub const GLOBAL_INCENTIVE_TOKEN_ACCOUNT: usize = 3;
        pub const USER_VOLUME_ACCUMULATOR: usize = 4;
        pub const MINT: usize = 5;
        pub const TOKEN_PROGRAM: usize = 6;
        pub const SYSTEM_PROGRAM: usize = 7;
        pub const ASSOCIATED_TOKEN_PROGRAM: usize = 8;
        pub const EVENT_AUTHORITY: usize = 9;
        pub const PROGRAM: usize = 10;
        pub const PAYER: usize = 11;
    }

    /// `close_user_volume_accumulator` accounts (4 accounts)
    pub mod close_user_volume_accumulator {
        pub const USER: usize = 0;
        pub const USER_VOLUME_ACCUMULATOR: usize = 1;
        pub const EVENT_AUTHORITY: usize = 2;
        pub const PROGRAM: usize = 3;
    }

    /// `collect_creator_fee` accounts (5 accounts)
    pub mod collect_creator_fee {
        pub const CREATOR: usize = 0;
        pub const CREATOR_VAULT: usize = 1;
        pub const SYSTEM_PROGRAM: usize = 2;
        pub const EVENT_AUTHORITY: usize = 3;
        pub const PROGRAM: usize = 4;
    }

    /// `create` (v1) accounts (14 accounts)
    pub mod create {
        pub const MINT: usize = 0;
        pub const MINT_AUTHORITY: usize = 1;
        pub const BONDING_CURVE: usize = 2;
        pub const ASSOCIATED_BONDING_CURVE: usize = 3;
        pub const GLOBAL: usize = 4;
        pub const MPL_TOKEN_METADATA: usize = 5;
        pub const METADATA: usize = 6;
        pub const USER: usize = 7;
        pub const SYSTEM_PROGRAM: usize = 8;
        pub const TOKEN_PROGRAM: usize = 9;
        pub const ASSOCIATED_TOKEN_PROGRAM: usize = 10;
        pub const RENT: usize = 11;
        pub const EVENT_AUTHORITY: usize = 12;
        pub const PROGRAM: usize = 13;
    }

    /// `create_v2` accounts (16 accounts)
    pub mod create_v2 {
        pub const MINT: usize = 0;
        pub const MINT_AUTHORITY: usize = 1;
        pub const BONDING_CURVE: usize = 2;
        pub const ASSOCIATED_BONDING_CURVE: usize = 3;
        pub const GLOBAL: usize = 4;
        pub const USER: usize = 5;
        pub const SYSTEM_PROGRAM: usize = 6;
        pub const TOKEN_PROGRAM: usize = 7;
        pub const ASSOCIATED_TOKEN_PROGRAM: usize = 8;
        pub const MAYHEM_PROGRAM_ID: usize = 9;
        pub const GLOBAL_PARAMS: usize = 10;
        pub const SOL_VAULT: usize = 11;
        pub const MAYHEM_STATE: usize = 12;
        pub const MAYHEM_TOKEN_VAULT: usize = 13;
        pub const EVENT_AUTHORITY: usize = 14;
        pub const PROGRAM: usize = 15;
    }

    /// `distribute_creator_fees` accounts (7 accounts)
    pub mod distribute_creator_fees {
        pub const MINT: usize = 0;
        pub const BONDING_CURVE: usize = 1;
        pub const SHARING_CONFIG: usize = 2;
        pub const CREATOR_VAULT: usize = 3;
        pub const SYSTEM_PROGRAM: usize = 4;
        pub const EVENT_AUTHORITY: usize = 5;
        pub const PROGRAM: usize = 6;
    }

    /// `extend_account` accounts (5 accounts)
    pub mod extend_account {
        pub const ACCOUNT: usize = 0;
        pub const USER: usize = 1;
        pub const SYSTEM_PROGRAM: usize = 2;
        pub const EVENT_AUTHORITY: usize = 3;
        pub const PROGRAM: usize = 4;
    }

    /// `get_minimum_distributable_fee` accounts (4 accounts)
    pub mod get_minimum_distributable_fee {
        pub const MINT: usize = 0;
        pub const BONDING_CURVE: usize = 1;
        pub const SHARING_CONFIG: usize = 2;
        pub const CREATOR_VAULT: usize = 3;
    }

    /// `init_user_volume_accumulator` accounts (6 accounts)
    pub mod init_user_volume_accumulator {
        pub const PAYER: usize = 0;
        pub const USER: usize = 1;
        pub const USER_VOLUME_ACCUMULATOR: usize = 2;
        pub const SYSTEM_PROGRAM: usize = 3;
        pub const EVENT_AUTHORITY: usize = 4;
        pub const PROGRAM: usize = 5;
    }

    /// `initialize` accounts (3 accounts)
    pub mod initialize {
        pub const GLOBAL: usize = 0;
        pub const USER: usize = 1;
        pub const SYSTEM_PROGRAM: usize = 2;
    }

    /// `migrate` accounts (24 accounts)
    pub mod migrate {
        pub const GLOBAL: usize = 0;
        pub const WITHDRAW_AUTHORITY: usize = 1;
        pub const MINT: usize = 2;
        pub const BONDING_CURVE: usize = 3;
        pub const ASSOCIATED_BONDING_CURVE: usize = 4;
        pub const USER: usize = 5;
        pub const SYSTEM_PROGRAM: usize = 6;
        pub const TOKEN_PROGRAM: usize = 7;
        pub const PUMP_AMM: usize = 8;
        pub const POOL: usize = 9;
        pub const POOL_AUTHORITY: usize = 10;
        pub const POOL_AUTHORITY_MINT_ACCOUNT: usize = 11;
        pub const POOL_AUTHORITY_WSOL_ACCOUNT: usize = 12;
        pub const AMM_GLOBAL_CONFIG: usize = 13;
        pub const WSOL_MINT: usize = 14;
        pub const LP_MINT: usize = 15;
        pub const USER_POOL_TOKEN_ACCOUNT: usize = 16;
        pub const POOL_BASE_TOKEN_ACCOUNT: usize = 17;
        pub const POOL_QUOTE_TOKEN_ACCOUNT: usize = 18;
        pub const TOKEN_2022_PROGRAM: usize = 19;
        pub const ASSOCIATED_TOKEN_PROGRAM: usize = 20;
        pub const PUMP_AMM_EVENT_AUTHORITY: usize = 21;
        pub const EVENT_AUTHORITY: usize = 22;
        pub const PROGRAM: usize = 23;
    }

    /// `migrate_bonding_curve_creator` accounts (5 accounts)
    pub mod migrate_bonding_curve_creator {
        pub const MINT: usize = 0;
        pub const BONDING_CURVE: usize = 1;
        pub const SHARING_CONFIG: usize = 2;
        pub const EVENT_AUTHORITY: usize = 3;
        pub const PROGRAM: usize = 4;
    }

    /// `sell` accounts (14 accounts)
    pub mod sell {
        pub const GLOBAL: usize = 0;
        pub const FEE_RECIPIENT: usize = 1;
        pub const MINT: usize = 2;
        pub const BONDING_CURVE: usize = 3;
        pub const ASSOCIATED_BONDING_CURVE: usize = 4;
        pub const ASSOCIATED_USER: usize = 5;
        pub const USER: usize = 6;
        pub const SYSTEM_PROGRAM: usize = 7;
        pub const CREATOR_VAULT: usize = 8;
        pub const TOKEN_PROGRAM: usize = 9;
        pub const EVENT_AUTHORITY: usize = 10;
        pub const PROGRAM: usize = 11;
        pub const FEE_CONFIG: usize = 12;
        pub const FEE_PROGRAM: usize = 13;
    }

    /// `set_creator` accounts (7 accounts)
    pub mod set_creator {
        pub const SET_CREATOR_AUTHORITY: usize = 0;
        pub const GLOBAL: usize = 1;
        pub const MINT: usize = 2;
        pub const METADATA: usize = 3;
        pub const BONDING_CURVE: usize = 4;
        pub const EVENT_AUTHORITY: usize = 5;
        pub const PROGRAM: usize = 6;
    }

    /// `set_mayhem_virtual_params` accounts (8 accounts)
    pub mod set_mayhem_virtual_params {
        pub const SOL_VAULT_AUTHORITY: usize = 0;
        pub const MAYHEM_TOKEN_VAULT: usize = 1;
        pub const MINT: usize = 2;
        pub const GLOBAL: usize = 3;
        pub const BONDING_CURVE: usize = 4;
        pub const TOKEN_PROGRAM: usize = 5;
        pub const EVENT_AUTHORITY: usize = 6;
        pub const PROGRAM: usize = 7;
    }

    /// `set_metaplex_creator` accounts (5 accounts)
    pub mod set_metaplex_creator {
        pub const MINT: usize = 0;
        pub const METADATA: usize = 1;
        pub const BONDING_CURVE: usize = 2;
        pub const EVENT_AUTHORITY: usize = 3;
        pub const PROGRAM: usize = 4;
    }

    /// `set_params` accounts (4 accounts)
    pub mod set_params {
        pub const GLOBAL: usize = 0;
        pub const AUTHORITY: usize = 1;
        pub const EVENT_AUTHORITY: usize = 2;
        pub const PROGRAM: usize = 3;
    }

    /// `set_reserved_fee_recipients` accounts (4 accounts)
    pub mod set_reserved_fee_recipients {
        pub const GLOBAL: usize = 0;
        pub const AUTHORITY: usize = 1;
        pub const EVENT_AUTHORITY: usize = 2;
        pub const PROGRAM: usize = 3;
    }

    /// `sync_user_volume_accumulator` accounts (5 accounts)
    pub mod sync_user_volume_accumulator {
        pub const USER: usize = 0;
        pub const GLOBAL_VOLUME_ACCUMULATOR: usize = 1;
        pub const USER_VOLUME_ACCUMULATOR: usize = 2;
        pub const EVENT_AUTHORITY: usize = 3;
        pub const PROGRAM: usize = 4;
    }

    /// `toggle_cashback_enabled` accounts (4 accounts)
    pub mod toggle_cashback_enabled {
        pub const GLOBAL: usize = 0;
        pub const AUTHORITY: usize = 1;
        pub const EVENT_AUTHORITY: usize = 2;
        pub const PROGRAM: usize = 3;
    }

    /// `toggle_create_v2` accounts (4 accounts)
    pub mod toggle_create_v2 {
        pub const GLOBAL: usize = 0;
        pub const AUTHORITY: usize = 1;
        pub const EVENT_AUTHORITY: usize = 2;
        pub const PROGRAM: usize = 3;
    }

    /// `toggle_mayhem_mode` accounts (4 accounts)
    pub mod toggle_mayhem_mode {
        pub const GLOBAL: usize = 0;
        pub const AUTHORITY: usize = 1;
        pub const EVENT_AUTHORITY: usize = 2;
        pub const PROGRAM: usize = 3;
    }

    /// `update_global_authority` accounts (5 accounts)
    pub mod update_global_authority {
        pub const GLOBAL: usize = 0;
        pub const AUTHORITY: usize = 1;
        pub const NEW_AUTHORITY: usize = 2;
        pub const EVENT_AUTHORITY: usize = 3;
        pub const PROGRAM: usize = 4;
    }
}

// ============================================================
// Event Data Structs (BorshDeserialize)
// Deserialize from ix.data[16..] (8-byte Anchor log prefix +
// 8-byte event discriminator)
// ============================================================

#[derive(Debug, Clone, BorshDeserialize)]
pub struct IdlCreateEvent {
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

#[derive(Debug, Clone, BorshDeserialize)]
pub struct IdlTradeEvent {
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
    pub track_volume: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,
    pub ix_name: String,
    pub mayhem_mode: bool,
    pub cashback_fee_basis_points: u64,
    pub cashback: u64,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct IdlCompleteEvent {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub timestamp: i64,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct IdlCompletePumpAmmMigrationEvent {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub mint_amount: u64,
    pub sol_amount: u64,
    pub pool_migration_fee: u64,
    pub bonding_curve: Pubkey,
    pub timestamp: i64,
    pub pool: Pubkey,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct IdlSetParamsEvent {
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub final_real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub fee_basis_points: u64,
    pub withdraw_authority: Pubkey,
    pub enable_migrate: bool,
    pub pool_migration_fee: u64,
    pub creator_fee_basis_points: u64,
    pub fee_recipients: [Pubkey; 8],
    pub timestamp: i64,
    pub set_creator_authority: Pubkey,
    pub admin_set_creator_authority: Pubkey,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct IdlCollectCreatorFeeEvent {
    pub timestamp: i64,
    pub creator: Pubkey,
    pub creator_fee: u64,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct IdlClaimCashbackEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub total_claimed: u64,
    pub total_cashback_earned: u64,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct IdlClaimTokenIncentivesEvent {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
}

// ============================================================
// Utility: Identify instruction by discriminator
// ============================================================

pub fn identify_instruction(data: &[u8]) -> Option<&'static str> {
    if data.len() < 8 {
        return None;
    }
    let disc: [u8; 8] = data[..8].try_into().ok()?;
    for &(name, ref d) in ix_discriminator::ALL {
        if disc == *d {
            return Some(name);
        }
    }
    None
}

/// Identify an Anchor event by its discriminator (data[8..16] after the log prefix)
pub fn identify_event(data: &[u8]) -> Option<&'static str> {
    if data.len() < 16 {
        return None;
    }
    let disc: [u8; 8] = data[8..16].try_into().ok()?;
    match disc {
        event_discriminator::CREATE_EVENT => Some("CreateEvent"),
        event_discriminator::TRADE_EVENT => Some("TradeEvent"),
        event_discriminator::COMPLETE_EVENT => Some("CompleteEvent"),
        event_discriminator::COMPLETE_PUMP_AMM_MIGRATION_EVENT => {
            Some("CompletePumpAmmMigrationEvent")
        }
        event_discriminator::SET_PARAMS_EVENT => Some("SetParamsEvent"),
        event_discriminator::COLLECT_CREATOR_FEE_EVENT => Some("CollectCreatorFeeEvent"),
        event_discriminator::CLAIM_CASHBACK_EVENT => Some("ClaimCashbackEvent"),
        event_discriminator::CLAIM_TOKEN_INCENTIVES_EVENT => Some("ClaimTokenIncentivesEvent"),
        event_discriminator::ADMIN_SET_CREATOR_EVENT => Some("AdminSetCreatorEvent"),
        event_discriminator::ADMIN_SET_IDL_AUTHORITY_EVENT => Some("AdminSetIdlAuthorityEvent"),
        event_discriminator::ADMIN_UPDATE_TOKEN_INCENTIVES_EVENT => {
            Some("AdminUpdateTokenIncentivesEvent")
        }
        event_discriminator::CLOSE_USER_VOLUME_ACCUMULATOR_EVENT => {
            Some("CloseUserVolumeAccumulatorEvent")
        }
        event_discriminator::DISTRIBUTE_CREATOR_FEES_EVENT => {
            Some("DistributeCreatorFeesEvent")
        }
        event_discriminator::EXTEND_ACCOUNT_EVENT => Some("ExtendAccountEvent"),
        event_discriminator::INIT_USER_VOLUME_ACCUMULATOR_EVENT => {
            Some("InitUserVolumeAccumulatorEvent")
        }
        event_discriminator::MIGRATE_BONDING_CURVE_CREATOR_EVENT => {
            Some("MigrateBondingCurveCreatorEvent")
        }
        event_discriminator::MINIMUM_DISTRIBUTABLE_FEE_EVENT => {
            Some("MinimumDistributableFeeEvent")
        }
        event_discriminator::RESERVED_FEE_RECIPIENTS_EVENT => {
            Some("ReservedFeeRecipientsEvent")
        }
        event_discriminator::SET_CREATOR_EVENT => Some("SetCreatorEvent"),
        event_discriminator::SET_METAPLEX_CREATOR_EVENT => Some("SetMetaplexCreatorEvent"),
        event_discriminator::SYNC_USER_VOLUME_ACCUMULATOR_EVENT => {
            Some("SyncUserVolumeAccumulatorEvent")
        }
        event_discriminator::UPDATE_GLOBAL_AUTHORITY_EVENT => {
            Some("UpdateGlobalAuthorityEvent")
        }
        event_discriminator::UPDATE_MAYHEM_VIRTUAL_PARAMS_EVENT => {
            Some("UpdateMayhemVirtualParamsEvent")
        }
        _ => None,
    }
}
