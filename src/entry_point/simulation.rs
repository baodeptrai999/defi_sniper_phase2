use pumpfun_sniper::*;
use futures::StreamExt;
use yellowstone_grpc_proto::geyser::{
    SubscribeRequestFilterTransactions,
    subscribe_update::UpdateOneof,
};

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    info!("{}", "[SIMULATION MODE]".cyan());
    info!("No real trades — pattern backtesting only\n");

    // Load manual patterns
    let manual_count = get_manual_patterns().len();
    info!("Loaded {} manual pattern(s)", manual_count);

    // Start pattern server to receive server-pushed patterns
    tokio::spawn(async {
        run_pattern_server(3356).await;
    });

    let engine = SimEngine::new();

    info!(
        "Config: buy_amount={} SOL | SL={:.0}% | tp_trailing={} | trailing_stop={} | confirmation=150ms",
        engine.buy_amount_sol,
        engine.stop_loss_pct * 100.0,
        engine.tp_trailing,
        engine.trailing_stop,
    );

    let grpc_config = GrpcClientConfig::new(
        "simulation".to_string(),
        GRPC_ENDPOINT.to_string(),
        GRPC_TOKEN.to_string(),
    );

    let subscribe_filter = SubscribeRequestFilterTransactions {
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

    info!("Connecting to gRPC stream...");
    info!("Press Ctrl+C to stop and generate report\n");

    tokio::select! {
        result = run_simulation_stream(&grpc_config, &subscribe_filter, &engine) => {
            if let Err(e) = result {
                error!("[SIM] Stream ended: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("\n[SIM] Ctrl+C received — generating report...");
        }
    }

    // Finalize and generate report
    engine.finalize();

    let report = generate_report(&engine);
    println!("\n{}", report);

    let path = save_report(&report);
    if !path.is_empty() {
        info!("[SIM] Report saved: {}", path);
    }
}

async fn run_simulation_stream(
    grpc_config: &GrpcClientConfig,
    subscribe_filter: &SubscribeRequestFilterTransactions,
    engine: &SimEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut grpc_client = grpc_config.setup_grpc_client().await?;
    let (subscribe_tx, subscribe_rx) = grpc_client.subscribe().await?;
    send_subscription_request_grpc(subscribe_tx, subscribe_filter.clone()).await?;

    info!("[SIM] Connected to gRPC — monitoring live transactions\n");

    let mut stream = subscribe_rx;

    while let Some(result) = stream.next().await {
        match result {
            Ok(update) => {
                let (account_keys, ixs, inner_ixs, tx_id, _signers) =
                    if let Some(data) = extract_transaction_data(&update) {
                        data
                    } else {
                        continue;
                    };

                let mut grouped = group_by_program_ids(
                    ixs,
                    inner_ixs,
                    &[BUDGET_COMPUTE_PROGRAM, PUMPFUN_PROGRAM_ID, PUMPSWAP_PROGRAM_ID],
                    &account_keys,
                );
                let ix_info_pumpswap = grouped.pop().unwrap();
                let ix_info_pumpfun = grouped.pop().unwrap();
                let budget_compute_ix_info = grouped.pop().unwrap();

                let mut all_pump_ix = Vec::with_capacity(ix_info_pumpfun.len() + ix_info_pumpswap.len());
                all_pump_ix.extend(ix_info_pumpfun.clone());
                all_pump_ix.extend(ix_info_pumpswap.clone());

                let transaction_update = match &update.update_oneof {
                    Some(UpdateOneof::Transaction(tx_update)) => {
                        match tx_update.transaction.as_ref() {
                            Some(tx) => tx,
                            None => continue,
                        }
                    }
                    _ => continue,
                };

                let budget_compute_data = get_budget_compute_info(budget_compute_ix_info);
                let pumpfun_trade_data =
                    get_pumpfun_trade_info(ix_info_pumpfun.clone(), account_keys.clone(), transaction_update);
                let migration_data = migrate_info(all_pump_ix.clone(), account_keys.clone());
                let pumpswap_trade_data =
                    get_pumpswap_trade_info(ix_info_pumpswap.clone(), account_keys.clone());

                engine.process_transaction(
                    budget_compute_data,
                    &pumpfun_trade_data,
                    &migration_data,
                    &pumpswap_trade_data,
                    &tx_id,
                );
            }
            Err(e) => {
                error!("[SIM] Stream error: {}", e);
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}
