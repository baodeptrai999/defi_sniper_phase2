use colored::*;
use pumpfun_sniper::*;
use std::sync::atomic::Ordering;
use tokio::time::{Duration, interval};
use yellowstone_grpc_proto::geyser::SubscribeRequestFilterTransactions;
use std::path::PathBuf;

#[tokio::main]
pub async fn main() {
    info!("{}", COPY_MODE_STR.green());
    show_bot_settings();

    init_jito().await;
    init_nozomi().await;
    init_zero_slot().await;

    tokio::spawn({
        async {
            loop {
                recent_blockhash_handler().await;
            }
        }
    });

    tokio::spawn({
        async {
            loop {
                check_no_activity_tokens().await;
            }
        }
    });

    let mut interval = interval(Duration::from_millis(500));

    tokio::spawn({
        async move {
            loop {
                interval.tick().await;
                let start_selling = check_auto_turn_off_time("sniper_mode");
                if start_selling {
                    AUTO_TURNOFF.store(true, Ordering::Relaxed);
                };
            }
        }
    });

    tokio::spawn(watch_wallet_blacklist_file(PathBuf::from(
        WALLET_BLACKLIST_PATH.as_str(),
    )));

    tokio::spawn(watch_token_blacklist_file(PathBuf::from(
        TOKEN_BLACKLIST_PATH.as_str(),
    )));

    tokio::spawn({
        async move {
            loop {
                show_blacklist_length().await;
            }
        }
    });

    let mut grpc_client = setup_grpc_client(GRPC_ENDPOINT.to_string(), GRPC_TOKEN.to_string())
        .await
        .unwrap();

    let (subscribe_tx, subscribe_rx) = grpc_client.subscribe().await.unwrap();
    let subscribe_pumpfun_program_id = SubscribeRequestFilterTransactions {
        account_include: vec![],
        account_exclude: vec![],
        account_required: vec![PUMPFUN_PROGRAM_ID.to_string()],
        vote: Some(false),
        failed: Some(false),
        signature: None,
    };

    send_subscription_request_grpc(subscribe_tx, subscribe_pumpfun_program_id)
        .await
        .unwrap();

    let _ = process_copy_mode(subscribe_rx).await;
}
