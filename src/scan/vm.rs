use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmInfo {
    pub name: String,
    pub state: String,
    pub ip: Option<String>,
}

pub struct VmScanner;

impl VmScanner {
    pub fn scan() -> Vec<VmInfo> {
        let output = Command::new("virsh")
            .arg("-c")
            .arg("qemu:///session")
            .arg("list")
            .arg("--all")
            .output();

        let mut vms = Vec::new();

        if let Ok(o) = output {
            let stdout = String::from_utf8_lossy(&o.stdout);
            for line in stdout.lines().skip(2) {
                // Skip Id/Name/State header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[1];
                    let state = parts.last().unwrap_or(&"unknown");

                    if !name.is_empty() {
                        let ip = Self::get_ip(name);
                        vms.push(VmInfo {
                            name: name.to_string(),
                            state: state.to_string(),
                            ip,
                        });
                    }
                }
            }
        }
        vms
    }

    fn get_ip(domain: &str) -> Option<String> {
        // 1. Try virsh domifaddr (Agent) - Most Accurate
        let output = Command::new("virsh")
            .args(&[
                "-c",
                "qemu:///session",
                "domifaddr",
                domain,
                "--source",
                "agent",
            ])
            .output();

        if let Ok(o) = &output {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout);
                for line in s.lines() {
                    if line.contains("ipv4") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(ip_cidr) =
                            parts.iter().find(|p| p.contains(".") && p.contains("/"))
                        {
                            let ip: Vec<&str> = ip_cidr.split('/').collect();
                            return Some(ip[0].to_string());
                        }
                    }
                }
            }
        }

        // 2. Try virsh domifaddr (Lease) - Reliable for NAT/Bridge
        let output_lease = Command::new("virsh")
            .args(&[
                "-c",
                "qemu:///session",
                "domifaddr",
                domain,
                "--source",
                "lease",
            ])
            .output();

        if let Ok(o) = &output_lease {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout);
                for line in s.lines() {
                    if line.contains("ipv4") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(ip_cidr) =
                            parts.iter().find(|p| p.contains(".") && p.contains("/"))
                        {
                            let ip: Vec<&str> = ip_cidr.split('/').collect();
                            return Some(ip[0].to_string());
                        }
                    }
                }
            }
        }

        // 3. Fallback: ARP Table via MAC address
        // ... (Existing ARP logic)
        // 3.1 Get MAC
        let mac_output = Command::new("virsh")
            .args(&["-c", "qemu:///session", "domiflist", domain])
            .output();

        let mut mac_addr = String::new();
        if let Ok(o) = mac_output {
            let s = String::from_utf8_lossy(&o.stdout);
            for line in s.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 && parts[4].contains(":") {
                    mac_addr = parts[4].to_string();
                    break;
                }
            }
        }

        if !mac_addr.is_empty() {
            // 3.2 Check /proc/net/arp (Fast/Direct)
            if let Ok(arp_content) = std::fs::read_to_string("/proc/net/arp") {
                for line in arp_content.lines() {
                    if line.contains(&mac_addr) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if !parts.is_empty() {
                            return Some(parts[0].to_string());
                        }
                    }
                }
            }

            // 3.3 Fallback to 'arp -a' command (System wide)
            let arp_output = Command::new("arp").arg("-a").output();
            if let Ok(o) = arp_output {
                let s = String::from_utf8_lossy(&o.stdout);
                for line in s.lines() {
                    if line.contains(&mac_addr) {
                        // arp -a output: "? (192.168.122.10) at 52:54:00:xx:xx:xx [ether] on virbr0"
                        if let Some(start) = line.find('(') {
                            if let Some(end) = line.find(')') {
                                return Some(line[start + 1..end].to_string());
                            }
                        }
                        // Alternative format
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if !parts.is_empty() && parts[0].contains('.') {
                            return Some(parts[0].to_string());
                        }
                    }
                }
            }
        }

        None
    }
}
