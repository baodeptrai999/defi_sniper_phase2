use console::Emoji;
use itertools::Itertools;
use std::path::PathBuf;
use tokio::time::{Duration, sleep};
use notify::{Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use tokio::sync::{mpsc};

use crate::*;

pub async fn watch_wallet_blacklist_file(file_path: PathBuf) -> Result<()> {
    if !*TOKEN_BLACK_LIST_FILTER || !*HOLDER_BLACK_LIST_FILTER {
        return Ok(()); // feature disabled
    }

    // Channel to receive file events asynchronously
    let (tx, mut rx) = mpsc::channel(100);

    // Create a Notify watcher in a blocking task
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event>| {
            // Use blocking_send instead of trying to use async runtime
            if let Err(e) = tx.blocking_send(res) {
                eprintln!("Failed to send filesystem event: {}", e);
            }
        },
        notify::Config::default(),
    )?;

    // Watch the specific file
    watcher.watch(&file_path, RecursiveMode::NonRecursive)?;

    // Initial load of the blacklist
    {
        let content = tokio::fs::read_to_string(&file_path).await
            .expect("Failed to read file initially");
        let lines: Vec<String> = content.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .unique()
            .collect();
        let mut wl = WALLET_BLACKLIST.write().await;
        *wl = lines;
    }

    // Process file change events asynchronously
    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                if event.kind.is_modify() {
                    // Small delay to ensure file write is complete
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    
                    match tokio::fs::read_to_string(&file_path).await {
                        Ok(content) => {
                            let new_lines: Vec<String> = content.lines()
                                .map(|l| l.trim().to_string())
                                .filter(|l| !l.is_empty())
                                .unique()
                                .collect();

                            let mut wl = WALLET_BLACKLIST.write().await;
                            *wl = new_lines;
                            info!("Updated wallet blacklist with {} entries", wl.len());
                        }
                        Err(e) => {
                            eprintln!("Failed to read updated file: {}", e);
                        }
                    }
                }
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }

    Ok(())
}

pub async fn watch_token_blacklist_file(file_path: PathBuf) -> Result<()> {
    if !*TOKEN_BLACK_LIST_FILTER || !*HOLDER_BLACK_LIST_FILTER {
        return Ok(()); // feature disabled
    }

    // Channel to receive file events asynchronously
    let (tx, mut rx) = mpsc::channel(100);

    // Create a Notify watcher in a blocking task
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event>| {
            // Use blocking_send instead of trying to use async runtime
            if let Err(e) = tx.blocking_send(res) {
                eprintln!("Failed to send filesystem event: {}", e);
            }
        },
        notify::Config::default(),
    )?;

    // Watch the specific file
    watcher.watch(&file_path, RecursiveMode::NonRecursive)?;

    // Initial load of the blacklist
    {
        let content = tokio::fs::read_to_string(&file_path).await
            .expect("Failed to read file initially");
        let lines: Vec<String> = content.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .unique()
            .collect();
        let mut wl = TOKEN_BLACKLIST.write().await;
        *wl = lines;
    }

    // Process file change events asynchronously
    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                if event.kind.is_modify() {
                    // Small delay to ensure file write is complete
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    
                    match tokio::fs::read_to_string(&file_path).await {
                        Ok(content) => {
                            let new_lines: Vec<String> = content.lines()
                                .map(|l| l.trim().to_string())
                                .filter(|l| !l.is_empty())
                                .unique()
                                .collect();

                            let mut wl = TOKEN_BLACKLIST.write().await;
                            *wl = new_lines;
                            info!("Updated wallet blacklist with {} entries", wl.len());
                        }
                        Err(e) => {
                            eprintln!("Failed to read updated file: {}", e);
                        }
                    }
                }
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }

    Ok(())
}

pub async fn show_blacklist_length() {
    if *HOLDER_BLACK_LIST_FILTER || *TOKEN_BLACK_LIST_FILTER {
        let wallet_blacklist = WALLET_BLACKLIST.read().await;
        let token_blacklist = TOKEN_BLACKLIST.read().await;
        log!(
            "\t[ {} Loaded ]\t\t{} blacked wallets\t\t{} blacked tokens.",
            Emoji("💳", ""),
            wallet_blacklist.len(),
            token_blacklist.len()
        );
        sleep(Duration::from_millis(10000)).await;
    }
}
