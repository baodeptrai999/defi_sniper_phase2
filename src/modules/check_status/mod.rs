use crate::*;
use colored::*;
use reqwest::Client;
use std::io::{self, Write};
use std::time::Instant;

pub async fn check_endpoint_status() {
    println!();
    println!(
        "  {}",
        "─── Endpoint Status ───".cyan().bold()
    );

    // 1. RPC
    print!("  {} RPC ({}) ... ", "🔗", mask_url(&RPC_ENDPOINT));
    io::stdout().flush().unwrap();
    let start = Instant::now();
    match RPC_CLIENT.get_version().await {
        Ok(v) => println!(
            "{} v{} ({:?})",
            "OK".green().bold(),
            v.solana_core,
            start.elapsed()
        ),
        Err(e) => println!("{} {}", "FAIL".red().bold(), e),
    }

    // 2. gRPC
    print!(
        "  {} gRPC ({}) ... ",
        "🔗",
        mask_url(&GRPC_ENDPOINT)
    );
    io::stdout().flush().unwrap();
    let start = Instant::now();
    match yellowstone_grpc_client::GeyserGrpcClient::build_from_shared(
        GRPC_ENDPOINT.clone(),
    ) {
        Ok(mut builder) => {
            if !GRPC_TOKEN.is_empty() {
                builder = builder.x_token(Some(GRPC_TOKEN.clone())).unwrap();
            }
            builder = builder
                .tls_config(yellowstone_grpc_client::ClientTlsConfig::new().with_native_roots())
                .unwrap();
            match builder.connect().await {
                Ok(_) => println!("{} ({:?})", "OK".green().bold(), start.elapsed()),
                Err(e) => println!("{} {}", "FAIL".red().bold(), e),
            }
        }
        Err(e) => println!("{} {}", "FAIL".red().bold(), e),
    }

    // 3. Zero-Slot landing service
    print!(
        "  {} Zero-Slot ({}) ... ",
        "🔗",
        mask_url(&ZERO_SLOT_ENDPOINT)
    );
    io::stdout().flush().unwrap();
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let start = Instant::now();
    match client.get(&*ZERO_SLOT_ENDPOINT).send().await {
        Ok(resp) => println!(
            "{} HTTP {} ({:?})",
            if resp.status().is_success() {
                "OK".green().bold()
            } else {
                "WARN".yellow().bold()
            },
            resp.status(),
            start.elapsed()
        ),
        Err(e) => println!("{} {}", "FAIL".red().bold(), e),
    }

    // 4. Helius landing service
    print!(
        "  {} Helius ({}) ... ",
        "🔗",
        mask_url(&HELIUS_ENDPOINT)
    );
    io::stdout().flush().unwrap();
    let start = Instant::now();
    match client.get(&*HELIUS_ENDPOINT).send().await {
        Ok(resp) => println!(
            "{} HTTP {} ({:?})",
            if resp.status().is_success() {
                "OK".green().bold()
            } else {
                "WARN".yellow().bold()
            },
            resp.status(),
            start.elapsed()
        ),
        Err(e) => println!("{} {}", "FAIL".red().bold(), e),
    }

    // 5. Wallet balance
    print!("  {} Wallet balance ... ", "💰");
    io::stdout().flush().unwrap();
    match RPC_CLIENT.get_balance(&*SIGNER_PUBKEY).await {
        Ok(bal) => println!(
            "{:.6} SOL",
            bal as f64 / 1e9
        ),
        Err(e) => println!("{} {}", "FAIL".red().bold(), e),
    }

    println!(
        "  {}",
        "───────────────────────".cyan()
    );
}

fn mask_url(url: &str) -> String {
    if let Some(idx) = url.find("api-key=") {
        let key_start = idx + 8;
        if url.len() > key_start + 6 {
            format!("{}{}...", &url[..key_start], &url[key_start..key_start + 6])
        } else {
            url.to_string()
        }
    } else if url.len() > 40 {
        format!("{}...", &url[..40])
    } else {
        url.to_string()
    }
}
