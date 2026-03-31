use colored::*;
use pumpfun_sniper::*;
use yellowstone_grpc_proto::geyser::SubscribeRequestFilterTransactions;

const PATTERN_SERVER_PORT: u16 = 3355;

#[tokio::main]
pub async fn main() {
    info!("{}", SNIPER_MODE_STR.green());
    let client = get_zero_slot_client();
    pre_warm_zero_slot_endpoint(client).await;

    tokio::spawn(async {
        run_pattern_server(PATTERN_SERVER_PORT).await;
    });

    tokio::spawn({
        async {
            loop {
                recent_blockhash_handler().await;
            }
        }
    });

    let grpc_config = GrpcClientConfig::new(
        "sniper_mode".to_string(),
        GRPC_ENDPOINT.to_string(),
        GRPC_TOKEN.to_string(),
    );

    let subscribe_pumpfun_program_id = SubscribeRequestFilterTransactions {
        account_include: vec![
            PUMPFUN_PROGRAM_ID.to_string(),
            PUMPSWAP_PROGRAM_ID.to_string(),
        ],
        account_exclude: vec![],
        account_required: vec![],
        vote: Some(false),
        failed: Some(false),
        signature: None,
    };

    if let Err(e) = grpc_config
        .subscribe_with_reconnect(subscribe_pumpfun_program_id)
        .await
    {
        error!(
            "Failed to maintain GRPC connection after all retries: {:?}",
            e
        );
    }
}
