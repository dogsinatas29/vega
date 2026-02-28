use crate::knowledge::KnowledgeBase;
use crate::reporting::analytics::Analytics;
use crate::storage::db::Database;

pub fn show_status(kb: &KnowledgeBase) {
    println!("📊 Vega Fleet Status");
    println!(
        "{:<15} | {:<15} | {:<10} | {:<25}",
        "Target", "IP Address", "OS Type", "Last Verified"
    );
    println!("{:-<15}-|-{:-<15}-|-{:-<10}-|-{:-<25}", "", "", "", "");

    for (name, entry) in &kb.targets {
        let os = entry.os_type.as_deref().unwrap_or("?");
        println!(
            "{:<15} | {:<15} | {:<10} | {}",
            name, entry.ip, os, entry.last_success
        );
    }
    println!("\nTotal Nodes: {}", kb.targets.len());

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
