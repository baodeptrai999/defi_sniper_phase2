use crate::*;
use futures::StreamExt;
use std::sync::atomic::Ordering;
use yellowstone_grpc_proto::{geyser::SubscribeUpdate, tonic::Status};

pub async fn process_sniper_mode<S>(mut stream: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: StreamExt<Item = Result<SubscribeUpdate, Status>> + Unpin,
{
    while let Some(result) = stream.next().await {
        match result {
            Ok(update) => {
                if AUTO_TURNOFF.load(Ordering::Relaxed) {
                    break;
                };
                let (account_keys, ixs, inner_ixs, tx_id, _signers) =
                    if let Some(data) = extract_transaction_data(&update) {
                        data
                    } else {
                        continue;
                    };
                let ix_info =
                    filter_by_program_id(ixs, inner_ixs, account_keys.clone(), PUMPFUN_PROGRAM_ID)
                        .unwrap();
                let trade_data = get_trade_info(ix_info, account_keys.clone());

                let trade_token_data_map = handle_sniper_event(trade_data, tx_id).await;

                make_sniper_tx(&trade_token_data_map).await;
            }

            Err(e) => {
                log!("Stream error: {}", e);
            }
        }
    }

    Ok(())
}
