use std::io::{self, Write};
use colored::Colorize;
use crate::safety::checker::RiskLevel;

pub fn confirm_action(risk: RiskLevel, command: &str) -> bool {
    match risk {
        RiskLevel::Info => true,
        RiskLevel::Warning => {
            println!("{} {}", "⚠️  WARNING:".yellow().bold(), "This command may modify your system.".yellow());
            println!("   Command: {}", command.cyan());
            print!("{} [y/N]: ", "Do you want to proceed?".yellow());
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            
            input.trim().to_lowercase() == "y"
        },
        RiskLevel::Critical => {
            println!("{} {}", "🚨 CRITICAL RISK:".red().bold().on_black(), "This command can cause DATA LOSS.".red().bold());
            println!("   Command: {}", command.red().bold());
            println!("{}", "To execute this command, you must type 'YES' (case-sensitive).".red());
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            
            input.trim() == "YES"
        }
    }
}
