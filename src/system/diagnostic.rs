use crate::context::SystemContext;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticData {
    pub context: SystemContext,
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

        DiagnosticData {
            context,
            listening_ports,
            active_connections,
            disk_io,
            uptime,
        }
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
