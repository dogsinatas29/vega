use log::{debug, info, warn};
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;

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
        let export_regex =
            Regex::new(r#"^\s*export\s+([A-Z_]+_API_KEY)=["']([^"']+)["']"#).unwrap();

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
                                        info!(
                                            "   ✅ Found potential key in {:?}:{} -> {}",
                                            file_path,
                                            line_num + 1,
                                            k
                                        );
                                        found_keys.insert(
                                            k.clone(),
                                            EnvKey {
                                                _key: k,
                                                value: v,
                                                source_file: file_path.clone(),
                                                line_num: line_num + 1,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
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

/// Scans standard shell configuration files for a specific API Key pattern.
/// Returns the first match found, MASKED for security.
/// Returns Tuple: (MaskedKey, SourcePath)
pub fn find_key(provider: &str) -> Option<(String, String)> {
    let target_var = match provider {
        "gemini" => "GEMINI_API_KEY",
        "chatgpt" => "OPENAI_API_KEY",
        "claude" => "ANTHROPIC_API_KEY",
        _ => return None,
    };

    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    // Shell config files to scan (zsh, bash, fish, profile)
    let candidates = vec![
        format!("{}/.zshrc", home),
        format!("{}/.bashrc", home),
        format!("{}/.config/fish/config.fish", home),
        format!("{}/.profile", home),
        format!("{}/.bash_profile", home),
    ];

    // Regex Explanation:
    // export\s+          : Match "export" followed by whitespace (optional in some shells, but standard for export)
    //                      Wait, some shells might just do KEY=VAL.
    //                      Let's make export optional: (?:export\s+)?
    // {target_var}       : The variable name (e.g. GEMINI_API_KEY)
    // \s*=\s*            : Equals sign with optional whitespace
    // ["']?              : Optional opening quote
    // ([a-zA-Z0-9_\-]+)  : Capture group 1: The key itself (alphanumeric + _ + -)
    // ["']?              : Optional closing quote
    let pattern = format!(
        r#"(?:export\s+)?{}\s*=\s*["']?([a-zA-Z0-9_\-]+)["']?"#,
        target_var
    );
    if let Ok(re) = Regex::new(&pattern) {
        for path in candidates {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Some(caps) = re.captures(&content) {
                    if let Some(matched) = caps.get(1) {
                        let raw_key = matched.as_str().to_string();
                        // We only return it if it looks "key-like" (e.g. not empty)
                        if !raw_key.is_empty() {
                            // Masking logic: Show first 3 chars, then ***
                            let masked = if raw_key.len() > 6 {
                                format!(
                                    "{}*******{}",
                                    &raw_key[0..3],
                                    &raw_key[raw_key.len() - 3..]
                                )
                            } else {
                                "******".to_string()
                            };
                            return Some((masked, path));
                        }
                    }
                }
            }
        }
    }

    None
}
