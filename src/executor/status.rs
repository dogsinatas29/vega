use crate::knowledge::KnowledgeBase;
use crate::reporting::analytics::Analytics;
use crate::storage::db::Database;
use std::net::ToSocketAddrs;

pub fn show_status(kb: &KnowledgeBase) {
    use crate::context::SystemContext;
    use colored::Colorize;

    println!("📊 Vega Fleet Status");
    println!(
        "{:<20} | {:<15} | {:<10} | {:<10} | {:<15}",
        "Target (AI Alias)", "IP Address", "Status", "Load", "Tags"
    );
    println!("{:-<20}-|-{:-<15}-|-{:-<10}-|-{:-<10}-|-{:-<15}", "", "", "", "", "");

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
        // Try to connect to port 22 (SSH) with a very short timeout
        let is_alive = if let Ok(addr) = format!("{}:22", entry.ip).parse::<std::net::SocketAddr>() {
            std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(300)).is_ok()
        } else {
            // Fallback for names that are not IPs (e.g. Hostnames in /etc/hosts)
            std::net::TcpStream::connect_timeout(&(entry.ip.as_str(), 22).to_socket_addrs().map(|mut i| i.next().unwrap()).unwrap(), std::time::Duration::from_millis(300)).is_ok()
        };

        let status_color = if is_alive {
            "ONLINE".green()
        } else {
            "OFFLINE".red()
        };

        let masked_name = masker.mask(name, Some(if entry.protocol == "ssh" { "HOST" } else { "STORAGE" }));

        println!(
            "{:<20} | {:<15} | {:<10} | {:<10} | {}",
            masked_name.bold().cyan(), 
            entry.ip, 
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
