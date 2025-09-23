use futures::StreamExt;
use yellowstone_grpc_proto::{geyser::SubscribeUpdate, tonic::Status};
use crate::*;

pub async fn process_sniper_mode<S>(mut stream: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: StreamExt<Item = Result<SubscribeUpdate, Status>> + Unpin,
{
    while let Some(result) = stream.next().await {
        match result {
            Ok(update) => {
                let (account_keys, ixs, inner_ixs, tx_id, signers) =
                    if let Some(data) = extract_transaction_data(&update) {
                        data
                    } else {
                        continue;
                    };
                let ix_info = filter_by_program_id(ixs, inner_ixs, account_keys.clone(), PUMPFUN_PROGRAM_ID).unwrap();
                let trade_data: Vec<MintInstructionAccounts> = get_trade_info(ix_info, account_keys.clone());

                handle_sniper(trade_data, tx_id).await;
            }

            Err(e) => {
                log!("Stream error: {}", e);
            }
        }
    }

    Ok(())
}
