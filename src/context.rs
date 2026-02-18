use crate::system::virt::{VirtManager, VirtualMachine};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub mount_point: String,
    pub filesystem: String,
    pub total_size: String,
    pub used: String,
    pub available: String,
    pub usage_percent: String,
    pub partition_type: PartitionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PartitionType {
    Root,
    User,
    Media,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    pub os_name: String,
    pub kernel_version: String,
    pub load_avg: Vec<f64>,
    pub mem_info: Value,
    pub block_devices: Value,
    pub pkg_manager: String,
    pub is_vm: bool,
    pub git_user: String,
    pub partitions: Vec<Partition>,
    pub vms: Vec<VirtualMachine>,
    pub env_vars: HashMap<String, String>,
    pub plugin_manager: Option<String>,
    pub ssh_auth_sock: Option<String>,
    pub locale: String,
}

impl SystemContext {
    pub fn new() -> Self {
        SystemContext {
            os_name: "Unknown".to_string(),
            kernel_version: "Unknown".to_string(),
            load_avg: Vec::new(),
            mem_info: Value::Null,
            block_devices: Value::Null,
            pkg_manager: "unknown".to_string(),
            is_vm: false,
            git_user: "Unknown".to_string(),
            partitions: Vec::new(),
            vms: Vec::new(),
            env_vars: HashMap::new(),
            plugin_manager: None,
            ssh_auth_sock: None,
            locale: "en_US.UTF-8".to_string(),
        }
    }

    pub fn collect() -> Self {
        SystemContext {
            os_name: Self::get_os_info(),
            kernel_version: Self::get_kernel_version(),
            load_avg: Self::get_load_avg(),
            mem_info: Self::get_mem_info(),
            block_devices: Self::get_block_devices(),
            pkg_manager: Self::detect_pkg_manager(),
            is_vm: Self::detect_vm(),
            git_user: Self::detect_git_user(),
            partitions: Self::scan_partitions(),
            vms: VirtManager::list_vms(),
            env_vars: HashMap::new(),
            plugin_manager: Self::detect_plugin_manager(),
            ssh_auth_sock: std::env::var("SSH_AUTH_SOCK").ok(),
            locale: Self::get_locale(),
        }
    }

    fn get_locale() -> String {
        std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .unwrap_or_else(|_| {
                Command::new("locale")
                    .arg("-a")
                    .output()
                    .ok()
                    .and_then(|o| {
                        String::from_utf8_lossy(&o.stdout)
                            .lines()
                            .next()
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| "en_US.UTF-8".to_string())
            })
    }

    fn scan_partitions() -> Vec<Partition> {
        let mut partitions = Vec::new();
        // Execute df -h
        // Expected output format: Filesystem Size Used Avail Use% Mounted on
        let output = Command::new("df").arg("-h").output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();

            for line in lines.iter().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 6 {
                    continue;
                }

                // Basic parsing
                let filesystem = parts[0].to_string();
                let total_size = parts[1].to_string();
                let used = parts[2].to_string();
                let available = parts[3].to_string();
                let usage_percent = parts[4].to_string();
                // Handle spaces in mount points by joining the rest
                let mount_point = parts[5..].join(" ");

                // Filter out pseudo-filesystems aggressively
                if filesystem == "tmpfs"
                    || filesystem == "devtmpfs"
                    || filesystem == "overlay"
                    || filesystem == "none"
                {
                    continue;
                }
                if mount_point.starts_with("/sys")
                    || mount_point.starts_with("/proc")
                    || mount_point.starts_with("/dev")
                {
                    continue;
                }
                // Helper: check for "loop" devices (Snap packages usually)
                if filesystem.contains("/dev/loop") {
                    continue;
                }

                // Advanced Partition Classification
                let partition_type = if mount_point == "/" {
                    PartitionType::Root
                } else if mount_point == "/home" {
                    PartitionType::User
                } else if mount_point.starts_with("/media")
                    || mount_point.starts_with("/run/media")
                    || mount_point.starts_with("/mnt")
                {
                    PartitionType::Media
                } else if mount_point.contains("User") || mount_point.contains("Home") {
                    // Case insensitive name check could be added here, but keeping it simple for now
                    PartitionType::User
                } else {
                    PartitionType::Other
                };

                partitions.push(Partition {
                    mount_point,
                    filesystem,
                    total_size,
                    used,
                    available,
                    usage_percent,
                    partition_type,
                });
            }
        }
        partitions
    }

    fn get_os_info() -> String {
        fs::read_to_string("/etc/os-release")
            .unwrap_or_default()
            .lines()
            .find(|l| l.starts_with("PRETTY_NAME="))
            .map(|l| l.replace("PRETTY_NAME=", "").replace("\"", ""))
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn get_kernel_version() -> String {
        fs::read_to_string("/proc/version")
            .unwrap_or_else(|_| "Unknown".to_string())
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn get_load_avg() -> Vec<f64> {
        fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .split_whitespace()
            .take(3)
            .filter_map(|s| s.parse().ok())
            .collect()
    }

    fn get_mem_info() -> Value {
        let content = fs::read_to_string("/proc/meminfo").unwrap_or_default();
        let mut map = serde_json::Map::new();
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                if ["MemTotal", "MemAvailable", "SwapTotal"].contains(&key) {
                    map.insert(key.to_string(), Value::String(parts[1].trim().to_string()));
                }
            }
        }
        Value::Object(map)
    }

    fn get_block_devices() -> Value {
        let output = Command::new("lsblk")
            .args(&["-J", "-o", "NAME,SIZE,TYPE,MOUNTPOINT"])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let s = String::from_utf8_lossy(&o.stdout);
                serde_json::from_str(&s).unwrap_or(Value::Null)
            }
            _ => Value::Null,
        }
    }

    fn detect_pkg_manager() -> String {
        let content = fs::read_to_string("/etc/os-release").unwrap_or_default();
        let mut id = "unknown";
        let mut id_like = "";

        for line in content.lines() {
            if line.starts_with("ID=") {
                id = line.split('=').nth(1).unwrap_or("").trim_matches('"');
            }
            if line.starts_with("ID_LIKE=") {
                id_like = line.split('=').nth(1).unwrap_or("").trim_matches('"');
            }
        }

        match id {
            "ubuntu" | "debian" | "kali" | "pop" => return "apt".to_string(),
            "fedora" | "rhel" | "centos" | "almalinux" => return "dnf".to_string(),
            "arch" | "manjaro" | "endeavouros" => return "pacman".to_string(),
            _ => {}
        }

        if id_like.contains("debian") || id_like.contains("ubuntu") {
            return "apt".to_string();
        }
        if id_like.contains("fedora") || id_like.contains("rhel") {
            return "dnf".to_string();
        }
        if id_like.contains("arch") {
            return "pacman".to_string();
        }

        "unknown".to_string()
    }

    fn detect_vm() -> bool {
        let output = Command::new("lsmod").output();
        if let Ok(o) = output {
            let s = String::from_utf8_lossy(&o.stdout);
            if s.contains("kvm") || s.contains("vbox") {
                return true;
            }
        }
        if let Ok(o) = Command::new("systemd-detect-virt").output() {
            if !o.stdout.is_empty() && String::from_utf8_lossy(&o.stdout).trim() != "none" {
                return true;
            }
        }
        false
    }

    fn detect_git_user() -> String {
        Command::new("git")
            .args(&["config", "--get", "user.name"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn detect_plugin_manager() -> Option<String> {
        crate::system::discovery::Discovery::detect_plugin_manager()
    }

    #[allow(dead_code)]
    pub fn detect_current_project() -> String {
        let cwd = std::env::current_dir().unwrap_or_default();
        let path_str = cwd.to_string_lossy();

        if path_str.contains("doom") || fs::metadata("DOOM.WAD").is_ok() {
            return "Project: DooM Engine".to_string();
        }
        if path_str.contains("vega") || fs::metadata("vega_config.toml").is_ok() {
            return "Project: Vega Agent".to_string();
        }
        "Project: Unknown".to_string()
    }
}
