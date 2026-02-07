use crate::knowledge::KnowledgeBase;

pub fn show_status(kb: &KnowledgeBase) {
    println!("📊 Vega Fleet Status");
    println!("{:<15} | {:<15} | {:<10} | {:<25}", "Target", "IP Address", "OS Type", "Last Verified");
    println!("{:-<15}-|-{:-<15}-|-{:-<10}-|-{:-<25}", "", "", "", "");

    for (name, entry) in &kb.targets {
        let os = entry.os_type.as_deref().unwrap_or("?");
        // Simple heuristic for status check could be added here (e.g. ping with timeout=1)
        // For now, just show DB state
        println!("{:<15} | {:<15} | {:<10} | {}", 
            name, 
            entry.ip, 
            os, 
            entry.last_success
        );
    }
    println!("\nTotal Nodes: {}", kb.targets.len());
}
