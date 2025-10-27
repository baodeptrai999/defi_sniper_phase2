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
            // Send events into async channel, blocking send OK inside sync closure
            tokio::runtime::Handle::current()
                .block_on(async {
                    tx.send(res).await.unwrap();
                });
        },
        notify::Config::default(),
    )?;

    // Watch the specific file
    watcher.watch(&file_path, RecursiveMode::NonRecursive)?;

    // Initial load of the blacklist
    {
        let content = tokio::fs::read_to_string(&file_path).await.expect("Failed to read file initially");
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).unique().collect();
        let mut wl = WALLET_BLACKLIST.write().await;
        *wl = lines;
    }

    // Process file change events asynchronously
    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                if event.kind.is_modify() {
                    // On any modify, reload the file and update blacklist
                    let content = tokio::fs::read_to_string(&file_path).await.expect("Failed to read file updated");
                    let new_lines: Vec<String> = content.lines().map(|l| l.to_string()).unique().collect();

                    let mut wl = WALLET_BLACKLIST.write().await;
                    wl.extend(new_lines);
                    wl.sort_unstable();
                    wl.dedup();
                }
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }

    Ok(())
}

pub async fn watch_token_blacklist_file(file_path: PathBuf) -> Result<()> {
    if !*TOKEN_BLACK_LIST_FILTER {
        return Ok(()); // feature disabled
    }

    // Channel to receive file events asynchronously
    let (tx, mut rx) = mpsc::channel(100);

    // Create a Notify watcher in a blocking task
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event>| {
            // Send events into async channel, blocking send OK inside sync closure
            tokio::runtime::Handle::current()
                .block_on(async {
                    tx.send(res).await.unwrap();
                });
        },
        notify::Config::default(),
    )?;

    // Watch the specific file
    watcher.watch(&file_path, RecursiveMode::NonRecursive)?;

    // Initial load of the blacklist
    {
        let content = tokio::fs::read_to_string(&file_path).await.expect("Failed to read file initially");
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).unique().collect();
        let mut wl = WALLET_BLACKLIST.write().await;
        *wl = lines;
    }

    // Process file change events asynchronously
    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                if event.kind.is_modify() {
                    // On any modify, reload the file and update blacklist
                    let content = tokio::fs::read_to_string(&file_path).await.expect("Failed to read file updated");
                    let new_lines: Vec<String> = content.lines().map(|l| l.to_string()).unique().collect();

                    let mut wl = WALLET_BLACKLIST.write().await;
                    wl.extend(new_lines);
                    wl.sort_unstable();
                    wl.dedup();
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
