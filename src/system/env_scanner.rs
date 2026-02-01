use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use regex::Regex;
use std::collections::HashMap;
use log::{debug, info, warn};

pub struct EnvScanner;

#[derive(Debug, Clone)]
pub struct EnvKey {
    pub _key: String,
    pub value: String,
    pub source_file: PathBuf,
    pub line_num: usize,
}

impl EnvScanner {
    pub fn scan_shell_configs() -> HashMap<String, EnvKey> {
        let mut found_keys = HashMap::new();
        let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
        
        // Scan standard shell configs
        let config_files = vec![
            PathBuf::from(&home).join(".bashrc"),
            PathBuf::from(&home).join(".zshrc"),
            PathBuf::from(&home).join(".bash_profile"),
            PathBuf::from(&home).join(".profile"),
        ];

        // Strict Regex: export KEY="VALUE" or export KEY='VALUE'
        let export_regex = Regex::new(r#"^\s*export\s+([A-Z_]+_API_KEY)=["']([^"']+)["']"#).unwrap();

        debug!("🔍 Starting EnvScanner...");

        for file_path in config_files {
            if !file_path.exists() {
                continue;
            }

            match File::open(&file_path) {
                Ok(file) => {
                    let reader = io::BufReader::new(file);
                    
                    for (line_num, line) in reader.lines().enumerate() {
                        if let Ok(line_content) = line {
                            let trimmed = line_content.trim();
                            if let Some(caps) = export_regex.captures(trimmed) {
                                let key = caps.get(1).map(|m| m.as_str().to_string());
                                let value = caps.get(2).map(|m| m.as_str().to_string());

                                if let (Some(k), Some(v)) = (key, value) {
                                    if !found_keys.contains_key(&k) {
                                        info!("   ✅ Found potential key in {:?}:{} -> {}", file_path, line_num + 1, k);
                                        found_keys.insert(k.clone(), EnvKey {
                                            _key: k,
                                            value: v,
                                            source_file: file_path.clone(),
                                            line_num: line_num + 1,
                                        });
                                    }
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    warn!("   ⚠️ Could not read file {:?}: {}", file_path, e);
                    continue;
                }
            }
        }
        
        found_keys
    }

    /// Reads system LANG environment variable
    #[allow(dead_code)]
    pub fn get_locale() -> String {
        env::var("LANG").unwrap_or_else(|_| "en_US.UTF-8".to_string())
    }

    /// Validates if a string looks like a valid URL (basic check)
    pub fn validate_url(url: &str) -> bool {
        // Basic regex for http/https URLs
        let url_regex = Regex::new(r"^(https?://)?[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}(/.*)?$").unwrap();
        url_regex.is_match(url)
    }
}
