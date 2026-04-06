use super::sim_engine::{SimEngine, SimOutcome, SimToken};
use crate::constants::constants::PUMP_FUN_TOKEN_TOTAL_SUPPLY;
use chrono::Local;
use std::collections::HashMap;
use std::fs;

pub fn generate_report(engine: &SimEngine) -> String {
    let results = engine.get_results();
    let pattern_counts = engine.get_pattern_counts();
    let total_tx = engine.get_total_tx();
    let elapsed = engine.get_elapsed();


    let now = Local::now();
    let date_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let bought_tokens: Vec<&SimToken> = results.iter().filter(|t| t.buy_confirmed).collect();
    let unbought: Vec<&SimToken> = results.iter().filter(|t| !t.buy_confirmed).collect();

    let wins: Vec<&&SimToken> = bought_tokens.iter().filter(|t| t.pnl_pct > 0.0).collect();
    let losses: Vec<&&SimToken> = bought_tokens
        .iter()
        .filter(|t| t.pnl_pct <= 0.0)
        .collect();

    let total_bought = bought_tokens.len();
    let win_count = wins.len();
    let loss_count = losses.len();
    let win_rate = if total_bought > 0 {
        (win_count as f64 / total_bought as f64) * 100.0
    } else {
        0.0
    };

    let total_pnl_pct: f64 = bought_tokens.iter().map(|t| t.pnl_pct).sum();
    let avg_pnl = if total_bought > 0 {
        total_pnl_pct / total_bought as f64
    } else {
        0.0
    };

    let avg_win = if win_count > 0 {
        wins.iter().map(|t| t.pnl_pct).sum::<f64>() / win_count as f64
    } else {
        0.0
    };
    let avg_loss = if loss_count > 0 {
        losses.iter().map(|t| t.pnl_pct).sum::<f64>() / loss_count as f64
    } else {
        0.0
    };

    let best_trade = bought_tokens
        .iter()
        .max_by(|a, b| a.pnl_pct.partial_cmp(&b.pnl_pct).unwrap_or(std::cmp::Ordering::Equal));
    let worst_trade = bought_tokens
        .iter()
        .min_by(|a, b| a.pnl_pct.partial_cmp(&b.pnl_pct).unwrap_or(std::cmp::Ordering::Equal));

    let buy_sol = engine.buy_amount_sol;
    let total_invested = total_bought as f64 * buy_sol;
    let total_returned: f64 = bought_tokens
        .iter()
        .map(|t| buy_sol * (1.0 + t.pnl_pct / 100.0))
        .sum();
    let net_profit_sol = total_returned - total_invested;
    let roi = if total_invested > 0.0 {
        (net_profit_sol / total_invested) * 100.0
    } else {
        0.0
    };

    let elapsed_min = elapsed.as_secs() / 60;
    let elapsed_sec = elapsed.as_secs() % 60;

    // ── Build report ──
    let mut report = String::new();

    let bar = "═".repeat(70);
    let thin = "─".repeat(70);

    report.push_str(&format!("{}\n", bar));
    report.push_str(&format!(
        "  SIMULATION REPORT  ·  {}\n",
        date_str
    ));
    report.push_str(&format!("{}\n\n", bar));

    report.push_str(&format!("  Runtime:              {} min {} sec\n", elapsed_min, elapsed_sec));
    report.push_str(&format!("  Transactions:         {}\n", total_tx));
    report.push_str(&format!("  Buy Amount:           {} SOL\n\n", buy_sol));

    // ── Overall stats ──
    report.push_str(&format!("{}\n", thin));
    report.push_str("  OVERALL STATISTICS\n");
    report.push_str(&format!("{}\n\n", thin));

    report.push_str(&format!("  Total Matched:        {}\n", results.len()));
    report.push_str(&format!("  Total Bought:         {}\n", total_bought));
    report.push_str(&format!("  Not Confirmed:        {}\n", unbought.len()));

    let holding_count = bought_tokens.iter().filter(|t| t.outcome == SimOutcome::Timeout).count();
    let tp_count_all = bought_tokens.iter().filter(|t| t.outcome == SimOutcome::TpHit).count();
    let sl_count_all = bought_tokens.iter().filter(|t| t.outcome == SimOutcome::SlHit).count();
    let partial_count_all = bought_tokens.iter().filter(|t| t.outcome == SimOutcome::Partial).count();

    report.push_str(&format!("  Wins:                 {}\n", win_count));
    report.push_str(&format!("  Losses:               {}\n", loss_count));
    report.push_str(&format!("  Win Rate:             {:.2}%\n", win_rate));
    report.push_str(&format!("  Outcomes:             TP: {} | SL: {} | Partial: {} | Holding: {}\n\n", tp_count_all, sl_count_all, partial_count_all, holding_count));

    report.push_str(&format!("  Total Invested:       {:.6} SOL\n", total_invested));
    report.push_str(&format!("  Total Returned:       {:.6} SOL\n", total_returned));
    report.push_str(&format!("  Net Profit:           {:.6} SOL\n", net_profit_sol));
    report.push_str(&format!("  ROI:                  {:.2}%\n\n", roi));

    report.push_str(&format!("  Avg P&L per trade:    {:.2}%\n", avg_pnl));
    report.push_str(&format!("  Avg Win:              {:.2}%\n", avg_win));
    report.push_str(&format!("  Avg Loss:             {:.2}%\n\n", avg_loss));

    if let Some(best) = best_trade {
        report.push_str(&format!(
            "  Best Trade:           {} | {:.2}% | {} | {}\n",
            best.mint, best.pnl_pct, best.pattern_label, best.outcome
        ));
    }
    if let Some(worst) = worst_trade {
        report.push_str(&format!(
            "  Worst Trade:          {} | {:.2}% | {} | {}\n\n",
            worst.mint, worst.pnl_pct, worst.pattern_label, worst.outcome
        ));
    }

    // ── Per-pattern breakdown ──
    report.push_str(&format!("{}\n", thin));
    report.push_str("  PER-PATTERN BREAKDOWN\n");
    report.push_str(&format!("{}\n\n", thin));

    let mut pattern_groups: HashMap<String, Vec<&SimToken>> = HashMap::new();
    for token in results.iter() {
        pattern_groups
            .entry(token.pattern_label.clone())
            .or_default()
            .push(token);
    }

    let mut patterns_sorted: Vec<_> = pattern_groups.into_iter().collect();
    patterns_sorted.sort_by_key(|(label, _)| label.clone());

    for (label, tokens) in patterns_sorted.iter() {
        let matched = pattern_counts.get(label.as_str()).copied().unwrap_or(tokens.len() as u64);
        let bought: Vec<&&SimToken> = tokens.iter().filter(|t| t.buy_confirmed).collect();
        let p_wins: Vec<&&&SimToken> = bought.iter().filter(|t| t.pnl_pct > 0.0).collect();
        let p_losses: Vec<&&&SimToken> = bought.iter().filter(|t| t.pnl_pct <= 0.0).collect();
        let p_total = bought.len();
        let p_wr = if p_total > 0 {
            (p_wins.len() as f64 / p_total as f64) * 100.0
        } else {
            0.0
        };

        let p_pnl: f64 = bought.iter().map(|t| t.pnl_pct).sum();
        let p_avg = if p_total > 0 { p_pnl / p_total as f64 } else { 0.0 };

        let p_invested = p_total as f64 * buy_sol;
        let p_returned: f64 = bought
            .iter()
            .map(|t| buy_sol * (1.0 + t.pnl_pct / 100.0))
            .sum();
        let p_net = p_returned - p_invested;

        report.push_str(&format!("  ┌── {} ──\n", label));
        report.push_str(&format!("  │  Matched:     {}  |  Bought: {}  |  Wins: {} | Losses: {}\n",
            matched, p_total, p_wins.len(), p_losses.len()));
        report.push_str(&format!("  │  Win Rate:    {:.2}%  |  Avg PnL: {:.2}%\n", p_wr, p_avg));
        report.push_str(&format!("  │  Invested:    {:.6} SOL  |  Net: {:.6} SOL\n", p_invested, p_net));

        // Outcome distribution
        let tp_count = tokens.iter().filter(|t| t.outcome == SimOutcome::TpHit).count();
        let sl_count = tokens.iter().filter(|t| t.outcome == SimOutcome::SlHit).count();
        let partial_count = tokens.iter().filter(|t| t.outcome == SimOutcome::Partial).count();
        let timeout_count = tokens.iter().filter(|t| t.outcome == SimOutcome::Timeout).count();

        report.push_str(&format!(
            "  │  Outcomes:    TP: {} | SL: {} | Partial: {} | Holding: {}\n",
            tp_count, sl_count, partial_count, timeout_count
        ));
        report.push_str("  └──\n\n");
    }

    // ── Detailed Token Log ──
    report.push_str(&format!("{}\n", thin));
    report.push_str("  DETAILED TOKEN LOG\n");
    report.push_str(&format!("{}\n\n", thin));

    // Sort by outcome: TP first, then Partial, then SL, then Timeout
    let mut sorted_results = results.clone();
    sorted_results.sort_by(|a, b| {
        let order = |o: &SimOutcome| match o {
            SimOutcome::TpHit => 0,
            SimOutcome::Partial => 1,
            SimOutcome::SlHit => 2,
            SimOutcome::Timeout => 3,
            SimOutcome::Pending => 4,
        };
        order(&a.outcome).cmp(&order(&b.outcome))
    });

    for (i, token) in sorted_results.iter().enumerate() {
        report.push_str(&format!("  #{} ─ {}\n", i + 1, token.mint));
        report.push_str(&format!("    Pattern:     {}\n", token.pattern_label));
        report.push_str(&format!("    Outcome:     {}\n", token.outcome));

        if token.buy_confirmed {
            let supply = PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
            report.push_str(&format!("    Buy MC:      {:.2} SOL\n", token.buy_price * supply));
            report.push_str(&format!("    Exit MC:     {:.2} SOL\n", token.exit_price * supply));
            report.push_str(&format!("    Max MC:      {:.2} SOL\n", token.max_price * supply));

            let max_potential = if token.buy_price > 0.0 {
                (token.max_price / token.buy_price - 1.0) * 100.0
            } else {
                0.0
            };
            report.push_str(&format!("    Max Gain:    {:.2}%\n", max_potential));
            report.push_str(&format!("    P&L:         {:.2}%\n", token.pnl_pct));
            report.push_str(&format!("    SOL P&L:     {:.6} SOL\n", buy_sol * token.pnl_pct / 100.0));

            if !token.tp_triggered_at.is_empty() {
                let supply = PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64;
                let tp_strs: Vec<String> = token
                    .tp_triggered_at
                    .iter()
                    .enumerate()
                    .map(|(i, p)| format!("TP{}: {:.2} SOL", i + 1, p * supply))
                    .collect();
                report.push_str(&format!("    TP Hits:     {}\n", tp_strs.join(" | ")));
            }
        }

        report.push_str(&format!("    Tx Count:    {}\n", token.tx_count));
        report.push_str(&format!("    Migrated:    {}\n", token.is_migrated));
        report.push_str(&format!("    Reason:      {}\n", token.exit_reason));
        report.push_str("\n");
    }

    report.push_str(&format!("{}\n", bar));
    report.push_str("  END OF REPORT\n");
    report.push_str(&format!("{}\n", bar));

    report
}

pub fn save_report(report: &str) -> String {
    let now = Local::now();
    let filename = format!("simulation_{}_{}.txt", now.format("%m"), now.format("%d"));
    let path = format!("src/assets/reports/{}", filename);

    if let Some(parent) = std::path::Path::new(&path).parent() {
        let _ = fs::create_dir_all(parent);
    }

    match fs::write(&path, report) {
        Ok(_) => {
            println!("Report saved to: {}", path);
            path
        }
        Err(e) => {
            eprintln!("Failed to save report: {}", e);
            String::new()
        }
    }
}
