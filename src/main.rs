use vega::*;

use std::env;
use std::process::Command;

use crate::context::SystemContext;
use crate::interactor::Interactor;
use crate::logger::ExecutionLogger;
use crate::setup::SetupWizard;
use crate::shell::ShellSnapshot;
use crate::token_saver::{Action, TokenSaver};
use colored::Colorize;

use crate::connection::ssh::SshConnection;
use crate::executor::pkg;
use crate::knowledge::{KnowledgeBase, KnowledgeEntry};
use crate::system::virt::VmScanner;

use crate::system::healer::Healer;
use crate::system::storage::SmartStorage;
use crate::system::virt::VmController;

#[tokio::main]
async fn main() {
    // 0. Parse Input (Early)
    let args: Vec<String> = env::args().collect();
    // println!("DEBUG: args={:?}", args); // Uncomment for debugging
    if args.len() < 2 {
        println!("Usage: vega <command>");
        println!("Commands: connect, install, backup, start, health, status, refresh, update --all, setup, login, history");
        return;
    }
    let input = &args[1];
    let full_input = args[1..].join(" ");
    let full_input = full_input.trim();

    if input == "setup" {
        if args.contains(&"--cookie".to_string()) {
            SetupWizard::setup_cookie();
        } else {
            SetupWizard::run();
        }
        return;
    }

    if input == "reset" {
        if args.contains(&"--all".to_string()) {
            println!("🧹 Performing Hard Reset...");

            // 1. Wipe Config
            let config_dir = dirs::config_dir().map(|mut p| {
                p.push("vega");
                p
            });
            if let Some(path) = config_dir {
                let _ = std::fs::remove_dir_all(&path);
                println!("   ✅ Config wiped: {:?}", path);
            }

            // 2. Wipe Cache/Data
            let data_dir = dirs::data_local_dir().map(|mut p| {
                p.push("vega");
                p
            });
            if let Some(path) = data_dir {
                let _ = std::fs::remove_dir_all(&path);
                println!("   ✅ Data/Cache wiped: {:?}", path);
            }

            println!("✨ System reset complete. Please run 'vega setup' to re-initialize.");
        } else {
            println!("⚠️  Usage: vega reset --all");
        }
        return;
    }

    if input == "update" {
        if args.contains(&"--all".to_string()) {
            println!("🛠️  [SRE Fallback] Performing System Update...");
            let status = Command::new("sudo").arg("apt").arg("update").status();

            if status.is_ok() {
                let _ = Command::new("sudo")
                    .arg("apt")
                    .arg("upgrade")
                    .arg("-y")
                    .status();
            }
            println!("✅ System update attempt complete.");
        } else {
            println!("⚠️  Usage: vega update --all");
        }
        return;
    }

    if input == "login" {
        println!("🔐 Starting Google OAuth Login...");
        match crate::auth::google::login().await {
            Ok(_) => println!("✅ Login successful! Token saved."),
            Err(e) => eprintln!("❌ Login failed: {}", e),
        }
        return;
    }

    if input == "history" {
        // ... (existing history code)
    }

    if input == "debug-keyring" {
        println!("🔍 Keyring Diagnostic Mode");
        crate::security::keyring::debug_persistence();
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
        .map(|mut p| {
            p.push("vega");
            p
        })
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

    // Sync: vega sync
    if input == "sync" {
        println!("🔄 Initiating Global Cloud Sync...");
        let ctx = SystemContext::collect();
        let primary = config
            .optimization
            .as_ref()
            .and_then(|o| o.primary_remote.clone());
        if let Err(e) = executor::orchestrator::sync_all_cloud(&ctx, primary).await {
            eprintln!("❌ Sync Failed: {}", e);
        } else {
            println!("✅ Global Sync Completed.");
        }
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

    // Reporting: vega report [--session <id>]
    if input == "report" {
        let session_id = args
            .iter()
            .position(|r| r == "--session")
            .and_then(|p| args.get(p + 1))
            .and_then(|s| s.parse::<i64>().ok());

        let sid = session_id.unwrap_or_else(|| {
            // Default to current or latest session
            if let Ok(db) = crate::storage::db::Database::new() {
                db.get_current_session_id().unwrap_or(0)
            } else {
                0
            }
        });

        let use_markdown =
            args.contains(&"--markdown".to_string()) || args.contains(&"-m".to_string());

        if use_markdown {
            println!("📝 Generating Markdown Report for Session {}...", sid);
            match crate::reporting::pdf::PdfEngine::generate_markdown_report(sid).await {
                Ok(path) => println!("✅ Markdown Report saved: {}", path),
                Err(e) => eprintln!("❌ Report Failed: {}", e),
            }
        } else {
            println!("📊 Generating PDF Report for Session {}...", sid);
            match crate::reporting::pdf::PdfEngine::generate_report(sid).await {
                Ok(path) => println!("✅ PDF Report saved: {}", path),
                Err(e) => eprintln!("❌ Report Failed: {}", e),
            }
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
                }
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
        println!(
            "🤖 [VEGA] Analyzing connection request for '{}'...",
            target_name
        );

        // 1. Resolve & Persist: Check Internal State (KB)
        let mut kb_hit = false;
        if let Some(entry) = kb.get(target_name).cloned() {
            kb_hit = true;
            println!(
                "📚 State DB: Found entry for '{}' ({})",
                target_name, entry.ip
            );

            print!("   Verifying reachability... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();

            if SshConnection::check_connection(&entry.ip, entry.user.as_deref()).is_ok() {
                println!("OK ✅");
                SshConnection::connect(&entry.ip, entry.user.as_deref());
                return;
            } else {
                println!("Failed ❌ (Stale or Unreachable)");
                println!(
                    "🔄 Silent Discovery: Initiating live scan for '{}'...",
                    target_name
                );
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
                    kb.add(
                        target_name,
                        KnowledgeEntry {
                            ip: ip.to_string(),
                            user: None,
                            protocol: "ssh".to_string(),
                            port: Some(22),
                            os_type: os_detected,
                            last_success: chrono::Local::now().to_rfc3339(),
                        },
                    );
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
            println!(
                "❌ Discovery Failed: No target found matching '{}'.",
                target_name
            );
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
                if let Some(selection) = Interactor::select_with_fzf(
                    "Found matches >",
                    history_matches,
                    Some(full_input),
                ) {
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

            if let Some(selection) =
                Interactor::select_with_fzf("Select Action >", candidates, None)
            {
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
        }
        Action::SshConnect(ref target) => {
            println!("🔌 [Hybrid] Detected SSH intent to '{}'", target);
            let status = Command::new("ssh").arg(target).status();

            match status {
                Ok(s) => {
                    if !s.success() {
                        success = false;
                    }
                }
                Err(e) => {
                    println!("SSH Failed: {}", e);
                    success = false;
                }
            }
        }
        Action::ShowLog => {
            println!("📜 [Hybrid] Showing logs...");
            let _ = Command::new("tail")
                .args(&["-n", "10", "logs/history.jsonl"])
                .status();
        }
        Action::Unknown => {
            // Intelligent Fallback: AI or fzf?
            // If input has spaces or is long, assume natural language -> AI
            // If input is short and single word without spaces -> fzf (typo likely)

            let is_complex = full_input.contains(' ') || full_input.len() > 10;

            if is_complex {
                println!("🤖 [VEGA] Analyzing natural language request...");
                println!("   Input: \"{}\"", full_input);

                // Collect context for the AI
                let ctx = SystemContext::collect();
                let preferred_engine = config.ai.as_ref().map(|a| a.provider.clone());

                // Call async generate_with_fallback
                match crate::ai::router::SmartRouter::generate_with_fallback(
                    &ctx,
                    full_input,
                    preferred_engine,
                )
                .await
                {
                    Ok(response_str) => {
                        // Try to parse as JSON
                        use crate::ai::{AiResponse, RiskLevel};
                        match serde_json::from_str::<AiResponse>(&response_str) {
                            Ok(ai_res) => {
                                println!("📝 Explanation: {}", ai_res.explanation);

                                // Colorize based on risk
                                let risk_display = match ai_res.risk_level {
                                    RiskLevel::INFO => "INFO".green(),
                                    RiskLevel::WARNING => "WARNING".yellow(),
                                    RiskLevel::CRITICAL => "CRITICAL".red().bold(),
                                };
                                println!("⚠️  Risk Level: {}", risk_display);

                                if !ai_res.command.is_empty() {
                                    println!("   > Command: {}", ai_res.command.green().bold());

                                    if Interactor::confirm("Execute this command?") {
                                        println!("⚡ Executing...");

                                        // RE-RESOLVE REMOTE NAMES (Unmasking)
                                        let ctx = SystemContext::collect();
                                        let mut masker = crate::remote::RemoteMasker::new();
                                        // Discovery::run() returns real names. We need to mask them to get the same REMOTE_XX mapping.
                                        if let Ok(discovery) =
                                            crate::system::discovery::Discovery::run()
                                        {
                                            for remote in discovery.cloud_remotes {
                                                let _ = masker.mask(&remote);
                                            }
                                        }
                                        let mut final_cmd = masker.resolve_command(&ai_res.command);

                                        if final_cmd != ai_res.command {
                                            println!("   🔗 [Resolved] {}", final_cmd.cyan());
                                        }

                                        // Internal Pruning Logic: Auto-inject blacklist for find
                                        if final_cmd.trim().starts_with("find ")
                                            && !final_cmd.contains("-prune")
                                        {
                                            let enriched = {
                                                let parts: Vec<&str> =
                                                    final_cmd.split_whitespace().collect();
                                                if parts.len() > 2 {
                                                    let path = parts[1];
                                                    let mut prune_rules = Vec::new();
                                                    for b_path in crate::system::SRE_BLACKLIST {
                                                        prune_rules.push(format!(
                                                            "-path '{}' -prune",
                                                            b_path
                                                        ));
                                                    }
                                                    let prune_str = prune_rules.join(" -o ");
                                                    let original_expr = parts[2..].join(" ");
                                                    Some(format!(
                                                        "find {} \\( {} \\) -prune -o \\( {} -print \\)",
                                                        path, prune_str, original_expr
                                                    ))
                                                } else {
                                                    None
                                                }
                                            };

                                            if let Some(new_cmd) = enriched {
                                                final_cmd = new_cmd;
                                                println!("   🛡️  [SRE Protection] Applied internal pruning rules.");
                                            }
                                        }

                                        let status = Command::new("sh")
                                            .arg("-c")
                                            .arg(&final_cmd)
                                            .stderr(std::process::Stdio::null()) // Sovereign SRE: Suppress "Permission denied" noise
                                            .status();

                                        match status {
                                            Ok(s) => {
                                                if s.success() {
                                                    println!("✅ Execution Successful.");
                                                } else if s.code() == Some(1)
                                                    && final_cmd.contains("find ")
                                                {
                                                    println!("✅ Search completed (system/protected paths skipped).");
                                                } else {
                                                    println!(
                                                        "❌ Execution Failed (Exit Code: {:?})",
                                                        s.code()
                                                    );
                                                }
                                            }
                                            Err(e) => println!("❌ Failed to spawn shell: {}", e),
                                        }
                                    } else {
                                        println!("🚫 Aborted by user.");
                                    }
                                } else {
                                    println!("ℹ️  No command to execute.");
                                }
                            }
                            Err(_) => {
                                // Fallback: Raw text response
                                println!("📝 Response (Raw):\n{}", response_str);
                            }
                        }
                    }
                    Err(e) => eprintln!("❌ AI Error: {}", e),
                }
                return; // handled by AI
            }

            // Fallthrough to fzf logic below for simple typos (e.g. "updtae")
            println!("🤔 Intent unknown locally. Trying Zero-Token fzf...");
        }
    }

    // 5. Log Execution
    logger.log(full_input, &format!("{:?}", action), success);

    // 6. Session Completion Auto-Sync Hook
    if config
        .optimization
        .as_ref()
        .and_then(|o| o.auto_sync)
        .unwrap_or(false)
    {
        let ctx = SystemContext::collect();
        if !ctx.cloud_nodes.is_empty() {
            println!("🔄 Auto-Syncing session state to cloud...");
            let primary = config
                .optimization
                .as_ref()
                .and_then(|o| o.primary_remote.clone());
            let _ = executor::orchestrator::sync_all_cloud(&ctx, primary).await;
        }
    }
}
