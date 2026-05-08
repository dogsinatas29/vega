use std::process::Command;
use std::collections::HashMap;

pub struct SmartStorage {
    pub aliases: HashMap<String, String>,
}

impl SmartStorage {
    pub fn new() -> Self {
        let mut aliases = HashMap::new();
        // Default smart aliases - in future load from config
        aliases.insert("구드".to_string(), "gdrive:".to_string());
        aliases.insert("gdrive".to_string(), "gdrive:".to_string());
        aliases.insert("나스".to_string(), "nas_sftp:".to_string());
        
        SmartStorage { aliases }
    }

    pub fn backup_cmd(&self, source: &str, target_alias: &str) -> String {
        let remote = self.aliases.get(target_alias).cloned().unwrap_or_else(|| target_alias.to_string());
        
        // Dynamic Memory Scaling
        let mut chunk_size = "64M";
        let mut transfers = 8;
        
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if line.starts_with("MemAvailable:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            if kb < 1_000_000 { // Less than 1GB available
                                chunk_size = "16M";
                                transfers = 4;
                            }
                        }
                    }
                    break;
                }
            }
        }

        let mut flags = "--progress".to_string();
        
        if remote.contains("gdrive") {
            flags.push_str(&format!(" --drive-chunk-size {} --transfers {} --vfs-cache-mode writes", chunk_size, transfers));
        } else if remote.contains("sftp") || remote.contains("nas") {
            flags.push_str(&format!(" --transfers {}", transfers * 2));
        }

        format!("rclone sync {} {} {}", source, remote, flags)
    }

    pub fn sync_vega_state(&self, target_alias: &str) -> Result<String, String> {
        let config_dir = dirs::config_dir().ok_or("Config dir not found")?.join("vega");
        let data_dir = dirs::data_local_dir().ok_or("Data dir not found")?.join("vega");

        let remote = self.aliases.get(target_alias).cloned().unwrap_or_else(|| target_alias.to_string());
        
        println!("☁️  [Sync] Preparing Cloud Synchronization to {}...", remote);
        
        // 1. Sync Config
        let config_cmd = self.backup_cmd(config_dir.to_str().unwrap(), &format!("{}/config", target_alias));
        let _ = Command::new("sh").arg("-c").arg(&config_cmd).status();

        // 2. Sync Data (DB, Reports)
        let data_cmd = self.backup_cmd(data_dir.to_str().unwrap(), &format!("{}/data", target_alias));
        let status = Command::new("sh").arg("-c").arg(&data_cmd).status()
            .map_err(|e| format!("Sync Failed: {}", e))?;

        if status.success() {
            Ok(format!("✅ Cloud Sync to {} completed successfully.", remote))
        } else {
            Err(format!("❌ Cloud Sync to {} failed with exit code: {:?}", remote, status.code()))
        }
    }
}
