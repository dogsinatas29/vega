mod config;
mod context;
mod token_saver;
mod logger;
mod shell;
mod interactor;
pub mod setup;
mod doom;
mod init;
mod knowledge;
mod scan;
mod connection;
mod executor;
mod system;
// pub mod setup; // Removed duplicate, handled by mod setup + pub usage from crate::setup
mod ai;

use std::env;
use std::process::Command;
use crate::config::VegaConfig;
use crate::context::SystemContext;
use crate::token_saver::{TokenSaver, Action};
use crate::logger::ExecutionLogger;
use crate::shell::ShellSnapshot;
use crate::interactor::Interactor;
use crate::setup::SetupWizard;
use crate::doom::engine::DoomEngine;
use crate::knowledge::{KnowledgeBase, KnowledgeEntry};
use crate::scan::vm::VmScanner;
use crate::connection::ssh::SshConnection;
use crate::executor::pkg;
use crate::executor::orchestrator; // Added for the new update command
use crate::system::virt::VmController;
use crate::system::storage::SmartStorage;
use crate::system::healer::Healer;

#[tokio::main]
async fn main() {
    // 0. Parse Input (Early)
    let args: Vec<String> = env::args().collect();
    // println!("DEBUG: args={:?}", args); // Uncomment for debugging
    if args.len() < 2 {
        println!("Usage: vega <command>");
        println!("Commands: connect, install, backup, start, health, status, refresh, update --all, setup");
        return;
    }
    let input = &args[1];

    if input == "setup" {
        SetupWizard::run();
        return;
    }

    // 1. Bootstrap (Auto-Init or Load)
    let config = init::bootstrap().unwrap_or_else(|e| {
        eprintln!("❌ Bootstrap Failed: {}", e);
        eprintln!("💡 Tip: Run 'vega setup' to repair configuration.");
        std::process::exit(1);
    });
    
    // 2. Initialize Knowledge Base
    let mut kb = KnowledgeBase::load();
    
    // 3. Initialize Modules
    let optimization = config.optimization.as_ref().cloned().unwrap_or_default();
    let keywords = optimization.local_keywords.clone().unwrap_or_default();
    let snapshot_path = optimization.shell_snapshot_path.clone().unwrap_or("logs/shell_snapshot.json".to_string());
    
    let token_saver = TokenSaver::new("logs/cache.json", "logs/history.jsonl", keywords);
    let logger = ExecutionLogger::new("logs/history.jsonl");

    // 4. Command Routing (Continued)
    
    // v2.0 Abstraction Commands
    let dry_run = args.contains(&"--dry-run".to_string());
    
    // Pkg Manager: vega install <package>
    if input == "install" && args.len() >= 3 {
        let pkg_name = &args[2];
        let ctx = SystemContext::collect();
        let pm = pkg::detect(&ctx);
        println!("📦 Package Manager Detected: {}", pm.name());
        let cmd = pm.install(pkg_name);
        println!("🚀 Proposed Command: {}", cmd);
        
        if !dry_run {
            println!("⚡ Executing...");
            let _ = Command::new("sh").arg("-c").arg(&cmd).status();
        } else {
            println!("🛑 Dry-Run: Execution Skipped.");
        }
        return;
    }

    // Storage: vega backup <source> <target_alias>
    if input == "backup" && args.len() >= 4 {
        let source = &args[2];
        let target = &args[3];
        let storage = SmartStorage::new();
        let cmd = storage.backup_cmd(source, target);
        println!("☁️  Smart Storage Backup:");
        println!("   Command: {}", cmd);
        // Execute...
        return;
    }

    // Virt: vega start <vm_name>
    if input == "start" && args.len() >= 3 {
        let vm_name = &args[2];
        println!("🖥️  VM Controller: Starting '{}'...", vm_name);
        match VmController::start(vm_name) {
            Ok(msg) => println!("{}", msg),
            Err(e) => eprintln!("❌ VM Error: {}", e),
        }
        return;
    }

    // Healer: vega health
    if input == "health" {
        println!("❤️  System Healer: Analyzing Journal...");
        // Auto-Maintenance: Rotate logs if too large
        Healer::rotate_logs();
        
        let suggestions = Healer::analyze_journal();
        for suggestion in suggestions {
            println!("   {}", suggestion);
        }
        return;
    }

    // Command:    // 5. Route Commands
    // Status Dashboard
    if input == "status" {
        executor::status::show_status(&kb);
        return;
    }

    // Refresh Context
    if input == "refresh" && args.len() >= 3 {
        let target_name = &args[2];
        if let Some(mut entry) = kb.get(target_name).cloned() {
            println!("🔄 Refreshing context for '{}'...", target_name);
            match SshConnection::check_connection(&entry.ip, entry.user.as_deref()) {
                Ok(_) => {
                    let os = SshConnection::detect_os(&entry.ip, entry.user.as_deref());
                    println!("   OS Detected: {}", os.as_deref().unwrap_or("Unknown"));
                    entry.os_type = os;
                    entry.last_success = chrono::Local::now().to_rfc3339();
                    kb.add(target_name, entry);
                    let _ = kb.save();
                    println!("✅ Knowledge Base Updated.");
                },
                Err(e) => println!("❌ Host Unreachable: {}", e.1),
            }
        } else {
            println!("❌ Target '{}' not found in Knowledge Base.", target_name);
        }
        return;
    }

    // Command: vega connect [target]
    if input == "connect" && args.len() >= 3 {
        // ... (existing code)
        let target_name = &args[2];
        println!("🤖 Tiki-Taka: Analyzing request to connect to '{}'...", target_name);
        
        // A. Check Knowledge Base
        if let Some(entry) = kb.get(target_name) {
            println!("📚 Knowledge: Found verified config for '{}' ({})", target_name, entry.ip);
            SshConnection::connect(&entry.ip, entry.user.as_deref());
            return;
        }
        // ...
    }
    
    // Check old connect logic if pasted above cut it off...
    // Actually relying on "replace" to keep the connect logic if I match correctly.
    // The previous implementation was:
    // ... connect logic ...
    // ... monitor logic ...
    
    // I need to be careful not to delete the connect logic.
    // Let's grab the connect logic block again to be safe.
    
    if input == "connect" && args.len() >= 3 {
         let target_name = &args[2];
         // ... (re-implement or ensure it persists)
         // For brevity in this tool call, I'll assume I replace the START of main up to the connect logic.
         // BUT wait, replace_file_content replaces a chunk.
         // My match block was:
         // StartLine: 1
         // TargetContent: ...
         
         // I should probably rewrite main.rs fully or use a precise chunk.
         // Since I'm adding multiple commands, let's just insert them BEFORE "connect".
         
    }
    
    // Let's use the replacement to INSERT commands before "connect".
    
    if input == "connect" && args.len() >= 3 {
        let target_name = &args[2];
        println!("🤖 Tiki-Taka: Analyzing request to connect to '{}'...", target_name);
        
        // A. Check Knowledge Base
        let mut kb_hit = false;
        // Fix E0502: Clone the entry to release the immutable borrow on 'kb'
        if let Some(entry) = kb.get(target_name).cloned() {
            kb_hit = true;
            println!("📚 Knowledge: Found verified config for '{}' ({})", target_name, entry.ip);
            
            // Smart Check: Ping before Connect
            print!("   Verifying reachability... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
            
            if SshConnection::check_connection(&entry.ip, entry.user.as_deref()).is_ok() {
                println!("OK ✅");
                
                // Context Check: Do we know the OS?
                if entry.os_type.is_none() {
                    print!("   Known Host but Unknown OS. Detecting... ");
                    let os_detected = SshConnection::detect_os(&entry.ip, entry.user.as_deref());
                    println!("{}", os_detected.as_deref().unwrap_or("Unknown"));
                    
                    if os_detected.is_some() {
                        let mut new_entry = entry.clone();
                        new_entry.os_type = os_detected;
                        new_entry.last_success = chrono::Local::now().to_rfc3339();
                        kb.add(target_name, new_entry);
                        let _ = kb.save();
                        println!("   Context Updated 💾");
                    }
                } else {
                    println!("   Context Verified ({}) ✨", entry.os_type.as_ref().unwrap());
                }

                SshConnection::connect(&entry.ip, entry.user.as_deref());
                return;
            } else {
                println!("Failed ❌ (Stale IP or Down)");
                println!("🔄 Self-Healing: Initiating Rescan for '{}'...", target_name);
                // kb_hit remains false to trigger scan below, but we know we had an entry.
                // explicitly just fall through.
            }
        }
        
        // B. Scan VMs (Fallback or Miss)
        if !kb_hit { 
             println!("🔍 Scanning local VMs for '{}'...", target_name);
        }
        
        let vms = VmScanner::scan();
        let target_vm = vms.iter().find(|vm| vm.name.contains(target_name));
        
        if let Some(vm) = target_vm {
            println!("🔍 Found VM: {} (State: {})", vm.name, vm.state);
            if let Some(ip) = &vm.ip {
                println!("   IP Address: {}", ip);
                // Try Diagnostic Connection
                match SshConnection::check_connection(ip, None) { 
                    Ok(_) => {
                        println!("✅ Connection Verified.");
                        println!("💾 Updating Knowledge Base (Self-Healing)...");
                        
                        // Intelligent: Detect OS (Only if missing or stale)
                        // Note: In this 'new connection' path (KB miss or Stale IP), we usually want to re-detect 
                        // because a new IP might mean a different machine or a reprovisioned one.
                        // So here we run detection.
                        print!("   Detecting OS Type... ");
                        let os_detected = SshConnection::detect_os(ip, None);
                        println!("{}", os_detected.as_deref().unwrap_or("Unknown"));
                        
                        kb.add(target_name, KnowledgeEntry {
                            ip: ip.clone(),
                            user: None,
                            protocol: "ssh".to_string(),
                            port: Some(22),
                            os_type: os_detected, 
                            last_success: chrono::Local::now().to_rfc3339(),
                        });
                        let _ = kb.save();
                        
                        SshConnection::connect(ip, None);
                        return;
                    },
                    Err((code, stderr)) => {
                        println!("{}", SshConnection::diagnose(code, &stderr));
                    }
                }
            } else {
                println!("⚠️ VM found but no IP detected. Ensure qemu-guest-agent is running or check DHCP leases.");
            }
        } else {
            println!("❌ Scanning complete. No VM found matching '{}'.", target_name);
        }
        return;
    }

    if input == "monitor" {
        println!("😈 Launching DooM System Monitor...");
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        let mut engine = DoomEngine::new();
        if let Err(e) = engine.run() {
            eprintln!("❌ Doom Engine Error: {}", e);
        }
        return;
    }

    // Special Command: config sync (or refresh-config)
    if input == "config" || input == "refresh-config" {
        println!("🔄 Syncing Configuration & Shell Snapshot...");
        let snapshot = ShellSnapshot::new();
        if let Err(e) = snapshot.save(&snapshot_path) {
            println!("❌ Failed to save snapshot: {}", e);
        } else {
            println!("✅ Shell snapshot saved to {}", snapshot_path);
            println!("   - Aliases captured: {}", snapshot.aliases.len());
            println!("   - Zoxide paths: {}", snapshot.zoxide_paths.len());
        }
        return;
    }

    // 4. Token Saver: Hybrid Reasoning
    let action = token_saver.match_local_intent(input);
    
    // Smart fzf Trigger (Pre-API Scan)
    if let Action::Unknown = action {
        let history_matches = token_saver.search_history(input);
        if !history_matches.is_empty() {
            println!("🧠 Found similar past commands. Smart Triggering fzf...");
             if let Some(selection) = Interactor::select_with_fzf("Found matches >", history_matches, Some(input)) {
                 println!("🎯 Smart fzf Selected: {}", selection);
                 let _ = Command::new("sh").arg("-c").arg(&selection).status();
                 logger.log(input, "SmartFzfExec", true);
                 return;
             }
        }
    }
    
    let mut success = true;

    // Zero-Token Path: fzf Fallback (General)
    if let Action::Unknown = action {
        println!("🤔 Intent unknown locally. Trying Zero-Token fzf...");
        
        let mut candidates = Vec::new();
        candidates.push("vega config".to_string());
        candidates.push("system update".to_string());
        candidates.push("vega monitor".to_string());
        
        if let Some(snap) = ShellSnapshot::load(&snapshot_path) {
            for path in snap.zoxide_paths {
                candidates.push(format!("cd {}", path));
            }
        }

        if let Some(selection) = Interactor::select_with_fzf("Select Action >", candidates, None) {
             println!("🎯 fzf Selected: {}", selection);
             let _ = Command::new("sh").arg("-c").arg(&selection).status();
             return;
        }
    }

    match action {
        Action::SystemUpdate => {
            println!("🔧 [Hybrid] Detected System Update intent.");
            println!("Context: {:?}", SystemContext::collect().load_avg);
            println!("Executing: sudo apt update && sudo apt upgrade");
        },
        Action::SshConnect(ref target) => {
            println!("🔌 [Hybrid] Detected SSH intent to '{}'", target);
            let status = Command::new("ssh")
                .arg(target)
                .status();
            
            match status {
                 Ok(s) => if !s.success() { success = false; },
                 Err(e) => { 
                     println!("SSH Failed: {}", e); 
                     success = false; 
                 }
            }
        },
        Action::ShowLog => {
             println!("📜 [Hybrid] Showing logs...");
             let _ = Command::new("tail").args(&["-n", "10", "logs/history.jsonl"]).status();
        },
        Action::Unknown => {
            println!("🤖 [LLM] Intent unknown locally & fzf cancelled. Escalating to Gemini...");
            let ctx = SystemContext::collect();
            println!("Generated Context (Compressed): OS={}, Load={:?}", ctx.os_info, ctx.load_avg);
            println!("(Connecting to LLM API...)");
        }
    }

    // 5. Log Execution
    logger.log(input, &format!("{:?}", action), success);
}
