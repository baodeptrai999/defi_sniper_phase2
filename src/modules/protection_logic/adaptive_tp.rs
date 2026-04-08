use crate::*;
use once_cell::sync::Lazy;
use solana_sdk::pubkey::Pubkey;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct LiveAthTracker {
    pattern_label: String,
    buy_price: f64,
    max_price: f64,
    started_at: Instant,
}

struct LiveEmaState {
    ema_tp: f64,
    update_count: u64,
}

pub struct AdaptiveTpEngine {
    ath_trackers: Mutex<HashMap<Pubkey, LiveAthTracker>>,
    ema_state: Mutex<HashMap<String, LiveEmaState>>,
    avg_history: Mutex<HashMap<String, VecDeque<f64>>>,
}

pub static ADAPTIVE_TP: Lazy<AdaptiveTpEngine> = Lazy::new(|| AdaptiveTpEngine::new());

impl AdaptiveTpEngine {
    fn new() -> Self {
        Self {
            ath_trackers: Mutex::new(HashMap::new()),
            ema_state: Mutex::new(HashMap::new()),
            avg_history: Mutex::new(HashMap::new()),
        }
    }

    pub fn is_adaptive() -> bool {
        *TP_MODE == "EMA" || *TP_MODE == "AVERAGE"
    }

    /// Start tracking a token's ATH after buy confirmation.
    pub fn start_tracking(&self, mint: Pubkey, pattern_label: String, buy_price: f64, current_price: f64) {
        if !Self::is_adaptive() {
            return;
        }
        let mut trackers = self.ath_trackers.lock().expect("ath_trackers lock");
        trackers.insert(mint, LiveAthTracker {
            pattern_label,
            buy_price,
            max_price: current_price,
            started_at: Instant::now(),
        });
    }

    /// Update ATH for a tracked token on every price update.
    pub fn update_ath(&self, mint: &Pubkey, new_price: f64) {
        if !Self::is_adaptive() {
            return;
        }
        let mut trackers = self.ath_trackers.lock().expect("ath_trackers lock");
        if let Some(tracker) = trackers.get_mut(mint) {
            tracker.max_price = tracker.max_price.max(new_price);
        }
    }

    /// Get adaptive TP for a pattern (as percentage, e.g. 150.0 = 1.5x).
    /// EMA mode: α × newest_peak + (1-α) × second_newest_peak from data series.
    /// AVERAGE mode: average of recent N tokens' ATH peaks.
    pub fn get_adaptive_tp(&self, pattern_label: &str, initial_tp: f64) -> f64 {
        let initial_mult = initial_tp / 100.0;

        let trackers = self.ath_trackers.lock().expect("ath_trackers lock");
        let mut active_peaks: Vec<(Instant, f64)> = trackers
            .values()
            .filter(|t| t.pattern_label == pattern_label && t.buy_price > 0.0)
            .map(|t| (t.started_at, t.max_price / t.buy_price))
            .collect();
        active_peaks.sort_by_key(|(started, _)| *started);

        match TP_MODE.as_str() {
            "EMA" => {
                let mut state = self.ema_state.lock().expect("ema_state lock");
                let entry = state.entry(pattern_label.to_string()).or_insert(LiveEmaState {
                    ema_tp: initial_mult,
                    update_count: 0,
                });
                let finalized = entry.ema_tp;

                let mut series: Vec<f64> = vec![finalized];
                for (_, peak) in active_peaks.iter() {
                    series.push(*peak);
                }

                let live_ema = if series.len() >= 2 {
                    let newest = series[series.len() - 1];
                    let second = series[series.len() - 2];
                    *EMA_ALPHA * newest + (1.0 - *EMA_ALPHA) * second
                } else {
                    finalized
                };

                live_ema * 100.0
            }
            "AVERAGE" => {
                let history = self.avg_history.lock().expect("avg_history lock");
                let finalized = history.get(pattern_label);

                let mut all_peaks: Vec<f64> = match finalized {
                    Some(q) => q.iter().copied().collect(),
                    None => Vec::new(),
                };
                for (_, peak) in active_peaks.iter() {
                    all_peaks.push(*peak);
                }

                if all_peaks.is_empty() {
                    initial_tp
                } else {
                    let window = *AVERAGE_WINDOW;
                    let start = if all_peaks.len() > window { all_peaks.len() - window } else { 0 };
                    let recent = &all_peaks[start..];
                    let avg = recent.iter().sum::<f64>() / recent.len() as f64;
                    avg * 100.0
                }
            }
            _ => initial_tp,
        }
    }

    /// Expire ATH trackers older than 1 hour — fold into EMA or average history.
    pub fn expire_trackers(&self) {
        if !Self::is_adaptive() {
            return;
        }
        let one_hour = Duration::from_secs(3600);
        let mut trackers = self.ath_trackers.lock().expect("ath_trackers lock");
        let expired: Vec<Pubkey> = trackers
            .iter()
            .filter(|(_, t)| t.started_at.elapsed() >= one_hour)
            .map(|(k, _)| *k)
            .collect();

        if expired.is_empty() {
            return;
        }

        match TP_MODE.as_str() {
            "EMA" => {
                let mut ema_state = self.ema_state.lock().expect("ema_state lock");
                for mint in expired {
                    if let Some(tracker) = trackers.remove(&mint) {
                        if tracker.buy_price > 0.0 {
                            let peak_multiple = tracker.max_price / tracker.buy_price;
                            let entry = ema_state
                                .entry(tracker.pattern_label.clone())
                                .or_insert(LiveEmaState {
                                    ema_tp: peak_multiple,
                                    update_count: 0,
                                });
                            let old_ema = entry.ema_tp;
                            entry.ema_tp = *EMA_ALPHA * peak_multiple + (1.0 - *EMA_ALPHA) * old_ema;
                            entry.update_count += 1;
                            info!(
                                "\n📊 [LIVE] [EMA_UPDATE]\n\
                                 │  Pattern:      {}\n\
                                 │  Mint:         {}\n\
                                 │  Peak mult:    {:.3}x\n\
                                 │  EMA:          {:.3}x → {:.3}x\n\
                                 │  Updates:      {}\n\
                                 └──────────────────────",
                                tracker.pattern_label, mint,
                                peak_multiple,
                                old_ema, entry.ema_tp,
                                entry.update_count,
                            );
                        }
                    }
                }
            }
            "AVERAGE" => {
                let mut avg = self.avg_history.lock().expect("avg_history lock");
                for mint in expired {
                    if let Some(tracker) = trackers.remove(&mint) {
                        if tracker.buy_price > 0.0 {
                            let peak_multiple = tracker.max_price / tracker.buy_price;
                            let q = avg.entry(tracker.pattern_label.clone()).or_insert_with(VecDeque::new);
                            q.push_back(peak_multiple);
                            while q.len() > *AVERAGE_WINDOW {
                                q.pop_front();
                            }
                            let current_avg: f64 = q.iter().sum::<f64>() / q.len() as f64;
                            info!(
                                "\n📊 [LIVE] [AVG_UPDATE]\n\
                                 │  Pattern:      {}\n\
                                 │  Mint:         {}\n\
                                 │  Peak mult:    {:.3}x\n\
                                 │  Window avg:   {:.3}x (n={})\n\
                                 └──────────────────────",
                                tracker.pattern_label, mint,
                                peak_multiple,
                                current_avg, q.len(),
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
