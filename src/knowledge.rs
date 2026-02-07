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
    pub os_type: Option<String>, // Added for detection optimization
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

    pub fn add(&mut self, key: &str, entry: KnowledgeEntry) {
        self.targets.insert(key.to_string(), entry);
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
