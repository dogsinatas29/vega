use crate::knowledge::KnowledgeBase;
use crate::reporting::analytics::Analytics;
use crate::storage::db::Database;
use std::net::ToSocketAddrs;

pub fn show_status(kb: &KnowledgeBase, target: Option<&str>) {
    use crate::context::SystemContext;
    use crate::connection::ssh::SshConnection;
    use colored::Colorize;

    if let Some(t) = target {
        let mut resolved_target = t.to_string();
        
        // Internal Alias Resolution: If target starts with HOST:, try to find the real name
        if t.starts_with("HOST:") {
            let mut masker = crate::remote::RemoteMasker::new();
            // Populate masker to find the mapping
            for name in kb.targets.keys() {
                let masked = masker.mask(name, Some("HOST"));
                if masked == t {
                    resolved_target = name.clone();
                    break;
                }
            }
        }

        println!("🔍 Deep Scanning Status for: {}", resolved_target.bold().cyan());
        if let Some(entry) = kb.get(&resolved_target) {
             // 1. Layer 4: TCP Probe (Physical Life)
             let port = entry.port.unwrap_or(22);
             let addr = format!("{}:{}", entry.ip, port);
             let tcp_alive = if let Ok(s_addr) = addr.parse::<std::net::SocketAddr>() {
                 std::net::TcpStream::connect_timeout(&s_addr, std::time::Duration::from_millis(500)).is_ok()
             } else {
                 false
             };

             if !tcp_alive {
                 println!("📡 Connection: {}", format!("OFFLINE (Port {} unreachable)", port).red());
                 println!("   💡 Tip: Check if the target IP is correct and the server is powered on.");
                 return;
             }

             // 2. Layer 7: SSH Handshake (Operational Life)
             match SshConnection::check_connection(&entry.ip, entry.user.as_deref(), entry.port) {
                 Ok(_) => {
                     println!("📡 Connection: {}", "ONLINE (SSH Verified)".green());
                     
                     // 1. OS Info
                     let os = SshConnection::detect_os(&entry.ip, entry.user.as_deref(), entry.port);
                     println!("🐧 OS: {}", os.as_deref().unwrap_or("Unknown").yellow());

                     // 2. Kernel & Disk via combined SSH probe
                     let cmd = "uname -r && df -h / --output=size,used,avail,pcent | tail -1";
                     match SshConnection::execute_output(&entry.ip, entry.user.as_deref(), entry.port, cmd) {
                         Ok(output) => {
                             let lines: Vec<&str> = output.lines().collect();
                             if lines.len() >= 2 {
                                 println!("⚙️  Kernel: {}", lines[0].trim().blue());
                                 let disk = lines[1].trim().split_whitespace().collect::<Vec<_>>();
                                 if disk.len() >= 4 {
                                     println!("💾 Disk (/): Total: {}, Used: {} ({}), Avail: {}", 
                                         disk[0], disk[1], disk[3].red(), disk[2].green());
                                 }
                             }
                         },
                         Err(_) => println!("⚠️  Failed to fetch detailed metrics."),
                     }

                     // 3. Service Discovery (Active Ports)
                     println!("\n🔍 Active Services (Service Discovery):");
                     let common_ports = vec![80, 443, 8080, 11434, 3000, 5432, 6379, 27017];
                     let mut active_ports = Vec::new();
                     for p in common_ports {
                         let addr = format!("{}:{}", entry.ip, p);
                         if let Ok(s_addr) = addr.parse::<std::net::SocketAddr>() {
                             if std::net::TcpStream::connect_timeout(&s_addr, std::time::Duration::from_millis(100)).is_ok() {
                                 let svc = match p {
                                     80 | 443 | 8080 => "HTTP/Web",
                                     11434 => "AI/Ollama",
                                     3000 => "Node/App",
                                     5432 => "PostgreSQL",
                                     6379 => "Redis",
                                     27017 => "MongoDB",
                                     _ => "Unknown",
                                 };
                                 active_ports.push(format!("{}({})", p.to_string().yellow(), svc.dimmed()));
                             }
                         }
                     }
                     if active_ports.is_empty() {
                         println!("   - No other common services detected.");
                     } else {
                         println!("   ✅ Found: {}", active_ports.join(", "));
                     }
                 },
                 Err((code, err)) => {
                      println!("📡 Connection: {}", "ONLINE (Physical) but SSH FAILED on registered port".yellow());
                      let diagnosis = SshConnection::diagnose(code, &err);
                      println!("   ❌ Error: {}", diagnosis.message.red());
                      
                      // 🧠 Milestone v0.0.13.14: Smart Port Balancer
                      let fallback_ports = vec![22, 2222];
                      let mut found_fallback = false;

                      for fp in fallback_ports {
                          if Some(fp) == entry.port { continue; }
                          println!("   🔄 [Adaptive Routing] Probing Standard Port {} for SSH Fingerprint...", fp);
                          
                          if SshConnection::verify_ssh_fingerprint(&entry.ip, fp) {
                              println!("   ✅ [Fingerprint Match] Found valid SSH-2.0 banner on Port {}!", fp);
                              println!("   🚀 [Temporary Override] SSH Connected via Port {} (Adaptive)", fp);
                              
                              if SshConnection::check_connection(&entry.ip, entry.user.as_deref(), Some(fp)).is_ok() {
                                  let os = SshConnection::detect_os(&entry.ip, entry.user.as_deref(), Some(fp));
                                  println!("🐧 OS: {} (via Port {})", os.as_deref().unwrap_or("Unknown").yellow(), fp);
                                  
                                  let cmd = "uname -r && df -h / --output=size,used,avail,pcent | tail -1";
                                  if let Ok(output) = SshConnection::execute_output(&entry.ip, entry.user.as_deref(), Some(fp), cmd) {
                                      let lines: Vec<&str> = output.lines().collect();
                                      if lines.len() >= 2 {
                                          println!("⚙️  Kernel: {}", lines[0].trim().blue());
                                          let disk = lines[1].trim().split_whitespace().collect::<Vec<_>>();
                                          if disk.len() >= 4 {
                                              println!("💾 Disk (/): Total: {}, Used: {} ({}), Avail: {}", 
                                                  disk[0], disk[1], disk[3].red(), disk[2].green());
                                          }
                                      }
                                  }
                                  println!("\n🔍 [KISS Sync] Actual SSH is active on Port {}.", fp.to_string().cyan());
                                  println!("   Would you like to update the Knowledge Base? Run: 'vega add-node {}:{}'", entry.ip, fp);
                                  found_fallback = true;
                                  break;
                              }
                          }
                      }

                      if !found_fallback {
                          println!("   ❌ [Scan Exhausted] No active SSH found on standard fallback ports.");
                          println!("   💡 Action: {}", diagnosis.recommendation.dimmed());
                      }
                  }
             }
        } else {
             println!("❌ Target '{}' not found in inventory.", t);
        }
        return;
    }

    println!("📊 Vega Fleet Status");
    println!(
        "{:<20} | {:<15} | {:<6} | {:<10} | {:<10} | {:<15}",
        "Target (AI Alias)", "IP Address", "Port", "Status", "Load", "Tags"
    );
    println!("{:-<20}-|-{:-<15}-|-{:-<6}-|-{:-<10}-|-{:-<10}-|-{:-<15}", "", "", "", "", "", "");

    let mut masker = crate::remote::RemoteMasker::new();

    // 1. Registered Nodes
    for (name, entry) in &kb.targets {
        let load = entry.cpu_load.as_deref().unwrap_or("?");
        let tags = if entry.tags.is_empty() {
            "-".to_string()
        } else {
            entry.tags.join(",")
        };
        
        let masked_name = masker.mask(name, Some(if entry.protocol == "ssh" { "HOST" } else { "STORAGE" }));
        
        // 1. Management Port (From KB)
        let mgmt_port = entry.port.unwrap_or(22);
        let is_mgmt_alive = if let Ok(addr) = format!("{}:{}", entry.ip, mgmt_port).parse::<std::net::SocketAddr>() {
            std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(300)).is_ok()
        } else {
            // Fallback for names
            match (entry.ip.as_str(), mgmt_port).to_socket_addrs() {
                Ok(mut iter) => {
                    if let Some(addr) = iter.next() {
                        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(300)).is_ok()
                    } else { false }
                }
                Err(_) => false
            }
        };

        let status_color = if is_mgmt_alive { "ONLINE".green() } else { "OFFLINE".red() };
        println!(
            "{:<20} | {:<15} | {:<6} | {:<10} | {:<10} | {}",
            masked_name.bold().cyan(), 
            entry.ip, 
            mgmt_port.to_string().yellow(),
            status_color,
            load,
            format!("{} (Primary)", tags).yellow()
        );

        // 2. Extra Service Discovery (Quick Probe for Dashboard)
        let extra_ports = vec![22, 11434, 80, 443, 8080, 3000, 5432, 6379, 27017];
        for p in extra_ports {
            if p == mgmt_port { continue; } // Skip if already shown
            
            let addr_str = format!("{}:{}", entry.ip, p);
            if let Ok(addr) = addr_str.parse::<std::net::SocketAddr>() {
                if std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(100)).is_ok() {
                    let svc_label = match p {
                        22 => "SSH",
                        11434 => "AI/Ollama",
                        80 | 443 | 8080 => "HTTP/Web",
                        3000 => "Node/App",
                        5432 => "PostgreSQL",
                        6379 => "Redis",
                        27017 => "MongoDB",
                        _ => "Service",
                    };
                    println!(
                        "{:<20} | {:<15} | {:<6} | {:<10} | {:<10} | {}",
                        masked_name.dimmed(), 
                        entry.ip, 
                        p.to_string().cyan(),
                        "ONLINE".green(),
                        "-",
                        svc_label.italic().dimmed()
                    );
                }
            }
        }
    }

    // 2. Discovered Nodes
    let ctx = SystemContext::collect();
    let mut discovered_count = 0;
    for node in &ctx.remotes {
        if node.r#type == crate::context::RemoteType::Host &&
           !kb.targets.values().any(|v| v.ip == node.real_name) && 
           !kb.targets.contains_key(&node.name) {
            if discovered_count == 0 {
                println!("\n🔍 Discovered (Not Registered):");
            }
            println!(
                "   📡 {:<17} | {:<15} | {}",
                node.name.yellow(), node.real_name, "Run 'vega refresh' to register".italic().dimmed()
            );
            discovered_count += 1;
        }
    }

    println!("\nTotal Managed Nodes: {}", kb.targets.len());

    // Usage Analytics Integration
    if let Ok(db) = Database::new() {
        if let Ok(sessions) = db.get_recent_sessions(5) {
            let data: Vec<(String, i32)> = sessions
                .into_iter()
                .map(|(id, weight)| (format!("Session #{}", id), weight))
                .collect();

            if !data.is_empty() {
                println!("\n📈 Project Activity (Last 5 Sessions):");
                println!("{}", Analytics::render_bar_chart(&data));
            }
        }
    }
}
