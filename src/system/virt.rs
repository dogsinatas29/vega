
use std::process::Command;
use std::io::{self, Write};
use colored::*;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualMachine {

    pub name: String,
    pub state: String,
    pub ip_address: Option<String>,
}

pub struct VirtManager;

impl VirtManager {
    /// Lists all VMs using virsh
    pub fn list_vms() -> Vec<VirtualMachine> {
        // virsh -c qemu:///session list --all
        let output = Command::new("virsh")
            .args(&["-c", "qemu:///session", "list", "--all"])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut vms = Vec::new();
                
                // Skip header lines (Id, Name, State)
                for line in stdout.lines().skip(2) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let name = parts[1].to_string();
                        let state = parts[2..].join(" "); // State can be "shut off" or "running"
                        
                        // If running, try to get IP
                        let ip_address = if state == "running" {
                            Self::get_vm_ip(&name)
                        } else {
                            None
                        };

                        if !name.is_empty() {
                            vms.push(VirtualMachine { name, state, ip_address });
                        }
                    }
                }
                vms
            },
            Err(_) => Vec::new(), // Fail silently if virsh is missing/broken
        }
    }

    /// Gets IP address of a running VM
    pub fn get_vm_ip(name: &str) -> Option<String> {
        // virsh -c qemu:///session domifaddr {name} --source agent
        // Fallback to standard sources if agent fails or just try default
        let output = Command::new("virsh")
            .args(&["-c", "qemu:///session", "domifaddr", name])
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Parse: ipv4  192.168.122.45/24
            for line in stdout.lines() {
                if line.contains("ipv4") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    for part in parts {
                        if part.contains('.') && !part.contains("ipv4") {
                            let ip = part.split('/').next().unwrap_or(part);
                            return Some(ip.to_string());
                        }
                    }
                }
            }
        }

        // Fallback: Intercept via ARP
        if let Some(mac) = Self::get_vm_mac(name) {
            return Self::find_ip_via_arp(&mac);
        }

        None
    }

    /// Extracts MAC address from VM XML
    pub fn get_vm_mac(name: &str) -> Option<String> {
        let output = Command::new("virsh")
            .args(&["-c", "qemu:///session", "dumpxml", name])
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Look for <mac address='52:54:00:...'/>
            for line in stdout.lines() {
                if line.contains("<mac address=") {
                    let parts: Vec<&str> = line.split('\'').collect();
                    if parts.len() >= 2 {
                        return Some(parts[1].to_string());
                    }
                }
            }
        }
        None
    }

    /// Sniffs IP from host ARP table using MAC
    pub fn find_ip_via_arp(mac: &str) -> Option<String> {
        let mac_lower = mac.to_lowercase();
        let output = Command::new("arp").arg("-an").output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.to_lowercase().contains(&mac_lower) {
                    // Line looks like: ? (192.168.122.45) at 52:54:00:XX:XX:XX [ether] on virbr0
                    let start = line.find('(')?;
                    let end = line.find(')')?;
                    return Some(line[start+1..end].to_string());
                }
            }
        }
        None
    }

    /// Checks SSH access and offers to fix it
    #[allow(dead_code)]
    pub fn ensure_ssh_access(ip: &str, user: &str) -> bool {
        // 1. Check if we can connect (BatchMode to avoid hang)
        let status = Command::new("ssh")
            .args(&["-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=no", "-o", "ConnectTimeout=3", 
                    &format!("{}@{}", user, ip), "exit"])
            .status();

        if let Ok(s) = status {
            if s.success() {
                return true;
            }
        }

        // 2. Connection failed. Ask user to fix.
        println!("{}", format!("\n🔓 SSH Access to {}@{} is blocked.", user, ip).yellow());
        print!("   Auto-create SSH Tunnel (ssh-copy-id)? [Y/n]: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim().to_lowercase();
        
        if choice == "y" || choice == "yes" || choice == "" {
            // Check if pub key exists
            let home = std::env::var("HOME").unwrap(); // Unsafe unwrap for now, but usually exists
            let pub_key = std::path::Path::new(&home).join(".ssh/id_rsa.pub"); // simplified
            
            if !pub_key.exists() {
               println!("   🔑 Generating SSH Key...");
               let _ = Command::new("ssh-keygen").args(&["-t", "rsa", "-N", "", "-f", &format!("{}/.ssh/id_rsa", home)]).status();
            }

            println!("   📤 Copying ID (Please enter VM password if requested)...");
            let copy_status = Command::new("ssh-copy-id")
                .arg(format!("{}@{}", user, ip))
                .status();

            match copy_status {
                Ok(s) => if s.success() {
                    println!("{}", "   ✅ SSH Tunnel Established.".green());
                    return true;
                },
                Err(_) => eprintln!("{}", "   ❌ Failed to run ssh-copy-id".red()),
            }
        }
        
        false
    }
}
