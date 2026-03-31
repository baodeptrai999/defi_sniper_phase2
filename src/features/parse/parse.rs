use crate::*;
use borsh::BorshDeserialize;
use solana_sdk::{bs58, pubkey::Pubkey};
use yellowstone_grpc_proto::{
    geyser::{SubscribeUpdate, subscribe_update::UpdateOneof},
    prelude::{CompiledInstruction, InnerInstruction, Message},
};

pub fn extract_transaction_data(
    update: &SubscribeUpdate,
) -> Option<(
    Vec<Pubkey>,
    Vec<CompiledInstruction>,
    Vec<InnerInstruction>,
    String,
    Vec<Pubkey>,
)> {
    let transaction_update = match &update.update_oneof {
        Some(UpdateOneof::Transaction(tx_update)) => tx_update,
        _ => return None,
    };

    let tx_info = transaction_update.transaction.as_ref()?;
    let transaction = tx_info.transaction.as_ref()?;
    let meta = tx_info.meta.as_ref()?;
    let tx_msg = transaction.message.as_ref()?;

    let (_, signers) = get_signers(tx_msg.clone());

    let mut account_keys: Vec<Pubkey> = tx_msg
        .account_keys
        .iter()
        .filter_map(|key_bytes| Pubkey::try_from(key_bytes.as_slice()).ok())
        .collect();

    account_keys.extend(
        meta.loaded_writable_addresses
            .iter()
            .filter_map(|key_bytes| Pubkey::try_from(key_bytes.as_slice()).ok()),
    );

    account_keys.extend(
        meta.loaded_readonly_addresses
            .iter()
            .filter_map(|key_bytes| Pubkey::try_from(key_bytes.as_slice()).ok()),
    );

    let ixs: Vec<CompiledInstruction> = tx_msg.instructions.clone();
    let inner_ixs: Vec<InnerInstruction> = meta
        .inner_instructions
        .iter()
        .flat_map(|ix| ix.instructions.clone())
        .collect();

    let signature = tx_info.signature.clone();
    let tx_id = bs58::encode(signature).into_string();

    Some((account_keys, ixs, inner_ixs, tx_id, signers))
}

pub fn get_signers(tx_msg: Message) -> (usize, Vec<Pubkey>) {
    let signer_count = tx_msg
        .header
        .map(|header| header.num_required_signatures as usize)
        .unwrap_or(0);

    let pubkeys: Vec<Pubkey> = tx_msg
        .account_keys
        .iter()
        .filter_map(|key_bytes| Pubkey::try_from(key_bytes.as_slice()).ok())
        .collect();

    let signer_pubkeys = &pubkeys[..signer_count.min(pubkeys.len())];
    (signer_count, signer_pubkeys.to_vec())
}

pub fn get_budget_compute_info(ix_infos: Vec<InstructionRawData>) -> (u32, u64) {
    let mut unit = 0;
    let mut micro_lamports = 0;
    ix_infos.iter().for_each(|info| {
        if info
            .data
            .starts_with(&SET_BUDGET_COMPUTE_UNIT_LIMIT_DISCRIMINATOR)
        {
            let mut data = &info.data[1..];
            let budget_compute_unit_limit = BudgetComputeUnitLimit::deserialize(&mut data).unwrap();
            unit = budget_compute_unit_limit.unit;
        } else if info
            .data
            .starts_with(&SET_BUDGET_COMPUTE_UNIT_PRICE_DISCRIMINATOR)
        {
            let mut data = &info.data[1..];
            let budget_compute_price = BudgetComputeUnitPrice::deserialize(&mut data).unwrap();
            micro_lamports = budget_compute_price.micro_lamports;
        }
    });

    (unit, micro_lamports)
}

pub fn get_pumpfun_trade_info(
    ix_infos: Vec<InstructionRawData>,
    account_keys: Vec<Pubkey>,
) -> (
    Vec<MintEvent>,
    Vec<PumpfunBuyEvent>,
    Vec<PumpfunSellEvent>,
    Vec<MintInstructionAccounts>,
    Vec<PumpfunBuyInstructionAccounts>,
    Vec<PumpfunSellInstructionAccounts>,
) {
    let mut mint_instruction_accounts: Vec<MintInstructionAccounts> = Vec::new();
    let mut buy_instruction_accounts: Vec<PumpfunBuyInstructionAccounts> = Vec::new();
    let mut sell_instruction_accounts: Vec<PumpfunSellInstructionAccounts> = Vec::new();
    let mut mint_events: Vec<MintEvent> = Vec::new();
    let mut buy_events: Vec<PumpfunBuyEvent> = Vec::new();
    let mut sell_events: Vec<PumpfunSellEvent> = Vec::new();

    ix_infos.iter().for_each(|info| {
        if info.data.starts_with(&PUMP_FUN_CREATE_V1_DISCRIMINATOR) {
            let mint_accounts = MintInstructionAccounts {
                mint: account_keys[info.accounts[0] as usize],
                bonding_curve: account_keys[info.accounts[2] as usize],
                associated_bonding_curve: account_keys[info.accounts[3] as usize],
                user: account_keys[info.accounts[7] as usize],
                system_program: account_keys[info.accounts[8] as usize],
                token_program: account_keys[info.accounts[9] as usize],
                associated_token_program: account_keys[info.accounts[10] as usize],
                event_authority: account_keys[info.accounts[12] as usize],
            };
            mint_instruction_accounts.push(mint_accounts);
        } else if info.data.starts_with(&PUMP_FUN_CREATE_V2_DISCRIMINATOR) {
            let mint_accounts = MintInstructionAccounts {
                mint: account_keys[info.accounts[0] as usize],
                bonding_curve: account_keys[info.accounts[2] as usize],
                associated_bonding_curve: account_keys[info.accounts[3] as usize],
                user: account_keys[info.accounts[5] as usize],
                system_program: account_keys[info.accounts[6] as usize],
                token_program: account_keys[info.accounts[7] as usize],
                associated_token_program: account_keys[info.accounts[8] as usize],
                event_authority: account_keys[info.accounts[14] as usize],
            };
            mint_instruction_accounts.push(mint_accounts);
        } else if info.data.starts_with(&PUMP_FUN_BUY_DISCRIMINATOR)
            || info
                .data
                .starts_with(&PUMP_FUN_BUY_EXACT_SOL_IN_DISCRIMINATOR)
        {
            let buy_accounts = PumpfunBuyInstructionAccounts {
                global: account_keys[info.accounts[0] as usize],
                fee_recipient: account_keys[info.accounts[1] as usize],
                mint: account_keys[info.accounts[2] as usize],
                bonding_curve: account_keys[info.accounts[3] as usize],
                associated_bonding_curve: account_keys[info.accounts[4] as usize],
                associated_user: account_keys[info.accounts[5] as usize],
                user: account_keys[info.accounts[6] as usize],
                system_program: account_keys[info.accounts[7] as usize],
                token_program: account_keys[info.accounts[8] as usize],
                creator_vault: account_keys[info.accounts[9] as usize],
                event_authority: account_keys[info.accounts[10] as usize],
                program: account_keys[info.accounts[11] as usize],
                global_volume_accumulator: account_keys[info.accounts[12] as usize],
                user_volume_accumulator: account_keys[info.accounts[13] as usize],
                fee_config: account_keys[info.accounts[14] as usize],
                fee_program: account_keys[info.accounts[15] as usize],
            };
            buy_instruction_accounts.push(buy_accounts);
        } else if info.data.starts_with(&PUMP_FUN_SELL_DISCRIMINATOR) {
            let sell_accounts = PumpfunSellInstructionAccounts {
                global: account_keys[info.accounts[0] as usize],
                fee_recipient: account_keys[info.accounts[0] as usize],
                mint: account_keys[info.accounts[0] as usize],
                bonding_curve: account_keys[info.accounts[0] as usize],
                associated_bonding_curve: account_keys[info.accounts[0] as usize],
                associated_user: account_keys[info.accounts[0] as usize],
                user: account_keys[info.accounts[0] as usize],
                system_program: account_keys[info.accounts[0] as usize],
                creator_vault: account_keys[info.accounts[0] as usize],
                token_program: account_keys[info.accounts[0] as usize],
                event_authority: account_keys[info.accounts[0] as usize],
                program: account_keys[info.accounts[0] as usize],
                fee_config: account_keys[info.accounts[0] as usize],
                fee_program: account_keys[info.accounts[0] as usize],
            };
            sell_instruction_accounts.push(sell_accounts);
        } else if info.data.starts_with(
            &[
                PUMP_FUN_EVENT_LOG_DISCRIMINATOR,
                PUMP_FUN_MINT_EVENT_DISCRIMINATOR,
            ]
            .concat(),
        ) {
            let mut data = &info.data[16..];
            let mint_event: MintEvent = MintEvent::deserialize(&mut data).unwrap();
            let mint_info: MintEvent = MintEvent {
                name: mint_event.name,
                symbol: mint_event.symbol,
                uri: mint_event.uri,
                mint: mint_event.mint,
                bonding_curve: mint_event.bonding_curve,
                user: mint_event.user,
                creator: mint_event.creator,
                timestamp: mint_event.timestamp,
                virtual_token_reserves: mint_event.virtual_token_reserves,
                virtual_sol_reserves: mint_event.virtual_sol_reserves,
                real_token_reserves: mint_event.real_token_reserves,
                token_total_supply: mint_event.token_total_supply,
                token_program: mint_event.token_program,
                is_mayhem_mode: mint_event.is_mayhem_mode,
                is_cashback_enabled: mint_event.is_cashback_enabled
            };
            mint_events.push(mint_info);
        } else if info.data.starts_with(
            &[
                PUMP_FUN_EVENT_LOG_DISCRIMINATOR,
                PUMP_FUN_TRADE_EVENT_DISCRIMINATOR,
            ]
            .concat(),
        ) {
            let mut data = &info.data[16..];
            let trade_event = PumpfunTradeEvent::deserialize(&mut data).unwrap();
            if trade_event.is_buy {
                let buy_event = PumpfunBuyEvent {
                    mint: trade_event.mint,
                    sol_amount: trade_event.sol_amount,
                    token_amount: trade_event.token_amount,
                    user: trade_event.user,
                    timestamp: trade_event.timestamp,
                    virtual_sol_reserves: trade_event.virtual_sol_reserves,
                    virtual_token_reserves: trade_event.virtual_token_reserves,
                    real_sol_reserves: trade_event.real_sol_reserves,
                    real_token_reserves: trade_event.real_token_reserves,
                    fee_recipient: trade_event.fee_recipient,
                    fee_basis_points: trade_event.fee_basis_points,
                    fee: trade_event.fee,
                    creator: trade_event.creator,
                    creator_fee_basis_points: trade_event.creator_fee_basis_points,
                    creator_fee: trade_event.creator_fee,
                };
                buy_events.push(buy_event);
            } else {
                let sell_event = PumpfunSellEvent {
                    mint: trade_event.mint,
                    sol_amount: trade_event.sol_amount,
                    token_amount: trade_event.token_amount,
                    user: trade_event.user,
                    timestamp: trade_event.timestamp,
                    virtual_sol_reserves: trade_event.virtual_sol_reserves,
                    virtual_token_reserves: trade_event.virtual_token_reserves,
                    real_sol_reserves: trade_event.real_sol_reserves,
                    real_token_reserves: trade_event.real_token_reserves,
                    fee_recipient: trade_event.fee_recipient,
                    fee_basis_points: trade_event.fee_basis_points,
                    fee: trade_event.fee,
                    creator: trade_event.creator,
                    creator_fee_basis_points: trade_event.creator_fee_basis_points,
                    creator_fee: trade_event.creator_fee,
                };
                sell_events.push(sell_event);
            }
        }
    });

    (
        mint_events,
        buy_events,
        sell_events,
        mint_instruction_accounts,
        buy_instruction_accounts,
        sell_instruction_accounts,
    )
}

///////Pumpswap trade info extraction
///
///
///
///
pub fn migrate_info(
    infos: Vec<InstructionRawData>,
    account_keys: Vec<Pubkey>,
) -> (
    Vec<MigrateInstructionAccounts>,
    Vec<CreatePoolInstructionAccounts>,
    Vec<CreatePoolEventData>,
) {
    let mut migrate_accounts: Vec<MigrateInstructionAccounts> = vec![];
    let mut create_pool_accounts: Vec<CreatePoolInstructionAccounts> = vec![];
    let mut create_pool_events: Vec<CreatePoolEventData> = vec![];

    infos.iter().for_each(|info| {
        if info.data.starts_with(&MIGRATE_DISCRIMINATOR) {
            let migrate_account = MigrateInstructionAccounts {
                global: account_keys[info.accounts[0] as usize],
                withdraw_authority: account_keys[info.accounts[1] as usize],
                mint: account_keys[info.accounts[2] as usize],
                bonding_curve: account_keys[info.accounts[3] as usize],
                associated_bonding_curve: account_keys[info.accounts[4] as usize],
                user: account_keys[info.accounts[5] as usize],
                system_program: account_keys[info.accounts[6] as usize],
                token_program: account_keys[info.accounts[7] as usize],
                pump_amm_program: account_keys[info.accounts[8] as usize],
                pool: account_keys[info.accounts[9] as usize],
                pool_authority: account_keys[info.accounts[10] as usize],
                pool_authority_mint_account: account_keys[info.accounts[11] as usize],
                pool_authority_wsol_account: account_keys[info.accounts[12] as usize],
                amm_global_config: account_keys[info.accounts[13] as usize],
                wsol_mint: account_keys[info.accounts[14] as usize],
                lp_mint: account_keys[info.accounts[15] as usize],
                user_pool_token_account: account_keys[info.accounts[16] as usize],
                pool_base_token_account: account_keys[info.accounts[17] as usize],
                pool_quote_token_account: account_keys[info.accounts[18] as usize],
                token_2022_program: account_keys[info.accounts[19] as usize],
                associated_token_program: account_keys[info.accounts[20] as usize],
                pump_amm_event_authority: account_keys[info.accounts[21] as usize],
                event_authority: account_keys[info.accounts[22] as usize],
                pump_fun_program: account_keys[info.accounts[23] as usize],
            };

            migrate_accounts.push(migrate_account);
        } else if info.data.starts_with(&CREATE_POOL_DISCRIMINATOR) {
            let create_pool_account = CreatePoolInstructionAccounts {
                pool: account_keys[info.accounts[0] as usize],
                global_config: account_keys[info.accounts[1] as usize],
                creator: account_keys[info.accounts[2] as usize],
                base_mint: account_keys[info.accounts[3] as usize],
                quote_mint: account_keys[info.accounts[4] as usize],
                lp_mint: account_keys[info.accounts[5] as usize],
                user_base_token_account: account_keys[info.accounts[6] as usize],
                user_quote_token_account: account_keys[info.accounts[7] as usize],
                user_pool_token_account: account_keys[info.accounts[8] as usize],
                pool_base_token_account: account_keys[info.accounts[9] as usize],
                pool_quote_token_account: account_keys[info.accounts[10] as usize],
                system_program: account_keys[info.accounts[11] as usize],
                token_2022_program: account_keys[info.accounts[12] as usize],
                base_token_program: account_keys[info.accounts[13] as usize],
                quote_token_program: account_keys[info.accounts[14] as usize],
                associated_token_program: account_keys[info.accounts[15] as usize],
                event_authority: account_keys[info.accounts[16] as usize],
                pump_amm_program: account_keys[info.accounts[17] as usize],
            };

            create_pool_accounts.push(create_pool_account);
        } else if info.data.starts_with(
            &[
                PUMP_FUN_EVENT_LOG_DISCRIMINATOR,
                CREATE_POOL_EVENT_DISCRIMINATOR,
            ]
            .concat(),
        ) {
            let mut data = &info.data[16..];
            let create_pool_event_data = CreatePoolEventData::deserialize(&mut data).unwrap();
            create_pool_events.push(create_pool_event_data);
        }
    });

    (migrate_accounts, create_pool_accounts, create_pool_events)
}

pub fn get_pumpswap_trade_info(
    infos: Vec<InstructionRawData>,
    account_keys: Vec<Pubkey>,
) -> (
    Vec<PumpswapBuyEvent>,
    Vec<PumpswapSellEvent>,
    Vec<PumpswapBuyInstructionAccounts>,
    Vec<PumpswapSellInstructionAccounts>,
) {
    let mut buy_events: Vec<PumpswapBuyEvent> = vec![];
    let mut sell_events: Vec<PumpswapSellEvent> = vec![];
    let mut buy_accounts: Vec<PumpswapBuyInstructionAccounts> = vec![];
    let mut sell_accounts: Vec<PumpswapSellInstructionAccounts> = vec![];
    infos.iter().for_each(|info| {
        if info.data.starts_with(&BUY_DISCRIMINATOR)
            || info.data.starts_with(&BUY_EXACT_QUOTE_IN_DISCRIMINATOR)
        {
            let buy_account = PumpswapBuyInstructionAccounts {
                pool: account_keys[info.accounts[0] as usize],
                user: account_keys[info.accounts[1] as usize],
                global_config: account_keys[info.accounts[2] as usize],
                base_mint: account_keys[info.accounts[3] as usize],
                quote_mint: account_keys[info.accounts[4] as usize],
                user_base_token_account: account_keys[info.accounts[5] as usize],
                user_quote_token_account: account_keys[info.accounts[6] as usize],
                pool_base_token_account: account_keys[info.accounts[7] as usize],
                pool_quote_token_account: account_keys[info.accounts[8] as usize],
                protocol_fee_recipient: account_keys[info.accounts[9] as usize],
                protocol_fee_recipient_token_account: account_keys[info.accounts[10] as usize],
                base_token_program: account_keys[info.accounts[11] as usize],
                quote_token_program: account_keys[info.accounts[12] as usize],
                system_program: account_keys[info.accounts[13] as usize],
                associated_token_program: account_keys[info.accounts[14] as usize],
                event_authority: account_keys[info.accounts[15] as usize],
                program: account_keys[info.accounts[16] as usize],
                coin_creator_vault_ata: account_keys[info.accounts[17] as usize],
                coin_creator_vault_authority: account_keys[info.accounts[18] as usize],
                global_volume_accumulator: account_keys[info.accounts[19] as usize],
                user_volume_accumulator: account_keys[info.accounts[20] as usize],
                fee_config: account_keys[info.accounts[21] as usize],
                fee_program: account_keys[info.accounts[22] as usize],
            };

            buy_accounts.push(buy_account);
        } else if info.data.starts_with(&SELL_DISCRIMINATOR) {
            let sell_account = PumpswapSellInstructionAccounts {
                pool: account_keys[info.accounts[0] as usize],
                user: account_keys[info.accounts[1] as usize],
                global_config: account_keys[info.accounts[2] as usize],
                base_mint: account_keys[info.accounts[3] as usize],
                quote_mint: account_keys[info.accounts[4] as usize],
                user_base_token_account: account_keys[info.accounts[5] as usize],
                user_quote_token_account: account_keys[info.accounts[6] as usize],
                pool_base_token_account: account_keys[info.accounts[7] as usize],
                pool_quote_token_account: account_keys[info.accounts[8] as usize],
                protocol_fee_recipient: account_keys[info.accounts[9] as usize],
                protocol_fee_recipient_token_account: account_keys[info.accounts[10] as usize],
                base_token_program: account_keys[info.accounts[11] as usize],
                quote_token_program: account_keys[info.accounts[12] as usize],
                system_program: account_keys[info.accounts[13] as usize],
                associated_token_program: account_keys[info.accounts[14] as usize],
                event_authority: account_keys[info.accounts[15] as usize],
                program: account_keys[info.accounts[16] as usize],
                coin_creator_vault_ata: account_keys[info.accounts[17] as usize],
                coin_creator_vault_authority: account_keys[info.accounts[18] as usize],
                fee_config: PUMPSWAP_FEE_CONFIG,
                fee_program: PUMPSWAP_FEE_PROGRAM,
            };

            sell_accounts.push(sell_account);
        } else if info
            .data
            .starts_with(&[EVENT_AUTH_ACC_DISC, BUY_EVENT_DISC].concat())
        {
            let mut data = &info.data[16..]; // skip the 8-byte discriminator
            let buy_event = PumpswapBuyEvent::deserialize(&mut data).unwrap();

            buy_events.push(buy_event);
        } else if info
            .data
            .starts_with(&[EVENT_AUTH_ACC_DISC, SELL_EVENT_DISC].concat())
        {
            let mut data = &info.data[16..]; // skip the 8-byte discriminator
            let sell_event = PumpswapSellEvent::deserialize(&mut data).unwrap();

            sell_events.push(sell_event);
        }
    });

    (buy_events, sell_events, buy_accounts, sell_accounts)
}

pub fn filter_by_program_id(
    ixs: Vec<CompiledInstruction>,
    inner_ixs: Vec<InnerInstruction>,
    program_id: Pubkey,
    account_keys: Vec<Pubkey>,
) -> Result<Vec<InstructionRawData>, Box<dyn std::error::Error>> {
    let program_id_index = match account_keys.iter().position(|&pos| pos == program_id) {
        Some(index) => index,
        None => {
            // println!("Program not found");
            return Ok(vec![]);
        }
    };

    let filtered_ixs = ixs
        .into_iter()
        .filter(|ix| ix.program_id_index == program_id_index as u32)
        .map(|ix| InstructionRawData {
            accounts: ix.accounts,
            data: ix.data,
            program_id_index: program_id_index as u32,
        });

    let filtered_inner_ixs = inner_ixs
        .into_iter()
        .filter(|ix| ix.program_id_index == program_id_index as u32)
        .map(|ix| InstructionRawData {
            accounts: ix.accounts,
            data: ix.data,
            program_id_index: program_id_index as u32,
        });

    Ok(filtered_ixs.chain(filtered_inner_ixs).collect())
}