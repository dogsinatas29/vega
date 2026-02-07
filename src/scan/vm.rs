use std::process::Command;

#[derive(Debug)]
pub struct VmInfo {
    pub name: String,
    pub state: String,
    pub ip: Option<String>,
}

pub struct VmScanner;

impl VmScanner {
    pub fn scan() -> Vec<VmInfo> {
        let output = Command::new("virsh")
            .arg("list")
            .arg("--all")
            .output();

        let mut vms = Vec::new();

        if let Ok(o) = output {
            let stdout = String::from_utf8_lossy(&o.stdout);
            for line in stdout.lines().skip(2) { // Skip Id/Name/State header
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
            .args(&["domifaddr", domain, "--source", "agent"])
            .output();

        if let Ok(o) = &output {
             if o.status.success() {
                 let s = String::from_utf8_lossy(&o.stdout);
                 for line in s.lines() {
                     if line.contains("ipv4") {
                         let parts: Vec<&str> = line.split_whitespace().collect();
                         if let Some(ip_cidr) = parts.iter().find(|p| p.contains(".") && p.contains("/")) {
                             let ip: Vec<&str> = ip_cidr.split('/').collect();
                             return Some(ip[0].to_string());
                         }
                     }
                 }
             }
        }

        // 2. Try virsh domifaddr (Lease) - Reliable for NAT/Bridge
        let output_lease = Command::new("virsh")
            .args(&["domifaddr", domain, "--source", "lease"])
            .output();

        if let Ok(o) = &output_lease {
             if o.status.success() {
                 let s = String::from_utf8_lossy(&o.stdout);
                 for line in s.lines() {
                     if line.contains("ipv4") {
                         let parts: Vec<&str> = line.split_whitespace().collect();
                         if let Some(ip_cidr) = parts.iter().find(|p| p.contains(".") && p.contains("/")) {
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
            .args(&["domiflist", domain])
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
            // 3.2 Check /proc/net/arp
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
        }

        None
    }
}
