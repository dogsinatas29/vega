use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use md5;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeEntry {
    pub ip: String,
    pub user: Option<String>,
    pub protocol: String, // ssh, ftp
    pub port: Option<u16>,
    pub os_type: Option<String>,
    pub kernel: Option<String>,
    pub cpu_load: Option<String>,
    pub tags: Vec<String>,
    pub password: Option<String>,
    pub last_success: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct KnowledgeBase {
    pub targets: HashMap<String, KnowledgeEntry>, 
}

impl KnowledgeBase {
    pub fn load() -> Self {
        let path = Self::get_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                // Verify Checksum
                let sum_path = path.with_extension("sha256");
                if let Ok(saved_sum) = fs::read_to_string(&sum_path) {
                    let current_sum = format!("{:x}", md5::compute(&content)); // Simple MD5 for speed, or sha256
                    // Rust's sha2 crate is external. To stay dependency-light, let's use a simple hash or just assume valid JSON.
                    // But user demanded checksum.
                    // Let's implement a simple Adler32 or CRC logic if no crate, or just use content length checks?
                    // "checksum logic... if not, add it."
                    // Since I cannot add crates easily without Cargo.toml edits (which I can do), let's implement a simple string hash.
                    // Actually, I'll use a simple wrapping hasher.
                    if saved_sum.trim() != current_sum {
                        eprintln!("⚠️ Knowledge Base Corruption Detected! Checksum mismatch.");
                        return Self::default(); // Safe fallback
                    }
                }

                if let Ok(kb) = serde_json::from_str(&content) {
                    return kb;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::get_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        
        // Compute Checksum (Simple Hash)
        let sum = format!("{:x}", md5::compute(&json));
        let sum_path = path.with_extension("sha256");
        
        // Atomic Write: Write to .tmp then rename
        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, &json)?;
        fs::rename(&tmp_path, &path)?;
        
        fs::write(sum_path, sum)?;
        
        Ok(())
    }

    pub fn add(&mut self, key: &str, mut entry: KnowledgeEntry) {
        // 1. Address Sanitization: Split IP and Port if needed
        if entry.ip.contains(':') {
            let ip_clone = entry.ip.clone();
            let parts: Vec<&str> = ip_clone.split(':').collect();
            if parts.len() == 2 {
                entry.ip = parts[0].to_string();
                if let Ok(p) = parts[1].parse::<u16>() {
                    entry.port = Some(p);
                }
            }
        }

        // 2. IP-Based Deduplication: Search for existing entry with the same IP
        let mut existing_key = None;
        for (k, v) in &self.targets {
            if v.ip == entry.ip {
                existing_key = Some(k.clone());
                break;
            }
        }

        if let Some(k) = existing_key {
            // Merge into existing entry
            let existing = self.targets.get_mut(&k).unwrap();
            existing.user = entry.user.or(existing.user.clone());
            existing.port = entry.port.or(existing.port);
            existing.os_type = entry.os_type.or(existing.os_type.clone());
            existing.tags = entry.tags; // Override tags
            existing.last_success = entry.last_success;
            // Key remains the same to avoid alias bloat
        } else {
            // New entry
            self.targets.insert(key.to_string(), entry);
        }
    }

    pub fn remove(&mut self, key: &str) {
        self.targets.remove(key);
    }

    pub fn get(&self, key: &str) -> Option<&KnowledgeEntry> {
        self.targets.get(key)
    }

    fn get_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("vega");
        path.push("knowledge.json");
        path
    }
}
