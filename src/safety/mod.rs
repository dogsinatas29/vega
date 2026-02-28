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
pub struct SafetyRegistry;

impl SafetyRegistry {
    pub fn check_transfer_size(size_bytes: u64) -> Result<(), String> {
        // Limit: 1GB (1024 * 1024 * 1024 bytes)
        const MAX_TRANSFER_SIZE: u64 = 1073741824;

        if size_bytes > MAX_TRANSFER_SIZE {
            return Err(format!(
                "🚨 Transfer rejected: Size ({} bytes) exceeds safety limit ({} bytes).",
                size_bytes, MAX_TRANSFER_SIZE
            ));
        }
        Ok(())
    }

    pub fn validate_rclone_command(args: &[&str]) -> RiskLevel {
        if args.contains(&"sync") || args.contains(&"copy") {
            return RiskLevel::Warning;
        }
        if args.contains(&"delete") || args.contains(&"purge") {
            return RiskLevel::Critical;
        }
        RiskLevel::Info
    }
}
