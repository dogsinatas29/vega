pub mod sanitizer;

use colored::Colorize;
use std::io::{self, Write};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RiskLevel {
    Info,
    Warning,
    Critical,
}

pub fn check_risk_level(command: &str) -> RiskLevel {
    let cmd = command.trim();

    // Critical: Destructive Data Loss
    if cmd.contains("rm -rf")
        || cmd.contains("mkfs")
        || cmd.contains("dd if=")
        || cmd.contains("> /dev/sd")
    {
        return RiskLevel::Critical;
    }

    // Warning: System Modification or Process Killing
    if cmd.contains("chmod 777")
        || cmd.contains("kill -9")
        || cmd.contains("shutdown")
        || cmd.contains("reboot")
        || cmd.contains("systemctl stop")
    {
        return RiskLevel::Warning;
    }

    // Warning: Package Managers (can be messy)
    if cmd.starts_with("apt remove") || cmd.starts_with("dnf remove") {
        return RiskLevel::Warning;
    }

    RiskLevel::Info
}

pub fn confirm_action(risk: RiskLevel, command: &str) -> bool {
    match risk {
        RiskLevel::Info => true,
        RiskLevel::Warning => {
            println!(
                "{} {}",
                "⚠️  WARNING:".yellow().bold(),
                "This command may modify your system.".yellow()
            );
            println!("   Command: {}", command.cyan());
            print!("{} [y/N]: ", "Do you want to proceed?".yellow());
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            input.trim().to_lowercase() == "y"
        }
        RiskLevel::Critical => {
            println!(
                "{} {}",
                "🚨 CRITICAL RISK:".red().bold().on_black(),
                "This command can cause DATA LOSS.".red().bold()
            );
            println!("   Command: {}", command.red().bold());
            println!(
                "{}",
                "To execute this command, you must type 'YES' (case-sensitive).".red()
            );
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            input.trim() == "YES"
        }
    }
}
