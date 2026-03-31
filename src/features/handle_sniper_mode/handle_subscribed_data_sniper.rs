use crate::*;
use futures::StreamExt;
use yellowstone_grpc_proto::{geyser::SubscribeUpdate, tonic::Status};

pub async fn process_sniper_mode<S>(mut stream: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: StreamExt<Item = Result<SubscribeUpdate, Status>> + Unpin,
{
    while let Some(result) = stream.next().await {
        match result {
            Ok(update) => {
                let (account_keys, ixs, inner_ixs, tx_id, _signers) =
                    if let Some(data) = extract_transaction_data(&update) {
                        data
                    } else {
                        continue;
                    };

                let budget_compute_ix_info = filter_by_program_id(
                    ixs.clone(),
                    inner_ixs.clone(),
                    BUDGET_COMPUTE_PROGRAM,
                    account_keys.clone(),
                )
                .unwrap();

                let ix_info_pumpfun = filter_by_program_id(
                    ixs.clone(),
                    inner_ixs.clone(),
                    PUMPFUN_PROGRAM_ID,
                    account_keys.clone(),
                )
                .unwrap();

                let ix_info_pumpswap = match filter_by_program_id(
                    ixs.clone(),
                    inner_ixs.clone(),
                    PUMPSWAP_PROGRAM_ID,
                    account_keys.clone(),
                ) {
                    Ok(data) => data,
                    Err(_) => {
                        vec![]
                    }
                };

                let mut all_pump_ix = vec![];
                all_pump_ix.extend(ix_info_pumpfun.clone());
                all_pump_ix.extend(ix_info_pumpswap.clone());

                let budget_compute_data = get_budget_compute_info(budget_compute_ix_info);
                let pumpfun_trade_data =
                    get_pumpfun_trade_info(ix_info_pumpfun.clone(), account_keys.clone());

                let migration_data = migrate_info(all_pump_ix.clone(), account_keys.clone());

                let pumpswap_trade_data =
                    get_pumpswap_trade_info(ix_info_pumpswap.clone(), account_keys.clone());

                let trade_token_data_map = handle_trade_events(
                    budget_compute_data,
                    pumpfun_trade_data,
                    migration_data,
                    pumpswap_trade_data,
                    tx_id.clone(),
                )
                .await;

                make_sniper_tx(&trade_token_data_map).await;
            }

            Err(e) => {
                log!("Stream error: {}", e);
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}
