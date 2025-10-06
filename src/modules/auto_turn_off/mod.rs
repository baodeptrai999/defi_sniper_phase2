use chrono::Local;
use std::fs::{File, create_dir_all};
use std::io::Write;

use crate::{SHUT_DOWN_TIME, SHUT_DOWN_TIMER_SELL_ALL, TOKEN_DB, info};

pub fn check_auto_turn_off_time(mode: &str) -> bool {
    let now = Local::now();
    create_dir_all("src/assets/panel").unwrap_or(());

    let dir_path = format!("src/assets/panel/monitor_token.{}", mode);
    let mut file = File::create(dir_path).expect("Unable to create file");
    let lists = TOKEN_DB.get_list_all().unwrap();

    let result_string = lists
        .iter()
        .enumerate()
        .map(|(idx, ele)| {
            format!(
                "{:<3} | {:<46} | {:<12.4e} | {:<15.4e} | {:<15.4e} | {:<12} |  {:<12} | {:<12.2}",
                idx + 1,
                ele.0,
                ele.1.token_price,
                ele.1.token_peak_price,
                ele.1.token_buying_point_price,
                format!("{:?}", ele.1.tp_state),
                format!("{:?}", ele.1.ts_state),
                ele.1.token_balance,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let head_line = format!(
        "{:<3} | {:<44} | {:<12} | {:<15} | {:<15} | {:<12} |  {:<12}",
        "IDX",
        "Mint Addr",
        "Price (sol)",
        "Max Peak (sol)",
        "Buy Point MC",
        "TP Status",
        "TS Status",
    );

    let current = if *SHUT_DOWN_TIMER_SELL_ALL {
        let current_time = now.format("%H:%M:%S").to_string();
        let comparing_time = format!("{}", *SHUT_DOWN_TIME);
        if current_time == comparing_time {
            info!("SELLING ALL TOKENS ... ");
            return true;
        };
        format!("Shutdown Timer ENABLED : {}", *SHUT_DOWN_TIME)
    } else {
        format!("Shutdown Timer DISABLED")
    };
    // Format the current timestamp with milliseconds and the sorted result
    let msg = format!(
        "Pump.fun Sniper Bot Overview Panel ( {mode} ) - {}.{:03}  ( ALL {} datas ) {current}\n{}\n{}",
        now.format("%Y-%m-%d_%H:%M:%S"), // Format to include hour, minute, and second
        now.timestamp_subsec_millis(),   // Milliseconds part
        lists.len(),
        head_line,
        result_string,
    );

    // Write the message to the file
    file.write_all(msg.as_bytes())
        .expect("Unable to write to file");

    false
}
