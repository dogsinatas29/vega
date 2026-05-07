use crate::context::SystemContext;
use crate::knowledge::{KnowledgeBase, KnowledgeEntry};
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
        let context = SystemContext::collect(true);
        
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

    /// 🛰️ Milestone v0.0.14: Remote Diagnostic Scan
    pub async fn scan_remote(target: &str) -> Result<DiagnosticData, String> {
        use crate::connection::ssh::SshConnection;
        use colored::*;

        // 🛡️ [Milestone v0.0.14.11] Fetch User/Port from Inventory
        let kb = KnowledgeBase::load();
        let (user, mut port) = if let Some(entry) = kb.targets.values().find(|e| e.ip == target) {
            (entry.user.as_deref(), entry.port)
        } else {
            (None, None)
        };

        // 🛡️ [Milestone v0.0.14.12] Intelligent Port Correction for Scan
        if port == Some(11434) {
            eprintln!("⚠️  [SRE Guard] Inventory has AI port (11434) for SSH. Redirecting to 22 for scan.");
            port = Some(22);
        }

        println!("📡 Scanning remote host: {} (User: {:?}, Port: {:?})", target.cyan(), user.unwrap_or("default"), port.unwrap_or(22));

        let mut final_pass: Option<String> = None;
        let mut current_user = user.map(|s| s.to_string());

        // 1. Basic Hostname/IP resolution for context (with Credential Recovery)
        let mut hostname_result = SshConnection::execute_remote_async(target, user, port, None, "hostname").await;
        
        if let Err(e) = &hostname_result {
            if e.contains("Permission denied") {
                eprintln!("🔑 [Auth Failed] Permission denied for {}@{}", current_user.as_deref().unwrap_or("default"), target);
                
                // 🛡️ [Milestone v0.0.14.15] Intelligent Skip: If user already exists, ask password directly
                if current_user.is_none() {
                    if let Some(new_user) = crate::interactor::Interactor::ask(&format!("Enter username for {}", target)) {
                        // 💾 Update KB
                        let mut kb_update = KnowledgeBase::load();
                        let mut found = false;
                        for entry in kb_update.targets.values_mut() {
                            if entry.ip == target {
                                entry.user = Some(new_user.clone());
                                found = true;
                            }
                        }
                        if found {
                            let _ = kb_update.save();
                            println!("✅ Updated Knowledge Base with user '{}'", new_user);
                        }
                        current_user = Some(new_user.clone());
                        
                        // 🔄 Retry 1: With Username
                        println!("🔄 Retrying scan with user '{}'...", new_user);
                        hostname_result = SshConnection::execute_remote_async(target, current_user.as_deref(), port, None, "hostname").await;
                    }
                }

                // Check again if we need a password
                if let Err(e2) = &hostname_result {
                    if e2.contains("Permission denied") && current_user.is_some() {
                        let user_str = current_user.clone().unwrap();
                        eprintln!("🔑 [Auth Failed] Password might be required for {}@{}", user_str, target);
                        if let Some(pass) = crate::interactor::Interactor::ask_password(&format!("Enter password for {}@{}", user_str, target)) {
                            println!("🔐 Password received. Persisting to Knowledge Base...");
                            
                            // 💾 [Milestone v0.0.14.23] Robust Persist: Match by IP or create new
                            let mut kb_update = KnowledgeBase::load();
                            let clean_target = target.trim();
                            let mut found = false;
                            
                            for entry in kb_update.targets.values_mut() {
                                if entry.ip.trim() == clean_target {
                                    entry.password = Some(pass.clone());
                                    found = true;
                                }
                            }
                            
                            if !found {
                                // Create new entry if not found
                                kb_update.targets.insert(format!("{}:{}", clean_target, port.unwrap_or(22)), KnowledgeEntry {
                                    ip: clean_target.to_string(),
                                    user: current_user.clone(),
                                    protocol: "ssh".to_string(),
                                    port: Some(port.unwrap_or(22)),
                                    os_type: None,
                                    kernel: None,
                                    cpu_load: None,
                                    tags: Vec::new(),
                                    password: Some(pass.clone()),
                                    last_success: "Added via Recovery".to_string(),
                                });
                                found = true;
                            }

                            if found {
                                let _ = kb_update.save();
                                println!("✅ Updated Knowledge Base with secure password for '{}'", clean_target);
                            }

                            final_pass = Some(pass.clone());
                            
                            // 🔄 Retry 2: With Password
                            hostname_result = SshConnection::execute_remote_async(target, current_user.as_deref(), port, final_pass.as_deref(), "hostname").await;
                        }
                    }
                }
            }
        }

        let final_user_ref = current_user.as_deref();
        let final_pass_ref = final_pass.as_deref();

        let hostname = match hostname_result {
            Ok(h) => h.trim().to_string(),
            Err(e) => {
                let diag = SshConnection::diagnose(Some(255), &e);
                eprintln!("❌ SSH Connection Failed: {}", diag.message.red());
                eprintln!("   Recommendation: {}", diag.recommendation.yellow());
                return Err(format!("SSH Connection Failed: {}", diag.message));
            }
        };
        
        // 🛡️ [Milestone v0.0.14.16] Remote Isolation: Do NOT mix with local context
        let mut context = SystemContext {
            hostname: hostname.clone(),
            local_ip: target.to_string(),
            os_name: "Unknown Remote".to_string(),
            kernel_version: "Unknown".to_string(),
            load_avg: vec![0.0, 0.0, 0.0],
            mem_info: serde_json::Value::Null,
            block_devices: serde_json::Value::Null,
            pkg_manager: "Unknown".to_string(),
            is_vm: false,
            git_user: "Unknown".to_string(),
            partitions: Vec::new(),
            vms: Vec::new(),
            env_vars: std::collections::HashMap::new(),
            plugin_manager: None,
            ssh_auth_sock: None,
            locale: "Unknown".to_string(),
            remotes: Vec::new(),
            sync_edges: Vec::new(),
        };
        
        // 2. Fetch ALL Remote Metrics in ONE SHOT (Optimization v2.0)
        let multi_cmd = "hostname; cat /etc/os-release | grep PRETTY_NAME | cut -d'=' -f2 | tr -d '\"'; uname -r; lscpu | grep 'Model name:' | cut -d':' -f2 | xargs; uptime -p; cat /proc/loadavg; cat /proc/meminfo | head -n 5; iostat -c 1 1 2>/dev/null || echo 'no-iostat'; ss -tunlp | head -n 20; ss -tunp | head -n 20";
        
        eprintln!("📊 [Internal] Fetching full metric payload in one-shot...");
        let payload = SshConnection::execute_remote_async(target, final_user_ref, port, final_pass_ref, multi_cmd).await?;
        let lines: Vec<&str> = payload.lines().collect();
        
        if lines.len() < 5 {
            return Err("Incomplete payload received from remote".to_string());
        }

        let hostname = lines.get(0).unwrap_or(&"Unknown").trim().to_string();
        let os_detailed = lines.get(1).unwrap_or(&"Unknown OS").trim().to_string();
        let kernel_version = lines.get(2).unwrap_or(&"Unknown Kernel").trim().to_string();
        let cpu_info = lines.get(3).unwrap_or(&"Unknown CPU").trim().to_string();
        let uptime = lines.get(4).unwrap_or(&"Unknown Uptime").trim().to_string();
        
        context.hostname = hostname;
        context.os_name = os_detailed.clone();
        context.kernel_version = kernel_version;
        
        // Find memory and other parts by markers if needed, but for now we'll assume a basic order
        // This is a simplified parser for the one-shot payload
        let load_line = lines.iter().find(|l| l.contains("0.") || l.contains("1.")).map(|s| s.to_string()).unwrap_or_default();
        let loads: Vec<f64> = load_line.split_whitespace().take(3).filter_map(|s| s.parse().ok()).collect();
        if loads.len() == 3 {
            context.load_avg = loads;
        }

        let mut mem_map = serde_json::Map::new();
        for line in lines.iter().filter(|l| l.contains("Mem") || l.contains("Swap")) {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                mem_map.insert(parts[0].trim().to_string(), serde_json::Value::String(parts[1].trim().to_string()));
            }
        }
        context.mem_info = serde_json::Value::Object(mem_map);

        Ok(DiagnosticData {
            context,
            os_detailed,
            de_info: "Remote (Headless)".to_string(),
            cpu_info,
            listening_ports: lines.iter().filter(|l| l.contains("LISTEN")).map(|s| s.to_string()).collect(),
            active_connections: lines.iter().filter(|l| l.contains("ESTAB")).map(|s| s.to_string()).collect(),
            disk_io: lines.iter().find(|l| l.contains("avg-cpu")).map(|s| s.to_string()).unwrap_or_else(|| "N/A".to_string()),
            uptime,
        })
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
