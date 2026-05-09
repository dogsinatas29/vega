use crate::remote::rclone::RcloneProvider;
use crate::remote::RemoteMasker;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub struct Discovery;

#[derive(Default)]
pub struct DiscoveryResult {
    pub cloud_remotes: Vec<String>,
    pub ssh_hosts: Vec<String>,
}

impl Discovery {
    pub fn run(silent: bool) -> Result<DiscoveryResult, String> {
        use crate::ui::console::SreConsole;
        
        let hostname = crate::context::SystemContext::get_hostname();
        let ip = crate::context::SystemContext::get_local_ip();
        
        if !silent {
            SreConsole::logic(&format!("Identifying as: {} ({})", hostname.cyan(), ip.cyan()));
        }

        let mut result = DiscoveryResult {
            cloud_remotes: Vec::new(),
            ssh_hosts: Vec::new(),
        };

        // 1. Check for local signatures
        if let Some(pm) = Self::detect_plugin_manager() {
            SreConsole::logic(&format!("Found specific configuration: {}", pm));
        }

        // 2. Cloud Discovery Integration
        if let Ok(remotes) = RcloneProvider::list_remotes() {
            if !remotes.is_empty() {
                SreConsole::logic(&format!("Found {} rclone remotes.", remotes.len()));
                let mut masker = RemoteMasker::new();
                for remote in remotes {
                    let masked = masker.mask(&remote, Some("STORAGE"));
                    SreConsole::logic(&format!("Remote identified: {}", masked));
                    result.cloud_remotes.push(remote.clone());

                    // Depth-limited search for "workspace" indicators
                    let provider = RcloneProvider::new(remote);
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
                                    SreConsole::logic(&format!("Potential WORKSPACE found on {}", masked));
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
                SreConsole::logic(&format!("Found {} potential SSH targets in config.", ssh_hosts.len()));
                let mut masker = RemoteMasker::new();
                for host in ssh_hosts {
                    let masked = masker.mask(&host, Some("HOST"));
                    SreConsole::logic(&format!("SSH Target identified: {}", masked));
                    result.ssh_hosts.push(host);
                }
            }
        }

        Ok(result)
    }

    pub fn parse_ssh_config() -> Option<Vec<String>> {
        let mut hosts = Vec::new();
        if let Ok(home) = std::env::var("HOME") {
            let config_path = std::path::PathBuf::from(&home).join(".ssh/config");
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(config_path) {
                    for line in content.lines() {
                        let line = line.trim();
                        if line.starts_with("Host ") && !line.contains('*') && !line.contains('?') {
                            let host = line.replace("Host ", "").trim().to_string();
                            if !host.is_empty() && host != "localhost" {
                                hosts.push(host);
                            }
                        }
                    }
                }
            }

            // Fallback: Check known_hosts
            if let Some(kh_hosts) = Self::parse_known_hosts() {
                for host in kh_hosts {
                    if !hosts.contains(&host) {
                        hosts.push(host);
                    }
                }
            }
        }

        if hosts.is_empty() {
            None
        } else {
            Some(hosts)
        }
    }

    pub fn parse_known_hosts() -> Option<Vec<String>> {
        if let Ok(home) = std::env::var("HOME") {
            let kh_path = std::path::PathBuf::from(&home).join(".ssh/known_hosts");
            if kh_path.exists() {
                if let Ok(content) = fs::read_to_string(kh_path) {
                    let mut hosts = Vec::new();
                    for line in content.lines() {
                        if let Some(host_part) = line.split_whitespace().next() {
                            // Extract first host (it might be a comma separated list of host/ip)
                            let host = host_part.split(',').next().unwrap_or("").trim();
                            // Filter out hashed hosts or complex entries for now
                            if !host.starts_with('|') && !host.is_empty() && host != "localhost" && host != "127.0.0.1" {
                                hosts.push(host.to_string());
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
