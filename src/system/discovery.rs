use crate::remote::rclone::RcloneProvider;
use crate::remote::RemoteMasker;
use std::fs;
use std::path::PathBuf;

pub struct Discovery;

pub struct DiscoveryResult {
    pub cloud_remotes: Vec<String>,
    pub ssh_hosts: Vec<String>,
}

impl Discovery {
    pub fn run() -> Result<DiscoveryResult, String> {
        eprintln!("⚠️  IP Unknown. Running automatic discovery...");
        let mut result = DiscoveryResult {
            cloud_remotes: Vec::new(),
            ssh_hosts: Vec::new(),
        };

        // 1. Check for local signatures
        if let Some(pm) = Self::detect_plugin_manager() {
            println!("🔍 Discovery: Found specific configuration: {}", pm);
        }

        // 2. Cloud Discovery Integration
        if let Ok(remotes) = RcloneProvider::list_remotes() {
            if !remotes.is_empty() {
                println!("☁️  Discovery: Found {} rclone remotes.", remotes.len());
                let mut masker = RemoteMasker::new();
                for remote in remotes {
                    let masked = masker.mask(&remote);
                    println!("   📡 Remote identified: {}", masked);
                    result.cloud_remotes.push(remote.clone());

                    // Depth-limited search for "workspace" indicators
                    let provider = RcloneProvider::new(remote);
                    // This is blocking for now, ideally we'd want it async or backgrounded
                    // But for discovery phase, simple is better.
                    // We'll just look for a few indicators in the root
                    if let Ok(output) = provider
                        .execute_rclone(vec!["lsjson", &format!("{}:", provider.remote_name)])
                    {
                        if let Ok(items) = serde_json::from_str::<serde_json::Value>(&output) {
                            if let Some(array) = items.as_array() {
                                if array.iter().any(|i| {
                                    let name = i["Name"].as_str().unwrap_or("");
                                    name == "Cargo.toml"
                                        || name == "lazy-lock.json"
                                        || name == ".git"
                                }) {
                                    println!("   🎯 Potential WORKSPACE found on {}", masked);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. SSH Discovery Integration
        if let Some(ssh_hosts) = Self::parse_ssh_config() {
            if !ssh_hosts.is_empty() {
                println!(
                    "🔑 Discovery: Found {} potential SSH targets in config.",
                    ssh_hosts.len()
                );
                for host in ssh_hosts {
                    println!("   📡 SSH Target identified: {}", host);
                    result.ssh_hosts.push(host);
                }
            }
        }

        eprintln!("ℹ️  Network discovery completed.");

        Ok(result)
    }

    pub fn parse_ssh_config() -> Option<Vec<String>> {
        if let Ok(home) = std::env::var("HOME") {
            let config_path = PathBuf::from(&home).join(".ssh/config");
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(config_path) {
                    let mut hosts = Vec::new();
                    for line in content.lines() {
                        let line: &str = line.trim();
                        if line.starts_with("Host ") && !line.contains('*') && !line.contains('?') {
                            let host = line.replace("Host ", "").trim().to_string();
                            if !host.is_empty() {
                                hosts.push(host);
                            }
                        }
                    }
                    return Some(hosts);
                }
            }
        }
        None
    }

    pub fn detect_plugin_manager() -> Option<String> {
        if let Ok(home) = std::env::var("HOME") {
            let lazy_lock = PathBuf::from(&home).join(".config/nvim/lazy-lock.json");
            if lazy_lock.exists() {
                return Some("lazy.nvim".to_string());
            }
        }
        None
    }
}
