/// Phase 2 — Filter Aggregator (Nâng cao)
///
/// Orchestrates all pre-buy filter modules and combines their results
/// into a unified buy/skip decision with dynamic position sizing.
///
/// Aggregation logic:
///   1. Any module returning `passed = false` → HARD REJECT
///   2. Total risk_score > MAX_TOTAL_RISK_SCORE → SOFT REJECT
///   3. Otherwise → ALLOW BUY, with buy_amount_multiplier based on risk
///
/// Dynamic Position Sizing:
///   - Risk = 0 → multiplier = 1.0 (full position)
///   - Risk > 0 → multiplier = max(min_buy_multiplier, 1.0 - risk/max_risk)
///   - This means higher risk → smaller position size

use crate::*;

// ══════════════════════════════════════════════════════════════════════
// Main aggregation function
// ══════════════════════════════════════════════════════════════════════

/// Run all enabled filter modules against a newly minted token.
///
/// This is the PRIMARY entry point called from handle_sniper_event.rs.
/// It takes a FilterContext (built from mint event data) and returns
/// an AggregatedFilterResult with the final buy/skip decision.
pub async fn run_pre_buy_filters(ctx: &FilterContext) -> AggregatedFilterResult {
    let mut results: Vec<FilterResult> = Vec::with_capacity(3);

    // ── Module 1: Genesis Bundle Detection ──
    // NOTE: genesis_check() is NOT called here. It is called SEPARATELY
    // from handle_sniper_event AFTER all buy events in this TX have been
    // recorded. This avoids the race condition where genesis data hasn't
    // been populated yet at mint-time.
    // See: handle_sniper_event.rs → deferred genesis check section.

    // ── Module 2 + 3: Run CONCURRENTLY for lower latency ──
    // ── Check Telegram Toggles ──
    let enable_metadata = std::sync::atomic::Ordering::Relaxed;
    let run_metadata = ENABLE_M5_METADATA.load(enable_metadata);
    let run_wallet = ENABLE_M3_DEV.load(enable_metadata);

    // ── Module 2 + 3: Run CONCURRENTLY for lower latency ──
    let (metadata_result_opt, wallet_result_opt) = tokio::join!(
        async { if run_metadata { Some(check_metadata(ctx).await) } else { None } },
        async { if run_wallet { Some(profile_dev_wallet(ctx.creator).await) } else { None } },
    );

    if let Some(res) = metadata_result_opt { results.push(res); }
    if let Some(res) = wallet_result_opt { results.push(res); }

    // ── Aggregate results ──
    let warn_only = WARN_ONLY_MODE.load(enable_metadata);
    let any_hard_fail = results.iter().any(|r| !r.passed);
    let total_risk: f64 = results.iter().map(|r| r.risk_score).sum::<f64>().max(0.0);

    let mut should_buy = !any_hard_fail && total_risk < *MAX_TOTAL_RISK_SCORE;
    if warn_only {
        should_buy = true;
    }

    if !crate::BOT_IS_RUNNING.load(std::sync::atomic::Ordering::Relaxed) {
        should_buy = false;
        results.push(FilterResult {
            module_name: "BOT_CONTROL".to_string(),
            passed: false,
            risk_score: 100.0,
            reason: "Bot is manually STOPPED from Telegram".to_string(),
        });
    }

    // Dynamic position sizing based on risk
    let buy_amount_multiplier = if should_buy && *ENABLE_DYNAMIC_SIZING && total_risk > 0.0 {
        let raw_multiplier = 1.0 - (total_risk / *MAX_TOTAL_RISK_SCORE);
        raw_multiplier.max(*MIN_BUY_MULTIPLIER).min(1.0)
    } else if should_buy {
        1.0
    } else {
        0.0 // rejected
    };

    let aggregated = AggregatedFilterResult {
        should_buy,
        total_risk_score: total_risk,
        buy_amount_multiplier,
        results,
    };

    // ── Log the decision ──
    crate::STAT_SCANNED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if should_buy {
        if total_risk > 0.0 {
            crate::STAT_WARNED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            info!(
                "🟡 [FILTER_PASS_WARN] MINT: {} | creator: {} | name: '{}' | risk: {:.1} | mul: {:.2}x | {}",
                ctx.mint,
                ctx.creator,
                ctx.name,
                total_risk,
                buy_amount_multiplier,
                aggregated.rejection_summary(),
            );
        } else {
            crate::STAT_PASSED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            info!(
                "🟢 [FILTER_PASS] MINT: {} | creator: {} | name: '{}' | CLEAN",
                ctx.mint, ctx.creator, ctx.name,
            );
        }
    } else {
        crate::STAT_REJECTED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        info!(
            "🔴 [FILTER_REJECT] MINT: {} | creator: {} | name: '{}' | risk: {:.1} | {}",
            ctx.mint,
            ctx.creator,
            ctx.name,
            total_risk,
            aggregated.rejection_summary(),
        );
    }

    // ── Log to CSV audit trail ──
    log_filter_result(ctx, &aggregated);

    // ── Send Telegram notification ──
    if crate::BOT_IS_RUNNING.load(std::sync::atomic::Ordering::Relaxed) {
        let reasons: Vec<String> = aggregated.results.iter()
            .filter(|r| !r.passed || r.risk_score > 0.0)
            .map(|r| format!("[{}] {}", r.module_name, r.reason))
            .collect();

        tg_send_filter_result(
            &ctx.mint.to_string(),
            &ctx.creator.to_string(),
            &ctx.name,
            should_buy,
            total_risk,
            buy_amount_multiplier,
            &reasons,
        );
    }

    aggregated
}
