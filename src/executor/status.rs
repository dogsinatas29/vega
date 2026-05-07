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
                 },
                 Err((code, err)) => {
                     println!("📡 Connection: {}", "ONLINE (Physical) but SSH FAILED".yellow());
                     let diagnosis = SshConnection::diagnose(code, &err);
                     println!("   ❌ Error: {}", diagnosis.message.red());
                     println!("   💡 Action: {}", diagnosis.recommendation.dimmed());
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
        
        // Real-time Health Probe (Senior's Secret Sauce)
         let port = entry.port.unwrap_or(22);
         let is_alive = if let Ok(addr) = format!("{}:{}", entry.ip, port).parse::<std::net::SocketAddr>() {
             std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(300)).is_ok()
         } else {
             // Fallback for names that are not IPs (e.g. Hostnames in /etc/hosts)
             std::net::TcpStream::connect_timeout(&(entry.ip.as_str(), port).to_socket_addrs().map(|mut i| i.next().unwrap()).unwrap(), std::time::Duration::from_millis(300)).is_ok()
         };

        let status_color = if is_alive {
            "ONLINE".green()
        } else {
            "OFFLINE".red()
        };

        let masked_name = masker.mask(name, Some(if entry.protocol == "ssh" { "HOST" } else { "STORAGE" }));
        let port = entry.port.unwrap_or(22).to_string();

        println!(
            "{:<20} | {:<15} | {:<6} | {:<10} | {:<10} | {}",
            masked_name.bold().cyan(), 
            entry.ip, 
            port.yellow(),
            status_color,
            load,
            tags.yellow()
        );
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
