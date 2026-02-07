use std::fs;
use std::process::Command;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct SystemContext {
    pub os_info: String,
    pub kernel_version: String,
    pub load_avg: Vec<f64>,
    pub mem_info: Value,
    pub block_devices: Value,
    pub pkg_manager: String,
    pub is_vm: bool,
    pub git_user: String,
}

impl SystemContext {
    pub fn collect() -> Self {
        SystemContext {
            os_info: Self::get_os_info(),
            kernel_version: Self::get_kernel_version(),
            load_avg: Self::get_load_avg(),
            mem_info: Self::get_mem_info(),
            block_devices: Self::get_block_devices(),
            pkg_manager: Self::detect_pkg_manager(),
            is_vm: Self::detect_vm(),
            git_user: Self::detect_git_user(),
        }
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
            },
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

        if id_like.contains("debian") || id_like.contains("ubuntu") { return "apt".to_string(); }
        if id_like.contains("fedora") || id_like.contains("rhel") { return "dnf".to_string(); }
        if id_like.contains("arch") { return "pacman".to_string(); }

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
