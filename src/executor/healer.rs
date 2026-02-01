use super::ExecuteResult;
use crate::system::os::{OsInfo, PackageManager};

pub struct Healer;

impl Healer {
    pub fn diagnose(result: &ExecuteResult, os_info: &OsInfo, original_cmd: &str) -> Option<String> {
        if result.success {
            return None;
        }

        let stderr_lower = result.stderr.to_lowercase();
        let stdout_lower = result.stdout.to_lowercase();
        let combined = format!("{} {}", stderr_lower, stdout_lower);

        // --- RUST SPECIFIC FIXES (Offline) ---
        if combined.contains("e0432") {
             return Some("Check your imports. Tried `cargo add`? Or check `mod.rs` exposure.".to_string());
        }
        if combined.contains("e0425") {
             return Some("Variable not found. Check scope or `self.` prefix.".to_string());
        }
        if combined.contains("e0282") {
             return Some("Type annotation needed. Try `: Type = ...`".to_string());
        }

        // 1. Check for Permission Denied
        if combined.contains("permission denied") || result.exit_code == 126 {
            if !original_cmd.trim().starts_with("sudo") {
                return Some(format!("sudo {}", original_cmd));
            }
        }

        // 2. Check for Command Not Found
        if combined.contains("command not found") || combined.contains("not found") {
            // Extract command name (naive approach)
            // e.g., "/bin/sh: 1: cargo: not found"
            let parts: Vec<&str> = result.stderr.split(':').collect();
            for part in parts {
                if part.contains("not found") {
                     // Try to get the word before "not found"
                     let potential_cmd = part.replace("not found", "").trim().to_string();
                     if !potential_cmd.is_empty() {
                         return Some(Self::suggest_install(&potential_cmd, &os_info.pkg_manager));
                     }
                }
            }
            // Fallback: try to guess from original command
            let cmd_name = original_cmd.split_whitespace().next().unwrap_or("");
            if !cmd_name.is_empty() {
                return Some(Self::suggest_install(cmd_name, &os_info.pkg_manager));
            }
        }

        // 3. Path/Directory Errors
        if combined.contains("no such file or directory") {
             return Some("ls -F".to_string());
        }
        
        // 4. APT Locked
        if combined.contains("could not get lock") || combined.contains("resource temporarily unavailable") {
            return Some("sudo rm /var/lib/apt/lists/lock && sudo rm /var/cache/apt/archives/lock && sudo dpkg --configure -a".to_string());
        }

        None
    }

    fn suggest_install(cmd: &str, pkg_manager: &PackageManager) -> String {
        match pkg_manager {
            PackageManager::Apt => format!("sudo apt install -y {}", cmd),
            PackageManager::Dnf => format!("sudo dnf install -y {}", cmd),
            PackageManager::Pacman => format!("sudo pacman -S --noconfirm {}", cmd),
            PackageManager::Unknown => format!("echo 'Please install {} manually'", cmd),
        }
    }
}
