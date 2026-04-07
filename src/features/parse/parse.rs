use crate::*;
use borsh::BorshDeserialize;
use solana_sdk::{bs58, pubkey::Pubkey};
use yellowstone_grpc_proto::{
    geyser::{SubscribeUpdate, subscribe_update::UpdateOneof},
    prelude::{CompiledInstruction, InnerInstruction, Message, SubscribeUpdateTransactionInfo},
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
            if let Ok(budget_compute_unit_limit) = BudgetComputeUnitLimit::deserialize(&mut data) {
                unit = budget_compute_unit_limit.unit;
            }
        } else if info
            .data
            .starts_with(&SET_BUDGET_COMPUTE_UNIT_PRICE_DISCRIMINATOR)
        {
            let mut data = &info.data[1..];
            if let Ok(budget_compute_price) = BudgetComputeUnitPrice::deserialize(&mut data) {
                micro_lamports = budget_compute_price.micro_lamports;
            }
        }
    });

    (unit, micro_lamports)
}

pub fn extract_mint_tx_context(
    transaction_update: &SubscribeUpdateTransactionInfo,
    pumpfun_ix_infos: &[InstructionRawData],
) -> MintTransactionContext {
    let mut ctx = MintTransactionContext::default();

    if let Some(tx) = transaction_update.transaction.as_ref() {
        if let Some(tx_msg) = tx.message.as_ref() {
            // Transaction type
            ctx.tx_type = if tx_msg.versioned {
                TxType::V0
            } else {
                TxType::Legacy
            };

            // Address lookup table address
            ctx.alt_addresses = tx_msg
                .address_table_lookups
                .iter()
                .filter_map(|alt| Pubkey::try_from(alt.account_key.as_slice()).ok())
                .collect();

            // Collect names for all top-level instructions
            if let Some(meta) = transaction_update.meta.as_ref() {
                let mut account_keys: Vec<Pubkey> = tx_msg
                    .account_keys
                    .iter()
                    .filter_map(|k| Pubkey::try_from(k.as_slice()).ok())
                    .collect();
                account_keys.extend(
                    meta.loaded_writable_addresses
                        .iter()
                        .filter_map(|k| Pubkey::try_from(k.as_slice()).ok()),
                );
                account_keys.extend(
                    meta.loaded_readonly_addresses
                        .iter()
                        .filter_map(|k| Pubkey::try_from(k.as_slice()).ok()),
                );

                // Top-level instructions
                for ix in &tx_msg.instructions {
                    let program_id = account_keys.get(ix.program_id_index as usize);
                    if program_id == Some(&PUMPFUN_PROGRAM_ID) {
                        if let Some(name) = identify_instruction(&ix.data) {
                            ctx.all_instruction_names
                                .push(name.to_string());
                        } else {
                            ctx.all_instruction_names
                                .push(format!("Pumpfun:Unknown({:?})", &ix.data.get(..8)));
                        }
                    } else if program_id == Some(&BUDGET_COMPUTE_PROGRAM) {
                        match ix.data.first() {
                            Some(&2) => ctx
                                .all_instruction_names
                                .push("CB:SetComputeUnitLimit".to_string()),
                            Some(&3) => ctx
                                .all_instruction_names
                                .push("CB:SetComputeUnitPrice".to_string()),
                            _ => ctx
                                .all_instruction_names
                                .push("CB:Other".to_string()),
                        }
                    } else if program_id == Some(&SYSTEM_PROGRAM) {
                        ctx.all_instruction_names
                            .push(identify_system_program_ix(&ix.data));
                    } else if program_id == Some(&ASSOCIATED_PROGRAM) {
                        ctx.all_instruction_names
                            .push(identify_ata_program_ix(&ix.data));
                    } else if program_id == Some(&TOKEN_2022_PROGRAM) {
                        ctx.all_instruction_names
                            .push(identify_token2022_program_ix(&ix.data));
                    } else if let Some(pid) = program_id {
                        ctx.all_instruction_names.push(format!("Program:{}", pid));
                    }
                }
            }
        }
    }

    // Token version + buy instruction name/data from the parsed PumpFun instructions
    for info in pumpfun_ix_infos {
        if info.data.starts_with(&ix_discriminator::CREATE) {
            ctx.token_version = TokenVersion::V1;
        } else if info.data.starts_with(&ix_discriminator::CREATE_V2) {
            ctx.token_version = TokenVersion::V2;
        } else if info.data.starts_with(&ix_discriminator::BUY) {
            ctx.buy_ix_name = Some("Pumpfun:Buy".to_string());
            if let Ok(args) = BuyArgs::deserialize_from_slice(&mut &info.data[8..]) {
                let structured = BuyInstructionData {
                    ix_name: "buy".to_string(),
                    amount: args.amount,
                    max_sol_cost: args.max_sol_cost,
                };
                ctx.buy_ix_data = serde_json::to_string(&structured).ok();
            }
            break;
        } else if info.data.starts_with(&ix_discriminator::BUY_EXACT_SOL_IN) {
            ctx.buy_ix_name = Some("Pumpfun:BuyExactSolIn".to_string());
            if let Ok(args) = BuyExactSolInArgs::deserialize_from_slice(&mut &info.data[8..]) {
                let structured = BuyExactSolInInstructionData {
                    ix_name: "buy_exact_sol_in".to_string(),
                    spendable_sol_in: args.spendable_sol_in,
                    min_tokens_out: args.min_tokens_out,
                };
                ctx.buy_ix_data = serde_json::to_string(&structured).ok();
            }
            break;
        }
    }

    ctx
}

pub fn get_pumpfun_trade_info(
    ix_infos: Vec<InstructionRawData>,
    account_keys: Vec<Pubkey>,
    transaction_update: &SubscribeUpdateTransactionInfo,
) -> (
    Vec<MintContext>,
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
        let acc = |i: usize| account_keys[info.accounts[i] as usize];

        if info.data.starts_with(&ix_discriminator::CREATE) {
            use ix_accounts::create as c;
            let mint_accounts = MintInstructionAccounts {
                mint: acc(c::MINT),
                bonding_curve: acc(c::BONDING_CURVE),
                associated_bonding_curve: acc(c::ASSOCIATED_BONDING_CURVE),
                user: acc(c::USER),
                system_program: acc(c::SYSTEM_PROGRAM),
                token_program: acc(c::TOKEN_PROGRAM),
                associated_token_program: acc(c::ASSOCIATED_TOKEN_PROGRAM),
                event_authority: acc(c::EVENT_AUTHORITY),
            };
            mint_instruction_accounts.push(mint_accounts);
        } else if info.data.starts_with(&ix_discriminator::CREATE_V2) {
            use ix_accounts::create_v2 as c;
            let mint_accounts = MintInstructionAccounts {
                mint: acc(c::MINT),
                bonding_curve: acc(c::BONDING_CURVE),
                associated_bonding_curve: acc(c::ASSOCIATED_BONDING_CURVE),
                user: acc(c::USER),
                system_program: acc(c::SYSTEM_PROGRAM),
                token_program: acc(c::TOKEN_PROGRAM),
                associated_token_program: acc(c::ASSOCIATED_TOKEN_PROGRAM),
                event_authority: acc(c::EVENT_AUTHORITY),
            };
            mint_instruction_accounts.push(mint_accounts);
        } else if info.data.starts_with(&ix_discriminator::BUY) {
            use ix_accounts::buy as b;
            let mut ix_data = &info.data[8..];
            let instruction_data = match BuyArgs::deserialize_from_slice(&mut ix_data) {
                Ok(args) => PumpfunBuyInstructionData::Buy {
                    amount: args.amount,
                    max_sol_cost: args.max_sol_cost,
                    track_volume: args.track_volume,
                },
                Err(_) => PumpfunBuyInstructionData::None,
            };
            let buy_accounts = PumpfunBuyInstructionAccounts {
                global: acc(b::GLOBAL),
                fee_recipient: acc(b::FEE_RECIPIENT),
                mint: acc(b::MINT),
                bonding_curve: acc(b::BONDING_CURVE),
                associated_bonding_curve: acc(b::ASSOCIATED_BONDING_CURVE),
                associated_user: acc(b::ASSOCIATED_USER),
                user: acc(b::USER),
                system_program: acc(b::SYSTEM_PROGRAM),
                token_program: acc(b::TOKEN_PROGRAM),
                creator_vault: acc(b::CREATOR_VAULT),
                event_authority: acc(b::EVENT_AUTHORITY),
                program: acc(b::PROGRAM),
                global_volume_accumulator: acc(b::GLOBAL_VOLUME_ACCUMULATOR),
                user_volume_accumulator: acc(b::USER_VOLUME_ACCUMULATOR),
                fee_config: acc(b::FEE_CONFIG),
                fee_program: acc(b::FEE_PROGRAM),
                instruction_data,
            };
            buy_instruction_accounts.push(buy_accounts);
        } else if info.data.starts_with(&ix_discriminator::BUY_EXACT_SOL_IN) {
            use ix_accounts::buy_exact_sol_in as b;
            let mut ix_data = &info.data[8..];
            let instruction_data = match BuyExactSolInArgs::deserialize_from_slice(&mut ix_data) {
                Ok(args) => PumpfunBuyInstructionData::BuyExactSolIn {
                    spendable_sol_in: args.spendable_sol_in,
                    min_tokens_out: args.min_tokens_out,
                    track_volume: args.track_volume,
                },
                Err(_) => PumpfunBuyInstructionData::None,
            };
            let buy_accounts = PumpfunBuyInstructionAccounts {
                global: acc(b::GLOBAL),
                fee_recipient: acc(b::FEE_RECIPIENT),
                mint: acc(b::MINT),
                bonding_curve: acc(b::BONDING_CURVE),
                associated_bonding_curve: acc(b::ASSOCIATED_BONDING_CURVE),
                associated_user: acc(b::ASSOCIATED_USER),
                user: acc(b::USER),
                system_program: acc(b::SYSTEM_PROGRAM),
                token_program: acc(b::TOKEN_PROGRAM),
                creator_vault: acc(b::CREATOR_VAULT),
                event_authority: acc(b::EVENT_AUTHORITY),
                program: acc(b::PROGRAM),
                global_volume_accumulator: acc(b::GLOBAL_VOLUME_ACCUMULATOR),
                user_volume_accumulator: acc(b::USER_VOLUME_ACCUMULATOR),
                fee_config: acc(b::FEE_CONFIG),
                fee_program: acc(b::FEE_PROGRAM),
                instruction_data,
            };
            buy_instruction_accounts.push(buy_accounts);
        } else if info.data.starts_with(&ix_discriminator::SELL) {
            use ix_accounts::sell as s;
            let sell_accounts = PumpfunSellInstructionAccounts {
                global: acc(s::GLOBAL),
                fee_recipient: acc(s::FEE_RECIPIENT),
                mint: acc(s::MINT),
                bonding_curve: acc(s::BONDING_CURVE),
                associated_bonding_curve: acc(s::ASSOCIATED_BONDING_CURVE),
                associated_user: acc(s::ASSOCIATED_USER),
                user: acc(s::USER),
                system_program: acc(s::SYSTEM_PROGRAM),
                creator_vault: acc(s::CREATOR_VAULT),
                token_program: acc(s::TOKEN_PROGRAM),
                event_authority: acc(s::EVENT_AUTHORITY),
                program: acc(s::PROGRAM),
                fee_config: acc(s::FEE_CONFIG),
                fee_program: acc(s::FEE_PROGRAM),
            };
            sell_instruction_accounts.push(sell_accounts);
        } else if info.data.starts_with(
            &[
                event_discriminator::ANCHOR_EVENT_LOG,
                event_discriminator::CREATE_EVENT,
            ]
            .concat(),
        ) {
            let mut data = &info.data[16..];
            if let Ok(mint_event) = IdlCreateEvent::deserialize(&mut data) {
                mint_events.push(MintEvent {
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
                    is_cashback_enabled: mint_event.is_cashback_enabled,
                });
            }
        } else if info.data.starts_with(
            &[
                event_discriminator::ANCHOR_EVENT_LOG,
                event_discriminator::TRADE_EVENT,
            ]
            .concat(),
        ) {
            let mut data = &info.data[16..];
            if let Ok(trade_event) = IdlTradeEvent::deserialize(&mut data) {
                if trade_event.is_buy {
                    buy_events.push(PumpfunBuyEvent {
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
                        ix_name: trade_event.ix_name,
                        track_volume: trade_event.track_volume,
                        cashback_fee_basis_points: trade_event.cashback_fee_basis_points,
                        cashback: trade_event.cashback,
                    });
                } else {
                    sell_events.push(PumpfunSellEvent {
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
                        ix_name: trade_event.ix_name,
                        track_volume: trade_event.track_volume,
                        cashback_fee_basis_points: trade_event.cashback_fee_basis_points,
                        cashback: trade_event.cashback,
                    });
                }
            }
        }
    });

    // Build MintContext by pairing each MintEvent with the transaction context
    let mint_contexts = if !mint_events.is_empty() {
        let tx_ctx = extract_mint_tx_context(transaction_update, &ix_infos);
        mint_events
            .into_iter()
            .map(|mint_event| MintContext {
                mint_event,
                mint_transaction_context: tx_ctx.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };

    (
        mint_contexts,
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
                event_discriminator::ANCHOR_EVENT_LOG,
                event_discriminator::CREATE_EVENT,
            ]
            .concat(),
        ) {
            let mut data = &info.data[16..];
            if let Ok(create_pool_event_data) = CreatePoolEventData::deserialize(&mut data) {
                create_pool_events.push(create_pool_event_data);
            }
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
        if info.data.starts_with(&PUMPSWAP_BUY_DISCRIMINATOR)
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
        } else if info.data.starts_with(&PUMPSWAP_SELL_DISCRIMINATOR) {
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
            if let Ok(buy_event) = PumpswapBuyEvent::deserialize(&mut data) {
                buy_events.push(buy_event);
            }
        } else if info
            .data
            .starts_with(&[EVENT_AUTH_ACC_DISC, SELL_EVENT_DISC].concat())
        {
            let mut data = &info.data[16..]; // skip the 8-byte discriminator
            if let Ok(sell_event) = PumpswapSellEvent::deserialize(&mut data) {
                sell_events.push(sell_event);
            }
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

pub fn group_by_program_ids(
    ixs: Vec<CompiledInstruction>,
    inner_ixs: Vec<InnerInstruction>,
    program_ids: &[Pubkey],
    account_keys: &[Pubkey],
) -> Vec<Vec<InstructionRawData>> {
    let index_map: Vec<Option<u32>> = program_ids
        .iter()
        .map(|pid| account_keys.iter().position(|k| k == pid).map(|i| i as u32))
        .collect();

    let mut results: Vec<Vec<InstructionRawData>> = vec![Vec::new(); program_ids.len()];

    let mut dispatch = |prog_idx: u32, accounts: Vec<u8>, data: Vec<u8>| {
        for (slot, opt_idx) in index_map.iter().enumerate() {
            if *opt_idx == Some(prog_idx) {
                results[slot].push(InstructionRawData {
                    accounts,
                    data,
                    program_id_index: prog_idx,
                });
                return;
            }
        }
    };

    for ix in ixs {
        dispatch(ix.program_id_index, ix.accounts, ix.data);
    }
    for ix in inner_ixs {
        dispatch(ix.program_id_index, ix.accounts, ix.data);
    }

    results
}
