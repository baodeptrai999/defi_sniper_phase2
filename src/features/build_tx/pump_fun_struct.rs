use borsh::BorshDeserialize;
use solana_sdk::system_instruction;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_sdk_ids::system_program;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;

use crate::*;

#[derive(Debug, Clone, BorshDeserialize)]
pub struct PumpFunSwapAccounts {
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
    pub global_volume_accumulator: Option<Pubkey>,
    pub user_volume_accumulator: Option<Pubkey>,
    pub fee_config: Pubkey,
    pub fee_program: Pubkey,
}

impl PumpFunSwapAccounts {
    pub fn from_mint(
        mint_instruction_account: &MintInstructionAccounts,
        mint_event: &MintEvent,
    ) -> Self {
        let associated_user = get_associated_token_address_with_program_id(
            &SIGNER_PUBKEY,
            &mint_instruction_account.mint,
            &spl_token::ID,
        );
        let (creator_vault, _) = Pubkey::find_program_address(
            &[CREATOR_VAULT_SEED, &mint_event.creator.as_ref()],
            &PUMPFUN_PROGRAM_ID,
        );

        Self {
            global: mint_instruction_account.global,
            fee_recipient: PUMPFUN_FEE_RECIPIENT,
            mint: mint_instruction_account.mint,
            bonding_curve: mint_instruction_account.bonding_curve,
            associated_bonding_curve: mint_instruction_account.associated_bonding_curve,
            associated_user: associated_user,
            user: *SIGNER_PUBKEY,
            system_program: system_program::ID,
            token_program: mint_instruction_account.token_program,
            creator_vault: creator_vault,
            event_authority: mint_instruction_account.event_authority,
            program: PUMPFUN_PROGRAM_ID,
            global_volume_accumulator: Some(PUMPFUN_GLOBAL_VOLUME_ACCUMULATOR),
            user_volume_accumulator: Some(*PUMPFUN_USER_VOLUME_ACCUMULATOR),
            fee_config: PUMPFUN_FEE_CONFIG,
            fee_program: PUMPFUN_FEE_PROGRAM,
        }
    }

    pub fn from_target_buy(buy_instruction_accounts: BuyInstructionAccounts) -> Self {
        let associated_user = get_associated_token_address_with_program_id(
            &SIGNER_PUBKEY,
            &buy_instruction_accounts.mint,
            &spl_token::ID,
        );
        Self {
            global: buy_instruction_accounts.global,
            fee_recipient: buy_instruction_accounts.fee_recipient,
            mint: buy_instruction_accounts.mint,
            bonding_curve: buy_instruction_accounts.bonding_curve,
            associated_bonding_curve: buy_instruction_accounts.associated_bonding_curve,
            associated_user: associated_user,
            user: *SIGNER_PUBKEY,
            system_program: buy_instruction_accounts.system_program,
            token_program: buy_instruction_accounts.token_program,
            creator_vault: buy_instruction_accounts.creator_vault,
            event_authority: buy_instruction_accounts.event_authority,
            program: buy_instruction_accounts.program,
            global_volume_accumulator: Some(PUMPFUN_GLOBAL_VOLUME_ACCUMULATOR),
            user_volume_accumulator: Some(*PUMPFUN_USER_VOLUME_ACCUMULATOR),
            fee_config: buy_instruction_accounts.fee_config,
            fee_program: buy_instruction_accounts.fee_program,
        }
    }

    pub fn get_create_ata_idempotent_ix(&self) -> Instruction {
        let create_token_ata = create_associated_token_account_idempotent(
            &*SIGNER_PUBKEY,
            &*SIGNER_PUBKEY,
            &self.mint,
            &self.token_program,
        );
        create_token_ata
    }

    pub fn get_sol_ix(&self) -> Instruction {
        let slippage_calculated_buy_amount = *BUY_AMOUNT_SOL as f64 * 10f64.powi(9) * *SLIPPAGE;
        let turncated_slippage_calculated_buy_amount =
            slippage_calculated_buy_amount.trunc() as u64;
        let transfer_ix = system_instruction::transfer(
            &*SIGNER_PUBKEY,
            &self.bonding_curve,
            turncated_slippage_calculated_buy_amount,
        );
        transfer_ix
    }

    pub fn get_half_copy_sol_ix(&self, amount: f64) -> Instruction {
        let slippage_calculated_buy_amount = amount * *SLIPPAGE;
        let turncated_slippage_calculated_buy_amount =
            slippage_calculated_buy_amount.trunc() as u64;
        let transfer_ix = system_instruction::transfer(
            &*SIGNER_PUBKEY,
            &self.bonding_curve,
            turncated_slippage_calculated_buy_amount,
        );
        transfer_ix
    }

    pub fn get_buy_ix(&mut self, token_price: f64) -> Instruction {
        let mut data = Vec::new();

        let base_out: f64 = (*BUY_AMOUNT_SOL / token_price) * 10f64.powi(6);
        let truncated_base_out: u64 = base_out.trunc() as u64;
        let max_quote_in: f64 = *BUY_AMOUNT_SOL as f64 * 10f64.powi(9) * *SLIPPAGE;
        let turncated_max_quote_in: u64 = max_quote_in.trunc() as u64;

        data.extend_from_slice(&PUMP_FUN_BUY_DISCRIMINATOR);
        data.extend_from_slice(&truncated_base_out.to_le_bytes());
        data.extend_from_slice(&turncated_max_quote_in.to_le_bytes());

        let accounts = vec![
            AccountMeta::new_readonly(self.global, false), // #1 - Global
            AccountMeta::new(self.fee_recipient, false),   // #2 - Fee Recipient
            AccountMeta::new_readonly(self.mint, false),   // #3 - Mint
            AccountMeta::new(self.bonding_curve, false),   // #4 - BondingCurve
            AccountMeta::new(self.associated_bonding_curve, false), // #5 - Quote Mint (TSFart)
            AccountMeta::new(self.associated_user, false), // #6 - Associated User
            AccountMeta::new(self.user, true),             // #7 - User
            AccountMeta::new_readonly(self.system_program, false), // #8 - System Program
            AccountMeta::new_readonly(self.token_program, false), // #9 - Token Program
            AccountMeta::new(self.creator_vault, false),   // #10 - Creator Vault
            AccountMeta::new_readonly(self.event_authority, false), // #11 - Event authority
            AccountMeta::new_readonly(self.program, false), // #12 - Pump.fun program
            AccountMeta::new(self.global_volume_accumulator.unwrap(), false), // #13 - Global volume accumulator
            AccountMeta::new(self.user_volume_accumulator.unwrap(), false), // #14 - User volume accumulator
            AccountMeta::new_readonly(self.fee_config, false),              // #15 - Fee Config
            AccountMeta::new_readonly(self.fee_program, false),             //#16 - Fee Program
        ];

        Instruction {
            program_id: PUMPFUN_PROGRAM_ID,
            accounts,
            data,
        }
    }

    pub fn get_half_copy_buy_ix(&mut self, sol_amount: f64, token_price: f64) -> Instruction {
        let mut data = Vec::new();

        let base_out: f64 = ((sol_amount / 10f64.powi(9)) / token_price) * 10f64.powi(6);
        let truncated_base_out: u64 = base_out.trunc() as u64;
        let max_quote_in: f64 = sol_amount * *SLIPPAGE;
        let turncated_max_quote_in: u64 = max_quote_in.trunc() as u64;

        data.extend_from_slice(&PUMP_FUN_BUY_DISCRIMINATOR);
        data.extend_from_slice(&truncated_base_out.to_le_bytes());
        data.extend_from_slice(&turncated_max_quote_in.to_le_bytes());

        let accounts = vec![
            AccountMeta::new_readonly(self.global, false), // #1 - Global
            AccountMeta::new(self.fee_recipient, false),   // #2 - Fee Recipient
            AccountMeta::new_readonly(self.mint, false),   // #3 - Mint
            AccountMeta::new(self.bonding_curve, false),   // #4 - BondingCurve
            AccountMeta::new(self.associated_bonding_curve, false), // #5 - Quote Mint (TSFart)
            AccountMeta::new(self.associated_user, false), // #6 - Associated User
            AccountMeta::new(self.user, true),             // #7 - User
            AccountMeta::new_readonly(self.system_program, false), // #8 - System Program
            AccountMeta::new_readonly(self.token_program, false), // #9 - Token Program
            AccountMeta::new(self.creator_vault, false),   // #10 - Creator Vault
            AccountMeta::new_readonly(self.event_authority, false), // #11 - Event authority
            AccountMeta::new_readonly(self.program, false), // #12 - Pump.fun program
            AccountMeta::new(self.global_volume_accumulator.unwrap(), false), // #13 - Global volume accumulator
            AccountMeta::new(self.user_volume_accumulator.unwrap(), false), // #14 - User volume accumulator
            AccountMeta::new_readonly(self.fee_config, false),              // #15 - Fee Config
            AccountMeta::new_readonly(self.fee_program, false),             //#16 - Fee Program
        ];

        Instruction {
            program_id: PUMPFUN_PROGRAM_ID,
            accounts,
            data,
        }
    }

    pub fn get_sell_ix(&mut self, sell_amount: u64) -> Instruction {
        let mut data = Vec::new();

        let min_sol_out: u64 = 1;

        data.extend_from_slice(&PUMP_FUN_SELL_DISCRIMINATOR);
        data.extend_from_slice(&sell_amount.to_le_bytes());
        data.extend_from_slice(&min_sol_out.to_le_bytes());

        let accounts = vec![
            AccountMeta::new_readonly(self.global, false), // #1 - Global
            AccountMeta::new(self.fee_recipient, false),   // #2 - Fee Recipient
            AccountMeta::new_readonly(self.mint, false),   // #3 - Mint
            AccountMeta::new(self.bonding_curve, false),   // #4 - BondingCurve
            AccountMeta::new(self.associated_bonding_curve, false), // #5 - Quote Mint (TSFart)
            AccountMeta::new(self.associated_user, false), // #6 - Associated User
            AccountMeta::new(self.user, true),             // #7 - User
            AccountMeta::new_readonly(self.system_program, false), // #8 - System Program
            AccountMeta::new(self.creator_vault, false),   // #9 - Creator Vault
            AccountMeta::new_readonly(self.token_program, false), // #10 - Token Program
            AccountMeta::new_readonly(self.event_authority, false), // #11 - Event authority
            AccountMeta::new_readonly(self.program, false), // #12 - Pump.fun program
            AccountMeta::new_readonly(self.fee_config, false), // #13 - Fee Config
            AccountMeta::new_readonly(self.fee_program, false), //#14 - Fee Program
        ];

        Instruction {
            program_id: PUMPFUN_PROGRAM_ID,
            accounts,
            data,
        }
    }
}
