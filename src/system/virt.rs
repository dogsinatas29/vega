use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmInfo {
    pub id: Option<String>,
    pub name: String,
    pub state: String, // running, shut off
    pub ip: Option<String>,
}

pub struct VmScanner;

impl VmScanner {
    pub fn scan() -> Vec<VmInfo> {
        let mut vms = Vec::new();

        // 1. Libvirt (virsh)
        // Check if virsh is installed
        if Command::new("virsh").arg("-v").output().is_ok() {
            // Get list of VMs
            let output = Command::new("virsh")
                .args(&["-c", "qemu:///session", "list", "--all"])
                .output();

            if let Ok(o) = output {
                let stdout = String::from_utf8_lossy(&o.stdout);
                // Parse virsh output
                // Id   Name            State
                // ------------------------------
                // 1    ubuntu22.04     running
                // -    fedora38        shut off

                for line in stdout.lines().skip(2) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let id = if parts[0] == "-" {
                            None
                        } else {
                            Some(parts[0].to_string())
                        };
                        let name = parts[1].to_string();
                        let state = parts[2..].join(" ");

                        // Try to find IP if running
                        let mut ip = None;
                        if state.contains("running") {
                            ip = Self::get_dom_ip(&name);
                        }

                        vms.push(VmInfo {
                            id,
                            name,
                            state,
                            ip,
                        });
                    }
                }
            }
        }

        vms
    }

    fn get_dom_ip(name: &str) -> Option<String> {
        // Try virsh domifaddr
        let output = Command::new("virsh")
            .args(&[
                "-c",
                "qemu:///session",
                "domifaddr",
                name,
                "--source",
                "agent",
            ])
            .output();

        if let Ok(o) = output {
            // If agent works, great. If not, try lease?
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Name       MAC address          Protocol     Address
            // -------------------------------------------------------------------------------
            // vnet0      52:54:00:1d:7b:3e    ipv4         192.168.122.185/24

            for line in stdout.lines() {
                if line.contains("ipv4") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(addr_part) = parts.iter().find(|p| p.contains('/')) {
                        let ip = addr_part.split('/').next().unwrap_or("").to_string();
                        return Some(ip);
                    }
                }
            }
        }

        // Fallback: ARP table lookup (naive)?
        // Or just return None for now as per KISS
        None
    }

    pub fn list_vms() -> Vec<VmInfo> {
        Self::scan()
    }
}

use std::process::Command;

// Aliases for compatibility with other modules (e.g. scanner.rs, context.rs)
pub type VirtualMachine = VmInfo;
pub type VirtManager = VmScanner;

pub struct VmController;

impl VmController {
    pub fn start(name: &str) -> Result<String, String> {
        // 1. Check if running
        let vms = VmScanner::scan();
        if let Some(vm) = vms.iter().find(|v| v.name == name) {
            if vm.state == "running" {
                return Ok(format!("VM '{}' is already running.", name));
            }
        }

        // 2. Start
        let output = Command::new("virsh")
            .args(&["-c", "qemu:///session", "start", name])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(format!("🚀 VM '{}' started successfully.", name))
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    #[allow(dead_code)]
    pub fn get_ip(name: &str) -> Option<String> {
        let vms = VmScanner::scan();
        if let Some(vm) = vms.iter().find(|v| v.name == name) {
            return vm.ip.clone();
        }
        None
    }
}
