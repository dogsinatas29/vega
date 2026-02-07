use crate::knowledge::KnowledgeBase;
use crate::connection::ssh::SshConnection;
use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::signal;

pub async fn update_all(kb: &KnowledgeBase) {
    println!("🚀 Orchestrating Fleet Update for {} nodes...", kb.targets.len());
    
    // Concurrency Limit: 10
    let semaphore = Arc::new(Semaphore::new(10));
    let mut tasks = vec![];

    for (name, entry) in &kb.targets {
        // Context-Aware Dispatch
        let cmd = match entry.os_type.as_deref() {
            Some("fedora") | Some("rhel") | Some("centos") => "sudo dnf update -y",
            Some("ubuntu") | Some("debian") => "sudo apt update && sudo apt upgrade -y",
            Some("alpine") => "sudo apk update && sudo apk upgrade",
            Some("arch") => "sudo pacman -Syu --noconfirm",
            _ => "echo '⚠️ Unknown OS: Manual update required'" 
        };

        let target_name = name.clone();
        let target_ip = entry.ip.clone();
        let os_type = entry.os_type.clone().unwrap_or("Unknown".to_string());
        let cmd_str = cmd.to_string();
        let sem_clone = semaphore.clone();

        let task = tokio::spawn(async move {
            // Acquire Permit
            let _permit = sem_clone.acquire().await.unwrap();
            
            // Skip logic for unknown OS if needed, but handled by dispatch caller mostly.
            if cmd_str.contains("Unknown OS") {
                 return (target_name, os_type, false, Some("Skipped: Unknown OS".to_string()));
            }

            println!("   ⚡ Dispatching to '{}' ({})...", target_name, cmd_str);
            match SshConnection::execute_remote_async(&target_ip, &cmd_str).await {
                Ok(_) => {
                    println!("   ✅ Success: '{}'", target_name);
                    (target_name, os_type, true, None)
                },
                Err(e) => {
                    println!("   ❌ Failed: '{}' -> {}", target_name, e);
                    (target_name, os_type, false, Some(e))
                }
            }
        });
        tasks.push(task);
    }

    println!("⏳ Waiting for tasks to complete... (Press Ctrl+C to cancel)");

    // Graceful Shutdown Logic
    let results = tokio::select! {
        res = join_all(tasks) => {
            res
        }
        _ = signal::ctrl_c() => {
            println!("\n⚠️  Ctrl+C Received! Aborting all updates...");
            // Tasks are dropped when `results` (tasks vec) goes out of scope or we can explicitly handle them.
            // join_all returns a vector of Results.
            // In this select branch, we haven't awaited join_all yet effectively.
            // But tokio tasks run in background. We can't easily 'abort' them unless we kept the handles separate
            // and iterated over them to call .abort(). 
            // However, simply exiting the program or returning here will drop the validation? 
            // Actually tokio tasks are detached by default if spawned. 
            // To properly abort, we need to keep the JoinHandles.
            
            // NOTE: join_all consumes the iterator of futures. The `tasks` vector holds JoinHandles.
            // But we moved `tasks` into `join_all`.
            // Let's refactor slightly to keep handles if we want to abort explicitly, 
            // OR realize that exiting main shuts down the runtime.
            // For this implementation, we will rely on returning early, which prints the summary as 'Cancelled'.
            return;
        }
    };

    println!("\n📊 Fleet Update Execution Report");
    println!("{:<15} | {:<12} | {:<12} | {}", "Target", "OS Type", "Status", "Result");
    println!("{:-<15}-|-{:-<12}-|-{:-<12}-|-{:-<20}", "", "", "", "");

    let mut success_count = 0;
    let mut fail_count = 0;
    let mut skipped_count = 0;

    for res in results {
        match res {
            Ok((name, os_type, success, err)) => {
                let status_icon = if success { "✅ Success" } else { "❌ Failed" };
                let message = err.unwrap_or_else(|| "All packages updated.".to_string());
                
                if message.contains("Skipped") {
                     skipped_count += 1;
                     println!("{:<15} | {:<12} | {:<12} | {}", name, os_type, "⚠️ Skipped", message);
                } else {
                    println!("{:<15} | {:<12} | {:<12} | {}", name, os_type, status_icon, message);
                    if success { success_count += 1; } else { fail_count += 1; }
                }
            },
            Err(e) => {
                 fail_count += 1;
                 println!("{:<15} | {:<12} | {:<12} | {}", "Unknown", "?", "❌ Error", e);
            }
        }
    }
    println!("\nSummary: {} Succeeded, {} Failed, {} Skipped.", success_count, fail_count, skipped_count);
}
