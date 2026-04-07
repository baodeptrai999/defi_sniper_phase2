use colored::*;
use pumpfun_sniper::check_endpoint_status;
use std::io::{self, Write};
use std::process::Command;

fn print_banner() {
    println!();
    println!("{}", "    ██████╗ ██╗   ██╗███╗   ███╗██████╗ ███████╗██╗   ██╗███╗   ██╗".cyan().bold());
    println!("{}", "    ██╔══██╗██║   ██║████╗ ████║██╔══██╗██╔════╝██║   ██║████╗  ██║".cyan().bold());
    println!("{}", "    ██████╔╝██║   ██║██╔████╔██║██████╔╝█████╗  ██║   ██║██╔██╗ ██║".cyan().bold());
    println!("{}", "    ██╔═══╝ ██║   ██║██║╚██╔╝██║██╔═══╝ ██╔══╝  ██║   ██║██║╚██╗██║".cyan().bold());
    println!("{}", "    ██║     ╚██████╔╝██║ ╚═╝ ██║██║     ██║     ╚██████╔╝██║ ╚████║".cyan().bold());
    println!("{}", "    ╚═╝      ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝      ╚═════╝ ╚═╝  ╚═══╝".cyan().bold());
    println!();
    println!("{}", "    ███████╗███╗   ██╗██╗██████╗ ███████╗██████╗ ".yellow().bold());
    println!("{}", "    ██╔════╝████╗  ██║██║██╔══██╗██╔════╝██╔══██╗".yellow().bold());
    println!("{}", "    ███████╗██╔██╗ ██║██║██████╔╝█████╗  ██████╔╝".yellow().bold());
    println!("{}", "    ╚════██║██║╚██╗██║██║██╔═══╝ ██╔══╝  ██╔══██╗".yellow().bold());
    println!("{}", "    ███████║██║ ╚████║██║██║     ███████╗██║  ██║".yellow().bold());
    println!("{}", "    ╚══════╝╚═╝  ╚═══╝╚═╝╚═╝     ╚══════╝╚═╝  ╚═╝".yellow().bold());
    println!();
    println!("{}", "    ┌─────────────────────────────────────────────────────────┐".bright_black());
    println!("{}", "    │    ⚡  Pumpfun Sniper Bot  ·  Durable Nonce Engine  ⚡   │".white().bold());
    println!("{}", "    └─────────────────────────────────────────────────────────┘".bright_black());
}

fn menu_row(key: &str, icon: &str, label: &str, is_exit: bool) -> String {
    let key_colored = if is_exit {
        format!(" {} ", key).red().bold().to_string()
    } else {
        format!(" {} ", key).green().bold().to_string()
    };
    let label_colored = if is_exit {
        format!("{}  {}", icon, label).bright_black().bold().to_string()
    } else {
        format!("{}  {}", icon, label).white().bold().to_string()
    };
    format!("      {}  {}", key_colored, label_colored)
}

fn print_menu() {
    let w = 57;
    let bar = "═".repeat(w);

    println!();
    println!("    {}", bar.cyan());
    println!(
        "{}",
        "              M A I N   M E N U".cyan().bold()
    );
    println!("    {}", bar.cyan());
    println!();
    println!("{}", menu_row("[ 1. ]", "🎯", "Start Sniper Bot", false));
    println!();
    println!("{}", menu_row("[ 2. ]", "🔑", "Advance Nonce Management", false));
    println!();
    println!("{}", menu_row("[ 3. ]", "💰", "All Sell", false));
    println!();
    println!("{}", menu_row("[ 4. ]", "🔄", "Wallet Rotation", false));
    println!();
    println!("{}", menu_row("[ 5. ]", "📊", "Simulation", false));
    println!();
    println!("{}", menu_row("[ 6. ]", "🔗", "Check Endpoint Status", false));
    println!();
    println!("    {}", bar.cyan());
    println!();
    println!("{}", menu_row("[ 0. ]", "⚓", "Exit", true));
    println!();
    println!("    {}", bar.cyan());
    println!();
    print!("    {} ", "▶  Select option >>".yellow().bold());
    io::stdout().flush().unwrap();
}

fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn bin_path(name: &str) -> String {
    let exe = std::env::current_exe().unwrap();
    let dir = exe.parent().unwrap();
    dir.join(name).to_string_lossy().to_string()
}

fn run_binary(name: &str) {
    let path = bin_path(name);
    match Command::new(&path).status() {
        Ok(status) => {
            if !status.success() {
                println!(
                    "{}",
                    format!("\n  ⚠️  {} exited with: {}", name, status).red()
                );
            }
        }
        Err(e) => {
            println!(
                "{}",
                format!("\n  ❌ Failed to launch {}: {}", name, e).red()
            );
        }
    }
}

#[tokio::main]
pub async fn main() {
    print_banner();

    loop {
        print_menu();
        let input = read_input();

        match input.as_str() {
            "1" => {
                run_binary("sniper-mode");
                break;
            }
            "2" => {
                run_binary("nonce-manager");
            }
            "3" => {
                run_binary("all-sell");
            }
            "4" => {
                run_binary("rotate-wallet");
            }
            "5" => {
                run_binary("simulation");
            }
            "6" => {
                check_endpoint_status().await;
            }
            "0" => {
                println!("{}", "\n  👋 Exiting...".cyan());
                break;
            }
            _ => {
                println!("{}", "\n  ⚠️  Invalid option. Try again.".red());
            }
        }
    }
}
