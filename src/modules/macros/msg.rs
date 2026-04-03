#[macro_export]
macro_rules! update {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        let now = chrono::Local::now();
        let ts = now.format("%H:%M:%S").to_string();
        let ms = format!("{:03}", now.timestamp_subsec_millis());
        let msg = format!($($arg)*);
        let terminal_msg = format!("{}.{} {} {}", ts.dimmed(), ms.dimmed(), " UPD".blue().bold(), msg);
        let file_msg = format!("{}.{} [UPD] {}", ts, ms, msg);
        println!("{}", terminal_msg);
        $crate::log_to_file(&file_msg);
    }};
}

#[macro_export]
macro_rules! result {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        let now = chrono::Local::now();
        let ts = now.format("%H:%M:%S").to_string();
        let ms = format!("{:03}", now.timestamp_subsec_millis());
        let msg = format!($($arg)*);
        let terminal_msg = format!("{}.{} {} {}", ts.dimmed(), ms.dimmed(), " RES".cyan().bold(), msg.cyan());
        let file_msg = format!("{}.{} [RES] {}", ts, ms, msg);
        println!("{}", terminal_msg);
        $crate::log_to_file(&file_msg);
    }};
}

#[macro_export]
macro_rules! alert {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        let now = chrono::Local::now();
        let ts = now.format("%H:%M:%S").to_string();
        let ms = format!("{:03}", now.timestamp_subsec_millis());
        let msg = format!($($arg)*);
        let terminal_msg = format!("{}.{} {} {}", ts.dimmed(), ms.dimmed(), "ALRT".magenta().bold(), msg.magenta());
        let file_msg = format!("{}.{} [ALRT] {}", ts, ms, msg);
        println!("{}", terminal_msg);
        $crate::log_to_file(&file_msg);
    }};
}

#[macro_export]
macro_rules! dev_log {
    ($($arg:tt)*) => {{
        if *$crate::DEV_MODE {
            use colored::Colorize;
            let now = chrono::Local::now();
            let ts = now.format("%H:%M:%S").to_string();
            let ms = format!("{:03}", now.timestamp_subsec_millis());
            let msg = format!($($arg)*);
            let terminal_msg = format!("{}.{} {} {}", ts.dimmed(), ms.dimmed(), " DEV".bright_black().bold(), msg.bright_black());
            let file_msg = format!("{}.{} [DEV] {}", ts, ms, msg);
            println!("{}", terminal_msg);
            $crate::log_to_file(&file_msg);
        }
    }};
}

#[macro_export]
macro_rules! dev_trade {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        let now = chrono::Local::now();
        let ts = now.format("%H:%M:%S").to_string();
        let ms = format!("{:03}", now.timestamp_subsec_millis());
        let msg = format!($($arg)*);
        let terminal_msg = format!("{}.{} {} {}", ts.dimmed(), ms.dimmed(), "DTRD".bright_purple().bold(), msg);
        let file_msg = format!("{}.{} [DTRD] {}", ts, ms, msg);
        println!("{}", terminal_msg);
        $crate::log_to_file(&file_msg);
    }};
}

#[macro_export]
macro_rules! pro_trade {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        let now = chrono::Local::now();
        let ts = now.format("%H:%M:%S").to_string();
        let ms = format!("{:03}", now.timestamp_subsec_millis());
        let msg = format!($($arg)*);
        let terminal_msg = format!("{}.{} {} {}", ts.dimmed(), ms.dimmed(), "PTRD".bright_green().bold(), msg.bright_green());
        let file_msg = format!("{}.{} [PTRD] {}", ts, ms, msg);
        println!("{}", terminal_msg);
        $crate::log_to_file(&file_msg);
    }};
}
