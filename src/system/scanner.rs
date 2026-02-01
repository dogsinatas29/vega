use std::fs;
use std::process::Command;
use crate::system::context::{SystemContext, Partition, PartitionType};

pub fn scan_system() -> SystemContext {
    let mut context = SystemContext::new();
    
    // 1. Scan OS Info (Graceful fallback)
    scan_os_info(&mut context);

    // 2. Scan Partitions (Robust parsing)
    context.partitions = scan_partitions();

    context
}

fn scan_os_info(context: &mut SystemContext) {
    match fs::read_to_string("/etc/os-release") {
        Ok(content) => {
            for line in content.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    let name = line.replace("PRETTY_NAME=", "").replace("\"", "");
                    context.os_name = name;
                    break; // Found it
                }
            }
            // Fallback if PRETTY_NAME not found
            context.os_name = "Linux (Unknown Distribution)".to_string();
        },
        Err(_) => {
            // Graceful handling for non-standard Linux or other OS
            context.os_name = "Unknown OS".to_string();
        }
    }

    // Kernel version
    if let Ok(output) = Command::new("uname").arg("-r").output() {
        context.kernel_version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    }
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
            if parts.len() < 6 { continue; }

            // Basic parsing
            let filesystem = parts[0].to_string();
            let total_size = parts[1].to_string();
            let used = parts[2].to_string();
            let available = parts[3].to_string();
            let usage_percent = parts[4].to_string();
            // Handle spaces in mount points by joining the rest
            let mount_point = parts[5..].join(" ");

            // Filter out pseudo-filesystems aggressively
            if filesystem == "tmpfs" || filesystem == "devtmpfs" || filesystem == "overlay" || filesystem == "none" {
                continue;
            }
            if mount_point.starts_with("/sys") || mount_point.starts_with("/proc") || mount_point.starts_with("/dev") {
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
            } else if mount_point.starts_with("/media") || mount_point.starts_with("/run/media") || mount_point.starts_with("/mnt") {
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
    } else {
        // Log error but don't panic. Return empty list or handle differently.
        eprintln!("Error: Failed to execute 'df -h'. Partition discovery skipped.");
    }

    partitions
}
