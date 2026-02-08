use std::fs;
use std::process::Command;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub enum PackageManager {
    Apt,
    Dnf,
    Pacman,
    Unknown,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct OsInfo {
    pub id: String,         // e.g., "ubuntu", "fedora"
    pub version: String,    // e.g., "25.10", "41"
    pub arch: String,       // e.g., "x86_64"
    pub pkg_manager: PackageManager,
    // VM Discovery fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_method: Option<String>,  // "ssh" or "console"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vm_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vm_uri: Option<String>,
}

impl OsInfo {
    pub fn detect() -> Self {
        let release_info = fs::read_to_string("/etc/os-release")
            .unwrap_or_else(|_| "ID=unknown\nVERSION_ID=unknown".to_string());

        let mut id = "unknown".to_string();
        let mut version = "unknown".to_string();

        // 1. Parse /etc/os-release
        for line in release_info.lines() {
            if let Some(val) = line.strip_prefix("ID=") {
                id = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("VERSION_ID=") {
                version = val.trim_matches('"').to_string();
            }
        }

        // 2. Detect Architecture
        let arch = std::env::consts::ARCH.to_string();

        // 3. Detect Package Manager
        let pkg_manager = Self::detect_package_manager();

        Self { 
            id, 
            version, 
            arch, 
            pkg_manager,
            ip_address: None,
            access_method: None,
            console_command: None,
            vm_name: None,
            vm_uri: None,
        }
    }

    fn detect_package_manager() -> PackageManager {
        if Command::new("apt").arg("--version").output().is_ok() {
            PackageManager::Apt
        } else if Command::new("dnf").arg("--version").output().is_ok() {
            PackageManager::Dnf
        } else if Command::new("pacman").arg("--version").output().is_ok() {
            PackageManager::Pacman
        } else {
            PackageManager::Unknown
        }
    }
}
