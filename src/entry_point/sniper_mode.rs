use pumpfun_sniper::*;
use yellowstone_grpc_proto::geyser::SubscribeRequestFilterTransactions;

const PATTERN_SERVER_PORT: u16 = 3355;

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    info!("{}", SNIPER_MODE_STR.green());
    let client = get_http_client();

    match LANDING_SERVICE.as_str() {
        "ZERO_SLOT" => {
            pre_warm_zero_slot_endpoint(client).await;
        }
        "HELIUS" => {
            pre_warm_helius_endpoint(client).await;
        }
        _ => {
            println!("Unsupported landing service, defaulting to 0-slot");
            pre_warm_zero_slot_endpoint(client).await;
        }
    }

    init_nonce_pool().await;

    // Load manual patterns at startup
    let manual_count = get_manual_patterns().len();
    info!("Loaded {} manual pattern(s)", manual_count);

    // Show adaptive TP mode info
    info!("TP Mode: {}", *TP_MODE);
    if *TP_MODE == "EMA" {
        info!("EMA Alpha: {} (ATH tracked for 1h per token)", *EMA_ALPHA);
    } else if *TP_MODE == "AVERAGE" {
        info!("Average Window: {} tokens (ATH tracked for 1h per token)", *AVERAGE_WINDOW);
    }

    // Show dynamic buy amount mode
    if *DYNAMIC_BUY_AMOUNT_MODE {
        info!("Dynamic Buy Amount: ENABLED (3 losses → 2/3x, 2 wins → 2x, max = initial)");
    }

    // Spawn background ATH tracker expiry loop (every 60 seconds)
    if AdaptiveTpEngine::is_adaptive() {
        tokio::spawn(async {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                ADAPTIVE_TP.expire_trackers();
            }
        });
    }

    tokio::spawn(async {
        run_pattern_server(PATTERN_SERVER_PORT).await;
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
