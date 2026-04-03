#[macro_export]
macro_rules! log {
  ($($arg:tt)*) => {{
    use colored::Colorize;
    let now = chrono::Local::now();
    let ts = now.format("%H:%M:%S").to_string();
    let ms = format!("{:03}", now.timestamp_subsec_millis());

    let msg = format!($($arg)*);
    let terminal_msg = format!("{}.{} {} {}", ts.dimmed(), ms.dimmed(), "LOG".white().dimmed(), msg);
    let file_msg = format!("{}.{} [LOG] {}", ts, ms, msg);

    println!("{}", terminal_msg);
    $crate::log_to_file(&file_msg);
  }}
}

#[macro_export]
macro_rules! info {
  ($($arg:tt)*) => {{
    use colored::Colorize;
    let now = chrono::Local::now();
    let ts = now.format("%H:%M:%S").to_string();
    let ms = format!("{:03}", now.timestamp_subsec_millis());

    let msg = format!($($arg)*);
    let terminal_msg = format!("{}.{} {} {}", ts.dimmed(), ms.dimmed(), "INFO".cyan().bold(), msg);
    let file_msg = format!("{}.{} [INFO] {}", ts, ms, msg);

    println!("{}", terminal_msg);
    $crate::log_to_file(&file_msg);
  }}
}

#[macro_export]
macro_rules! success {
  ($($arg:tt)*) => {{
    use colored::Colorize;
    let now = chrono::Local::now();
    let ts = now.format("%H:%M:%S").to_string();
    let ms = format!("{:03}", now.timestamp_subsec_millis());

    let msg = format!($($arg)*);
    let terminal_msg = format!("{}.{} {} {}", ts.dimmed(), ms.dimmed(), "  OK".green().bold(), msg.green());
    let file_msg = format!("{}.{} [OK] {}", ts, ms, msg);

    println!("{}", terminal_msg);
    $crate::log_to_file(&file_msg);
  }}
}

#[macro_export]
macro_rules! warning {
  ($($arg:tt)*) => {{
    use colored::Colorize;
    let now = chrono::Local::now();
    let ts = now.format("%H:%M:%S").to_string();
    let ms = format!("{:03}", now.timestamp_subsec_millis());

    let msg = format!($($arg)*);
    let terminal_msg = format!("{}.{} {} {}", ts.dimmed(), ms.dimmed(), "WARN".yellow().bold(), msg.yellow());
    let file_msg = format!("{}.{} [WARN] {}", ts, ms, msg);

    println!("{}", terminal_msg);
    $crate::log_to_file(&file_msg);
  }}
}

#[macro_export]
macro_rules! error {
  ($($arg:tt)*) => {{
    use colored::Colorize;
    let now = chrono::Local::now();
    let ts = now.format("%H:%M:%S").to_string();
    let ms = format!("{:03}", now.timestamp_subsec_millis());

    let msg = format!($($arg)*);
    let terminal_msg = format!("{}.{} {} {}", ts.dimmed(), ms.dimmed(), " ERR".red().bold(), msg.red());
    let file_msg = format!("{}.{} [ERR] {}", ts, ms, msg);

    println!("{}", terminal_msg);
    $crate::log_to_file(&file_msg);
  }}
}

