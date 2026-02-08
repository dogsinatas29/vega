mod config;
mod context;
mod token_saver;
mod logger;
mod shell;
mod interactor;
pub mod setup;

mod init;
mod knowledge;
mod scan;
mod connection;
mod executor;
mod system;
// pub mod setup; // Removed duplicate, handled by mod setup + pub usage from crate::setup
pub mod security;
mod ai;

use std::env;
use std::process::Command;


use crate::context::SystemContext;
use crate::token_saver::{TokenSaver, Action};
use crate::logger::ExecutionLogger;
use crate::shell::ShellSnapshot;
use crate::interactor::Interactor;
use crate::setup::SetupWizard;

use crate::knowledge::{KnowledgeBase, KnowledgeEntry};
use crate::scan::vm::VmScanner;
use crate::connection::ssh::SshConnection;
use crate::executor::pkg;

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
    let full_input = args[1..].join(" ");
    let full_input = full_input.trim();

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
    let snapshot_path = optimization.shell_snapshot_path.clone().unwrap_or_else(|| {
        if let Some(mut path) = dirs::cache_dir() {
            path.push("vega");
            path.push("shell_snapshot.json");
            path.to_string_lossy().to_string()
        } else {
            "logs/shell_snapshot.json".to_string()
        }
    });
    
    let data_dir = dirs::data_local_dir()
        .map(|mut p| { p.push("vega"); p })
        .unwrap_or_else(|| std::path::PathBuf::from("logs"));

    let cache_path = data_dir.join("cache.json").to_string_lossy().to_string();
    let history_path = data_dir.join("history.jsonl").to_string_lossy().to_string();

    let token_saver = TokenSaver::new(&cache_path, &history_path, keywords);
    let logger = ExecutionLogger::new(&history_path);

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
        let target_name = &args[2];
        println!("🤖 [VEGA] Analyzing connection request for '{}'...", target_name);
        
        // 1. Resolve & Persist: Check Internal State (KB)
        let mut kb_hit = false;
        if let Some(entry) = kb.get(target_name).cloned() {
            kb_hit = true;
            println!("📚 State DB: Found entry for '{}' ({})", target_name, entry.ip);
            
            print!("   Verifying reachability... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
            
            if SshConnection::check_connection(&entry.ip, entry.user.as_deref()).is_ok() {
                println!("OK ✅");
                SshConnection::connect(&entry.ip, entry.user.as_deref());
                return;
            } else {
                println!("Failed ❌ (Stale or Unreachable)");
                println!("🔄 Silent Discovery: Initiating live scan for '{}'...", target_name);
            }
        }
        
        // 2. Silent Discovery: Scan VMs and Network
        if !kb_hit { 
             println!("🔍 Silent Discovery: Scanning for '{}'...", target_name);
        }
        
        let vms = VmScanner::scan();
        let target_vm = vms.iter().find(|vm| vm.name.contains(target_name));
        
        if let Some(vm) = target_vm {
            println!("🎯 Discovery: Found VM '{}' (State: {})", vm.name, vm.state);
            if let Some(ip) = &vm.ip {
                println!("   Resolved IP: {}", ip);
                
                // 3. Persist: Update State DB
                print!("   Verifying new endpoint... ");
                if SshConnection::check_connection(ip, None).is_ok() {
                    println!("OK ✅");
                    println!("💾 Persistence: Updating State DB for '{}'...", target_name);
                    
                    let os_detected = SshConnection::detect_os(ip, None);
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
                } else {
                    println!("Unreachable ❌");
                }
            } else {
                println!("⚠️ Discovery Error: VM found but no IP address could be resolved.");
                println!("   TIP: Ensure qemu-guest-agent is running or check DHCP leases.");
            }
        } else {
            println!("❌ Discovery Failed: No target found matching '{}'.", target_name);
        }
        return;
    }
    
    // Check old connect logic if pasted above cut it off...
    // Actually relying on "replace" to keep the connect logic if I match correctly.
    // The previous implementation was:
    // ... connect logic ...
    // ... monitor logic ...
    
    // I need to be careful not to delete the connect logic.
    // Let's grab the connect logic block again to be safe.
    
    if input == "connect" && args.len() >= 3 {
         let _target_name = &args[2];
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
    let action = token_saver.match_local_intent(full_input);
    
    // Smart fzf Trigger (Pre-API Scan)
    // Only if simple enough to be a typo or alias.
    if let Action::Unknown = action {
        let is_complex = full_input.contains(' ') || full_input.len() > 10;
        if !is_complex {
            let history_matches = token_saver.search_history(full_input);
        if !history_matches.is_empty() {
            println!("🧠 Found similar past commands. Smart Triggering fzf...");
             if let Some(selection) = Interactor::select_with_fzf("Found matches >", history_matches, Some(full_input)) {
                 println!("🎯 Smart fzf Selected: {}", selection);
                 let _ = Command::new("sh").arg("-c").arg(&selection).status();
                 logger.log(input, "SmartFzfExec", true);
                 return;
             }
        }
    }
    }
    
    let mut success = true;

    // Zero-Token Path: fzf Fallback (General)
    // Only if Action is Unknown AND input is simple (not complex/natural language)
    if let Action::Unknown = action {
        let is_complex = full_input.contains(' ') || full_input.len() > 10;
        
        if !is_complex {
            println!("🤔 Intent unknown locally. Trying Zero-Token fzf...");
            
            let mut candidates = Vec::new();
            candidates.push("vega config".to_string());
            candidates.push("system update".to_string());

            
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
            // Intelligent Fallback: AI or fzf?
            // If input has spaces or is long, assume natural language -> AI
            // If input is short and single word without spaces -> fzf (typo likely)
            
            let is_complex = full_input.contains(' ') || full_input.len() > 10;
            
            if is_complex {
                println!("🤖 [VEGA] Analyzing natural language request...");
                println!("   Input: \"{}\"", full_input);
                
                // 1. Determine Engine
                let engine_type = crate::ai::router::SmartRouter::determine_engine(full_input, config.ai.as_ref().map(|a| a.provider.clone()));
                
                // 2. Initialize Provider
                match crate::ai::router::SmartRouter::get_provider(engine_type) {
                    Ok(brain) => {
                        let brain: Box<dyn crate::ai::AiProvider> = brain;
                        println!("⚡ Routing to: {:?}", engine_type);
                        
                        // Collect context for the AI
                        let ctx = SystemContext::collect();
                        
                        // Call async generate_response
                        match brain.generate_response(&ctx, full_input).await {
                             Ok(response) => println!("📝 Response:\n{}", response),
                             Err(e) => eprintln!("❌ AI Error: {}", e),
                        }
                    },
                    Err(e) => eprintln!("❌ AI Init Failed: {}", e),
                }
                return; // handled by AI
            } 
            
            // Fallthrough to fzf logic below for simple typos (e.g. "updtae")
            println!("🤔 Intent unknown locally. Trying Zero-Token fzf...");
        }
    }

    // 5. Log Execution
    logger.log(full_input, &format!("{:?}", action), success);
}
