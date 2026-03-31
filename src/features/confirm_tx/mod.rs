use crate::*;
use futures::FutureExt;
use futures::future::BoxFuture;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use solana_sdk::signature::Signature;
use tokio::time::{Duration, sleep};

#[derive(PartialEq)]
pub enum TradeType {
    Buy,
    Sell,
}

pub fn confirm(
    raw_instructions: Vec<Instruction>,
    tag: String,
) -> BoxFuture<'static, Option<Signature>> {
    async move {
        let results = send_zero_slot_transaction(raw_instructions, tag.clone()).await;

        if let Some(signature_str) = results {
            return wait_for_confirmation(&signature_str, tag.clone()).await;
        }

        None
    }
    .boxed()
}

pub fn confirm_sell_with_retry(
    mint: Pubkey,
    sell_amount: u64,
    raw_instructions: Vec<Instruction>,
    tag: String,
) -> BoxFuture<'static, Option<Signature>> {
    async move {
        let max_attempts = 3usize;
        let mut current_ix = raw_instructions;
        let mut current_tag = tag;

        for attempt in 1..=max_attempts {
            let submitted = send_zero_slot_transaction(current_ix, current_tag.clone()).await;

            if let Some(signature_str) = submitted
                && let Some(confirmed_sig) = wait_for_confirmation(&signature_str, current_tag.clone()).await
            {
                return Some(confirmed_sig);
            }

            if attempt < max_attempts {
                alert!(
                    "[SELL_RETRY]\n\t*Mint: {}\n\t*Attempt: {}/{}\n\t*Reason: tx failed or not confirmed, rebuilding sell route",
                    mint,
                    attempt,
                    max_attempts,
                );

                sleep(Duration::from_millis(600)).await;

                if let Some((next_ix, next_tag)) = build_retry_sell_instructions(mint, sell_amount) {
                    current_ix = next_ix;
                    current_tag = next_tag;
                    continue;
                }
            }

            reset_sell_submission_state(mint);
            return None;
        }

        reset_sell_submission_state(mint);
        None
    }
    .boxed()
}

fn build_retry_sell_instructions(mint: Pubkey, sell_amount: u64) -> Option<(Vec<Instruction>, String)> {
    let token_data = TOKEN_DB.get(mint).ok().flatten()?;
    let mut token_data = token_data.clone();
    let amount = sell_amount.min(token_data.token_balance);

    if amount == 0 {
        return None;
    }

    if token_data.token_is_migrated
        && let Some(mut pumpswap_struct) = token_data.pumpswap_struct
    {
        let mut ix: Vec<Instruction> = Vec::new();
        let create_ix: Vec<Instruction> = pumpswap_struct.get_create_ata_idempotent_ix();
        let sell_ix: Instruction = pumpswap_struct.get_sell_ix(
            amount,
            token_data.token_creator,
            token_data.is_cashback_enabled,
        );
        let close_ix = pumpswap_struct.close_wsol_ata();

        ix.extend(create_ix);
        ix.push(sell_ix);
        ix.push(close_ix);

        let tag = format!(
            "[SELL_RETRY]\t*ROUTE: PUMPSWAP\t*MINT: {}\t*AMOUNT: {}",
            mint, amount
        );

        return Some((ix, tag));
    }

    let sell_ix: Instruction = token_data.pumpfun_struct.get_sell_ix(
        token_data.token_creator,
        amount,
        token_data.is_cashback_enabled,
    );

    let mut ix: Vec<Instruction> = Vec::new();
    ix.push(sell_ix);
    if amount >= token_data.token_balance {
        ix.push(token_data.pumpfun_struct.get_close_ata_ix());
    }

    let tag = format!(
        "[SELL_RETRY]\t*ROUTE: PUMPFUN\t*MINT: {}\t*AMOUNT: {}",
        mint, amount
    );
    Some((ix, tag))
}

fn reset_sell_submission_state(mint: Pubkey) {
    if let Ok(Some(mut token_data)) = TOKEN_DB.get(mint) {
        token_data.token_sell_status = TokenSellStatus::None;
        token_data.tracked_sl_state = SLMode::None;
        let _ = TOKEN_DB.upsert(mint, token_data);
    }
}

pub async fn wait_for_confirmation(signature_str: &str, tag: String) -> Option<Signature> {
    let trimed_clean_sig = signature_str
        .trim()
        .replace("\"", "")
        .replace("'", "")
        .replace("\n", "")
        .replace("\r", "");
    let signature = match trimed_clean_sig.parse::<Signature>() {
        Ok(sig) => sig,
        Err(_) => {
            error!(
                "[FORCE_CHECK]
                \t* Check : {}
                \t* {}
                \t* States : Invalid signature",
                solscan!(signature_str),
                tag.clone()
            );

            return None;
        }
    };

    let mut attempts = 0;

    loop {
        match RPC_CLIENT.get_signature_statuses(&[signature]).await {
            Ok(statuses) => {
                if let Some(Some(status)) = statuses.value.get(0) {
                    if status.err.is_some() {
                        error!(
                            "[FORCE_CHECK]\n\t* Check : {}\n\t* States : Confirmed but failed on-chain\n\t* {}",
                            solscan!(signature),
                            tag
                        );
                        return None;
                    }

                    if status.confirmations.is_none() || status.confirmations.unwrap_or(0) > 0 {
                        success!(
                            "[FORCE_CHECK]
                            \t* Check : {}
                            \t* States : Confirmed
                            \t* {}",
                            solscan!(signature),
                            tag
                        );
                        return Some(signature);
                    }
                }
            }
            Err(_) => {}
        }

        attempts += 1;
        if attempts >= 10 {
            error!(
                "[FORCE_CHECK]
                \t* Check : https://solscan.io/tx/{}
                \t* States : Failed
                \t* {}",
                signature,
                tag.clone()
            );
            return None;
        }

        sleep(Duration::from_secs(2)).await;
    }
}
