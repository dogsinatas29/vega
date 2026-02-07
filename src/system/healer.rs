use std::process::Command;

pub struct Healer;

impl Healer {
    pub fn analyze_journal() -> Vec<String> {
        // Read last 50 lines of journal
        let output = Command::new("journalctl")
            .args(&["-xe", "-n", "50", "--no-pager"])
            .output();

        if let Ok(o) = output {
            let log = String::from_utf8_lossy(&o.stdout);
            Self::diagnose(&log)
        } else {
            vec!["Failed to read system journal.".to_string()]
        }
    }

    fn diagnose(log: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if log.contains("nvidia") && (log.contains("mismatch") || log.contains("failed")) {
            // Verify Kernel vs Module
            let uname = Command::new("uname").arg("-r").output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            
            // Check if module loads
            let modinfo = Command::new("modinfo").arg("nvidia").output();
            let module_ok = modinfo.map(|o| o.status.success()).unwrap_or(false);

            if !module_ok {
                suggestions.push(format!("⚠️ NVIDIA Module Missing for kernel {}: Try 'sudo dkms autoinstall'.", uname));
            } else {
                suggestions.push("⚠️ NVIDIA Version Mismatch: Reboot recommended to load new kernel module.".to_string());
            }
        }
        if log.contains("No space left on device") {
            suggestions.push("⚠️ Disk Full: Try 'sudo apt autoremove' or 'docker system prune'.".to_string());
        }
        if log.contains("Address already in use") {
            suggestions.push("⚠️ Port Conflict: Check 'sudo netstat -tulpn'.".to_string());
        }

        if suggestions.is_empty() {
            suggestions.push("✅ No critical system errors detected in recent logs.".to_string());
        }
        suggestions
    }

    pub fn rotate_logs() {
        let log_path = "logs/history.jsonl";
        if let Ok(metadata) = std::fs::metadata(log_path) {
            // Rotates if > 10MB
            if metadata.len() > 10 * 1024 * 1024 {
                let _ = std::fs::rename(log_path, "logs/history.jsonl.bak");
                println!("🧹 Log Rotated: history.jsonl -> history.jsonl.bak");
            }
        }
    }
}
