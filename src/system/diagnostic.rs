use crate::context::SystemContext;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticData {
    pub context: SystemContext,
    pub os_detailed: String,
    pub de_info: String,
    pub cpu_info: String,
    pub listening_ports: Vec<String>,
    pub active_connections: Vec<String>,
    pub disk_io: String,
    pub uptime: String,
}

pub struct DiagnosticScanner;

impl DiagnosticScanner {
    pub fn scan() -> DiagnosticData {
        let context = SystemContext::collect();
        
        let listening_ports = Self::get_listening_ports();
        let active_connections = Self::get_active_connections();
        let disk_io = Self::get_disk_io();
        let uptime = Self::get_uptime();
        let os_detailed = Self::get_os_detailed();
        let de_info = Self::get_de_info();
        let cpu_info = Self::get_cpu_info();

        DiagnosticData {
            context,
            os_detailed,
            de_info,
            cpu_info,
            listening_ports,
            active_connections,
            disk_io,
            uptime,
        }
    }

    fn get_os_detailed() -> String {
        let output = Command::new("lsb_release")
            .arg("-ds")
            .output();
        if let Ok(out) = output {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        } else {
            // Fallback to /etc/os-release
            if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
                for line in content.lines() {
                    if line.starts_with("PRETTY_NAME=") {
                        return line.replace("PRETTY_NAME=", "").replace("\"", "").to_string();
                    }
                }
            }
            "Unknown Linux".to_string()
        }
    }

    fn get_de_info() -> String {
        std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_else(|_| std::env::var("DESKTOP_SESSION").unwrap_or_else(|_| "None (Headless/CLI)".to_string()))
    }

    fn get_cpu_info() -> String {
        let output = Command::new("lscpu")
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.starts_with("Model name:") {
                    return line.replace("Model name:", "").trim().to_string();
                }
            }
        }
        "Unknown CPU".to_string()
    }

    fn get_listening_ports() -> Vec<String> {
        // Simple ss/netstat wrapper
        let output = Command::new("ss")
            .args(&["-tunlp"])
            .output();

        if let Ok(out) = output {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .skip(1)
                .map(|s| s.to_string())
                .collect()
        } else {
            vec!["Unable to fetch ports".to_string()]
        }
    }

    fn get_active_connections() -> Vec<String> {
        let output = Command::new("ss")
            .args(&["-tunp"])
            .output();

        if let Ok(out) = output {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .skip(1)
                .map(|s| s.to_string())
                .collect()
        } else {
            vec!["Unable to fetch connections".to_string()]
        }
    }

    fn get_disk_io() -> String {
        let output = Command::new("iostat")
            .arg("-c")
            .output();
            
        if let Ok(out) = output {
            String::from_utf8_lossy(&out.stdout).to_string()
        } else {
            "iostat not found".to_string()
        }
    }

    fn get_uptime() -> String {
        let output = Command::new("uptime")
            .arg("-p")
            .output();

        if let Ok(out) = output {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        } else {
            "Unknown".to_string()
        }
    }
}
