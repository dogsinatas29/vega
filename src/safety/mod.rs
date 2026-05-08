pub mod sanitizer;
pub mod risk;
pub mod policy;

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

pub struct SreGuard;

impl SreGuard {
    pub fn secure_port_rewrite(raw_cmd: &str) -> String {
        let cmd = raw_cmd.trim();
        let is_ssh_variant = cmd.starts_with("ssh ") || cmd.starts_with("scp ") || 
                             cmd.starts_with("rsync ") || cmd.starts_with("sftp ");
        if !is_ssh_variant { return raw_cmd.to_string(); }
        let partition_idx = cmd.find('\'').or_else(|| cmd.find('"'));
        match partition_idx {
            Some(idx) => {
                let (transport, payload) = cmd.split_at(idx);
                let safe_transport = transport.replace(":11434", ":22").replace("-p 11434", "-p 22").replace("-P 11434", "-P 22");
                if safe_transport != transport { println!("{}", "🛡️  [SRE Guard] Transport port 11434 redirected to 22. Payload preserved.".yellow()); }
                format!("{}{}", safe_transport, payload)
            }
            None => {
                let safe_cmd = cmd.replace(":11434", ":22").replace("-p 11434", "-p 22").replace("-P 11434", "-P 22");
                if safe_cmd != cmd { println!("{}", "🛡️  [SRE Guard] Direct connection port 11434 redirected to 22.".yellow()); }
                safe_cmd
            }
        }
    }

    /// 🛡️ Anti-Scraping Shield (v2.1)
    /// Blocks fragile shell pipelines using a Risk Score system.
    pub fn check_scraping(command: &str) -> Result<(), String> {
        let cmd_lower = command.to_lowercase();
        let mut risk_score = 0;
        
        // Count pipes
        risk_score += cmd_lower.chars().filter(|&c| c == '|').count() * 2;
        
        // Forbidden tools
        let forbidden = vec!["grep", "awk", "sed", "cut", "jq", "xargs"];
        for tool in forbidden {
            if cmd_lower.contains(tool) {
                risk_score += 3;
            }
        }
        
        if risk_score >= 5 {
            return Err(format!(
                "🚨 Scraping Risk Alert (Score: {}): This command looks like a fragile status-parsing pipeline. Please use a Typed Intent instead.",
                risk_score
            ));
        }
        Ok(())
    }
}
