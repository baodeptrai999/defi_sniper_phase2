use crate::*;
use borsh::BorshDeserialize;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_sdk_ids::system_program;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;

#[derive(Debug, Clone, BorshDeserialize)]
pub struct PumpfunStruct {
    pub global: Pubkey,
    pub fee_recipient: Pubkey,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub associated_bonding_curve: Pubkey,
    pub user: Pubkey,
    pub associated_user: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
    pub event_authority: Pubkey,
    pub program: Pubkey,
    pub fee_config: Pubkey,
    pub fee_program: Pubkey,
    pub global_volume_accumulator: Pubkey,
    pub user_volume_accumulator: Pubkey,
    pub bonding_curve_v2_pda: Pubkey,
}

impl PumpfunStruct {
    pub fn from_mint(
        mint_instruction_account: &MintInstructionAccounts,
        mint_event: &MintEvent,
    ) -> Self {
        let fee_recipient = if mint_event.is_mayhem_mode {
            MAYHEM_PROTOCOL_FEE_RECIPIENT
        } else {
            PUMPFUN_FEE_RECIPIENT
        };

        let (bonding_curve_v2_pda, _) = Pubkey::find_program_address(
            &[
                PUMPFUN_BONDING_CURVE_V2_SEED,
                &mint_instruction_account.mint.as_ref(),
            ],
            &PUMPFUN_PROGRAM_ID,
        );

        let associated_user = get_associated_token_address_with_program_id(
            &*SIGNER_PUBKEY,
            &mint_event.mint,
            &mint_event.token_program,
        );

        Self {
            global: PUMPFUN_GLOBAL,
            fee_recipient: fee_recipient,
            mint: mint_instruction_account.mint,
            bonding_curve: mint_instruction_account.bonding_curve,
            associated_bonding_curve: mint_instruction_account.associated_bonding_curve,
            user: *SIGNER_PUBKEY,
            associated_user: associated_user,
            system_program: system_program::ID,
            token_program: mint_instruction_account.token_program,
            event_authority: mint_instruction_account.event_authority,
            program: PUMPFUN_PROGRAM_ID,
            fee_config: PUMPFUN_FEE_CONFIG,
            fee_program: PUMPFUN_FEE_PROGRAM,
            global_volume_accumulator: PUMPFUN_GLOBAL_VOLUME_ACCUMULATOR,
            user_volume_accumulator: get_pumpfun_user_volume_accumulator(*SIGNER_PUBKEY),
            bonding_curve_v2_pda: bonding_curve_v2_pda,
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

    pub fn get_close_ata_ix(&self) -> Instruction {
        let accounts = vec![
            AccountMeta::new(self.associated_user, false),
            AccountMeta::new(*SIGNER_PUBKEY, true),
            AccountMeta::new(*SIGNER_PUBKEY, true),
        ];
        let data = vec![9];

        Instruction {
            program_id: self.token_program,
            accounts,
            data,
        }
    }

    pub fn get_buy_ix(&mut self, updated_token_creator: Pubkey, buy_amount_sol: f64) -> Instruction {
        //get custom accounts
        let (updated_creator_vault, _) = Pubkey::find_program_address(
            &[PUMPFUN_CREATOR_VAULT_SEED, &updated_token_creator.as_ref()],
            &PUMPFUN_PROGRAM_ID,
        );

        //build instruction data — buy_exact_sol_in
        let mut data = Vec::new();

        let spendable_sol_in: u64 = (buy_amount_sol * 10f64.powi(9)).trunc() as u64;
        let min_tokens_out: u64 = 1;

        data.extend_from_slice(&ix_discriminator::BUY_EXACT_SOL_IN);
        data.extend_from_slice(&spendable_sol_in.to_le_bytes());
        data.extend_from_slice(&min_tokens_out.to_le_bytes());

        let accounts = vec![
            AccountMeta::new_readonly(self.global, false), // #1 - Global
            AccountMeta::new(self.fee_recipient, false),   // #2 - Fee Recipient
            AccountMeta::new_readonly(self.mint, false),   // #3 - Mint
            AccountMeta::new(self.bonding_curve, false),   // #4 - BondingCurve
            AccountMeta::new(self.associated_bonding_curve, false), // #5 - Quote Mint (TSFart)
            AccountMeta::new(self.associated_user, false), // #6 - Associated User
            AccountMeta::new(*SIGNER_PUBKEY, true),        // #7 - User
            AccountMeta::new_readonly(self.system_program, false), // #8 - System Program
            AccountMeta::new_readonly(self.token_program, false), // #9 - Token Program
            AccountMeta::new(updated_creator_vault, false), // #10 - Creator Vault
            AccountMeta::new_readonly(self.event_authority, false), // #11 - Event authority
            AccountMeta::new_readonly(self.program, false), // #12 - Pump.fun program
            AccountMeta::new(PUMPFUN_GLOBAL_VOLUME_ACCUMULATOR, false), // #13 - Global volume accumulator
            AccountMeta::new(self.user_volume_accumulator, false), // #14 - User volume accumulator
            AccountMeta::new_readonly(self.fee_config, false),     // #15 - Fee Config
            AccountMeta::new_readonly(self.fee_program, false),    //#16 - Fee Program
            AccountMeta::new_readonly(self.bonding_curve_v2_pda, false), //#17 - Bonding Curve V2 PDA
        ];

        Instruction {
            program_id: PUMPFUN_PROGRAM_ID,
            accounts,
            data,
        }
    }

    pub fn get_sell_ix(
        &mut self,
        updated_token_creator: Pubkey,
        sell_amount: u64,
        is_cashback_enabled: bool,
    ) -> Instruction {
        //get custom accounts
        let (updated_creator_vault, _) = Pubkey::find_program_address(
            &[PUMPFUN_CREATOR_VAULT_SEED, &updated_token_creator.as_ref()],
            &PUMPFUN_PROGRAM_ID,
        );

        //build instruction data
        let mut data = Vec::new();

        let min_sol_out: u64 = 1;

        data.extend_from_slice(&ix_discriminator::SELL);
        data.extend_from_slice(&sell_amount.to_le_bytes());
        data.extend_from_slice(&min_sol_out.to_le_bytes());

        let accounts = if !is_cashback_enabled {
            vec![
                AccountMeta::new_readonly(self.global, false), // #1 - Global
                AccountMeta::new(self.fee_recipient, false),   // #2 - Fee Recipient
                AccountMeta::new_readonly(self.mint, false),   // #3 - Mint
                AccountMeta::new(self.bonding_curve, false),   // #4 - BondingCurve
                AccountMeta::new(self.associated_bonding_curve, false), // #5 - Quote Mint (TSFart)
                AccountMeta::new(self.associated_user, false), // #6 - Associated User
                AccountMeta::new(*SIGNER_PUBKEY, true),        // #7 - User
                AccountMeta::new_readonly(self.system_program, false), // #8 - System Program
                AccountMeta::new(updated_creator_vault, false), // #9 - Creator Vault
                AccountMeta::new_readonly(self.token_program, false), // #10 - Token Program
                AccountMeta::new_readonly(self.event_authority, false), // #11 - Event authority
                AccountMeta::new_readonly(self.program, false), // #12 - Pump.fun program
                AccountMeta::new_readonly(self.fee_config, false), // #13 - Fee Config
                AccountMeta::new_readonly(self.fee_program, false), //#14 - Fee Program
                AccountMeta::new_readonly(self.bonding_curve_v2_pda, false), //#15 - Bonding Curve V2 PDA
            ]
        } else {
            vec![
                AccountMeta::new_readonly(self.global, false), // #1 - Global
                AccountMeta::new(self.fee_recipient, false),   // #2 - Fee Recipient
                AccountMeta::new_readonly(self.mint, false),   // #3 - Mint
                AccountMeta::new(self.bonding_curve, false),   // #4 - BondingCurve
                AccountMeta::new(self.associated_bonding_curve, false), // #5 - Quote Mint (TSFart)
                AccountMeta::new(self.associated_user, false), // #6 - Associated User
                AccountMeta::new(*SIGNER_PUBKEY, true),        // #7 - User
                AccountMeta::new_readonly(self.system_program, false), // #8 - System Program
                AccountMeta::new(updated_creator_vault, false), // #9 - Creator Vault
                AccountMeta::new_readonly(self.token_program, false), // #10 - Token Program
                AccountMeta::new_readonly(self.event_authority, false), // #11 - Event authority
                AccountMeta::new_readonly(self.program, false), // #12 - Pump.fun program
                AccountMeta::new_readonly(self.fee_config, false), // #13 - Fee Config
                AccountMeta::new_readonly(self.fee_program, false), //#14 - Fee Program
                AccountMeta::new(self.user_volume_accumulator, false), //#15 - User Volume Accumulator
                AccountMeta::new_readonly(self.bonding_curve_v2_pda, false), //#16 - Bonding Curve V2 PDA
            ]
        };

        Instruction {
            program_id: PUMPFUN_PROGRAM_ID,
            accounts,
            data,
        }
    }
}
