use crate::*;
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

pub fn filter_by_program_id(
    ixs: Vec<CompiledInstruction>,
    inner_ixs: Vec<InnerInstruction>,
    account_keys: Vec<Pubkey>,
    program_id: Pubkey,
) -> Result<Vec<InstructionRawData>, Box<dyn std::error::Error>> {
    let program_id_index = match account_keys.iter().position(|&pos| pos == program_id) {
        Some(index) => index,
        None => {
            println!("Program not found");
            return Err("program_id not found".into());
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

pub fn get_trade_info(ix_infos: Vec<InstructionRawData>, account_keys: Vec<Pubkey>) -> Vec<MintInstructionAccounts> {
    let mut mint_instruction_accounts: Vec<MintInstructionAccounts> = Vec::new();
    ix_infos.iter().for_each(|info| {
        if info.data.starts_with(&PUMP_FUN_MINT_DISCRIMINATOR) {
            let mint_accounts = MintInstructionAccounts {
                mint: account_keys[info.accounts[0] as usize],
                mint_authority: account_keys[info.accounts[1] as usize],
                bonding_curve: account_keys[info.accounts[2] as usize],
                associated_bonding_curve: account_keys[info.accounts[3] as usize],
                global: account_keys[info.accounts[4] as usize],
                mpl_token_metadata: account_keys[info.accounts[5] as usize],
                metadata: account_keys[info.accounts[6] as usize],
                user: account_keys[info.accounts[7] as usize],
                system_program: account_keys[info.accounts[8] as usize],
                token_program: account_keys[info.accounts[9] as usize],
                associated_token_program: account_keys[info.accounts[10] as usize],
                rent: account_keys[info.accounts[11] as usize],
                event_authority: account_keys[info.accounts[12] as usize],
                program: account_keys[info.accounts[13] as usize],
            };
            mint_instruction_accounts.push(mint_accounts);
        }
    });

    mint_instruction_accounts
}
