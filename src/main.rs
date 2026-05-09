use vega::*;

use std::env;
use std::fs;
use std::io::{self, Write};
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
use crate::config::VegaConfig;

use std::sync::Arc;
#[tokio::main]
async fn main() {
    // 0. Parse Input (Early)
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: vega <command>");
        println!("Commands: connect, install, backup, start, health, status, refresh, update, setup, login, history");
        return;
    }
    let input = &args[1];
    let full_input = args[1..].join(" ");
    let full_input = full_input.trim();

    // 1. Bootstrap (Auto-Init or Load)
    let _config = init::bootstrap().unwrap_or_else(|e| {
        if input != "setup" {
            eprintln!("❌ Bootstrap Failed: {}", e);
            eprintln!("💡 Tip: Run 'vega setup' to repair configuration.");
            std::process::exit(1);
        }
        VegaConfig::default() // Dummy for setup
    });

    // 2. Initialize Knowledge Base
    let mut kb = KnowledgeBase::load();

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

    if input == "search" {
        let query = args[2..].join(" ");
        if query.is_empty() {
            println!("Usage: vega search <query>");
            return;
        }
        if let Ok(db) = crate::storage::db::Database::new() {
            println!("🔍 Searching Knowledge Base for: '{}'", query.cyan());
            match db.search_knowledge(&query, 5) {
                Ok(results) => {
                    if results.is_empty() {
                        println!("   (No matches found)");
                    }
                    for (content, origin) in results {
                        println!("   [{}] {}", origin.yellow(), content);
                    }
                }
                Err(e) => eprintln!("❌ Search Failed: {}", e),
            }
        }
        return;
    }

    if input == "report" {
        if let Ok(db) = crate::storage::db::Database::new() {
            if let Some(session_id) = db.get_session_id() {
                println!("📊 Generating SRE Session Report for Session #{}...", session_id);
                let db_arc = std::sync::Arc::new(db);
                match crate::reporting::sre_report::SreReport::generate_session_summary(db_arc, session_id).await {
                    Ok(md) => {
                        let path = format!("vega_report_{}.md", session_id);
                        std::fs::write(&path, md).unwrap();
                        println!("✅ Report saved to: {}", path.green().bold());
                    }
                    Err(e) => eprintln!("❌ Report Generation Failed: {}", e),
                }
            } else {
                println!("⚠️ No active session found to report.");
            }
        }
        return;
    }

    if input == "sync" {
        if let Ok(db) = crate::storage::db::Database::new() {
            let storage = crate::system::storage::SmartStorage::new();
            let remote = db.get_metadata("default_sync_remote").unwrap_or(None)
                .unwrap_or_else(|| "구드".to_string());
            
            match storage.sync_vega_state(&remote) {
                Ok(msg) => println!("{}", msg.green().bold()),
                Err(e) => {
                    eprintln!("{}", e.red());
                    println!("💡 Tip: Ensure rclone is configured and 'default_sync_remote' is set in vega metadata.");
                }
            }
        }
        return;
    }

    if input == "update" {
        if args.contains(&"--fleet".to_string()) {
            println!("🛠️  [Fleet Management] Performing Global Maintenance...");
            let tag_filter = args.iter().position(|r| r == "--tag")
                .and_then(|idx| args.get(idx + 1).map(|s| s.as_str()));

            if let Err(e) = executor::orchestrator::update_fleet(&mut kb, tag_filter).await {
                eprintln!("❌ Fleet Update Failed: {}", e);
            } else {
                println!("✅ Fleet Maintenance Completed.");
            }
        } else if args.contains(&"--all".to_string()) {
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
            println!("⚠️  Usage: vega update --all (local) or vega update --fleet (remote)");
        }
        return;
    }

    if input == "setup" {
        if args.contains(&"--cookie".to_string()) {
            SetupWizard::setup_cookie();
        } else {
            SetupWizard::run().await;
        }
        return;
    }

    if input == "add-node" {
        println!("➕ Registering New Node for Management...");
        let name = args.get(2).cloned().unwrap_or_else(|| {
            print!("   Enter Host/IP: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        });
        
        if name.is_empty() {
            println!("❌ Host/IP cannot be empty.");
            return;
        }

        println!("   📡 Verifying connectivity to {}...", name);
        
        let (ip, port) = if name.contains(':') {
            let parts: Vec<&str> = name.split(':').collect();
            (parts[0].to_string(), parts[1].parse::<u16>().unwrap_or(22))
        } else {
            (name.clone(), 22)
        };

        if SshConnection::check_connection(&ip, None, Some(port)).is_ok() {
            let os = SshConnection::detect_os(&ip, None, Some(port));
            kb.add(&name, crate::knowledge::KnowledgeEntry {
                ip: ip,
                user: None,
                protocol: "ssh".to_string(),
                port: Some(port),
                os_type: os,
                kernel: None,
                cpu_load: None,
                tags: Vec::new(),
                password: None,
                last_success: chrono::Local::now().to_rfc3339(),
            });
            let _ = kb.save();
            println!("✅ Successfully registered node: {} (Port: {})", name, port);
        } else {
            println!("⚠️  Warning: Host is unreachable. Register anyway? (y/n)");
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            if input.trim().to_lowercase() == "y" {
                kb.add(&name, crate::knowledge::KnowledgeEntry {
                    ip: ip,
                    user: None,
                    protocol: "ssh".to_string(),
                    port: Some(port),
                    os_type: None,
                    kernel: None,
                    cpu_load: None,
                    tags: Vec::new(),
                    password: None,
                    last_success: "Never".to_string(),
                });
                let _ = kb.save();
                println!("✅ Registered node (Unverified): {} (Port: {})", name, port);
            }
        }
        return;
    }

    if input == "sync-ssh" {
        println!("🔄 Synchronizing Knowledge Base to ~/.ssh/config...");
        let mut config_block = String::from("\n");
        let mut count = 0;

        for (name, entry) in &kb.targets {
            if entry.protocol == "ssh" {
                config_block.push_str(&format!("Host {}\n", name));
                config_block.push_str(&format!("    HostName {}\n", entry.ip));
                if let Some(port) = entry.port {
                    config_block.push_str(&format!("    Port {}\n", port));
                }
                config_block.push_str("    ConnectTimeout 5\n\n");
                count += 1;
            }
        }

        if count == 0 {
            println!("⚠️  No SSH nodes found in Knowledge Base to sync.");
            return;
        }

        if let Ok(home) = env::var("HOME") {
            let ssh_dir = std::path::Path::new(&home).join(".ssh");
            let config_path = ssh_dir.join("config");

            if !ssh_dir.exists() {
                let _ = fs::create_dir_all(&ssh_dir);
            }

            let current_content = if config_path.exists() {
                fs::read_to_string(&config_path).unwrap_or_default()
            } else {
                String::new()
            };

            // Enhanced Sanitization: Robust Block Replacement
            let begin_marker = "# --- VEGA MANAGED HOSTS (AUTO-GENERATED) ---";
            let end_marker = "# --- END OF VEGA MANAGED HOSTS ---";
            
            let mut new_content = String::new();
            let mut in_vega_block = false;
            
            for line in current_content.lines() {
                if line.contains(begin_marker) { in_vega_block = true; continue; }
                if line.contains(end_marker) { in_vega_block = false; continue; }
                if !in_vega_block {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }
            
            new_content.push_str("\n");
            new_content.push_str(begin_marker);
            new_content.push_str(&config_block);
            new_content.push_str(end_marker);
            new_content.push_str("\n");

            if let Err(e) = fs::write(&config_path, new_content) {
                println!("❌ Failed to write SSH config: {}", e);
            } else {
                println!("✅ Successfully synced {} hosts to ~/.ssh/config", count);
            }
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

    // v0.0.10 Pipeline Proof of Concept
    if input == "run-v10" && args.len() >= 3 {
        let nli = args[2..].join(" ");
        println!("🚀 Running v0.0.10 Pipeline for: \"{}\"", nli);

        use crate::executor::pipeline::*;
        use crate::ai::intent::*;
        use crate::executor::template::BasicTemplateBuilder;
        use crate::ai::generator::AiOptionGenerator;
        use crate::executor::virt::BasicVee;
        use crate::safety::risk::DefaultRiskEvaluator;

        let orchestrator = PipelineOrchestrator {
            intent_resolver: Box::new(HybridIntentResolver {
                local: LocalIntentResolver,
                ai: AiIntentResolver,
            }),
            template_builder: Box::new(BasicTemplateBuilder),
            option_generator: Box::new(AiOptionGenerator),
            vee: Box::new(BasicVee),
            risk_evaluator: Box::new(DefaultRiskEvaluator),
            execution_provider: Box::new(LocalExecutionProvider),
        };

        let intent = match orchestrator.intent_resolver.resolve(&nli).await {
            Ok(i) => i,
            Err(e) => {
                eprintln!("❌ Failed to resolve intent: {}", e);
                return;
            }
        };
        match orchestrator.run_pipeline(&nli, intent, "localhost", None, None, None).await {
            Ok(res) => {
                if res.success {
                    println!("✅ Pipeline Success!");
                    if !res.stdout.is_empty() { println!("STDOUT: {}", res.stdout); }
                } else {
                    println!("❌ Pipeline Execution Failed (Code: {:?})", res.exit_code);
                    eprintln!("STDERR: {}", res.stderr);
                }
            },
            Err(e) => eprintln!("❌ Pipeline Error: {}", e),
        }
        return;
    }

    // 4. Command Routing (Continued)

    // v2.0 Abstraction Commands
    let dry_run = args.contains(&"--dry-run".to_string());

    // Pkg Manager: vega install <package>
    if input == "install" && args.len() >= 3 {
        let pkg_name = &args[2];
        let ctx = SystemContext::collect(true);
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
        let ctx = SystemContext::collect(true);
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
        
        let use_sre = args.contains(&"--sre".to_string());

        if use_sre {
            println!("🚀 Generating AI-Powered SRE 5-Step Report for Session {}...", sid);
            if let Ok(db) = crate::storage::db::Database::new() {
                if let Ok(lineage) = db.get_decision_lineage(sid) {
                    match crate::reporting::sre_report::SreReport::generate_from_lineage(sid, &lineage).await {
                        Ok(report) => {
                            let md = report.render_markdown();
                            let filename = format!("SRE_REPORT_SESSION_{}.md", sid);
                            let _ = fs::write(&filename, md);
                            println!("✅ SRE Report saved to: {}", filename);
                        },
                        Err(e) => eprintln!("❌ SRE Report Failed: {}", e),
                    }
                }
            }
        } else if use_markdown {
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
        executor::status::show_status(&kb, args.get(2).map(|s| s.as_str()));
        return;
    }

    // Refresh Context
    if input == "refresh" {
        let target_name = args.get(2).cloned();
        
        if let Some(name) = target_name {
            if let Some(mut entry) = kb.get(&name).cloned() {
                println!("🔄 Refreshing context for '{}'...", name);
                match SshConnection::check_connection(&entry.ip, entry.user.as_deref(), entry.port) {
                    Ok(_) => {
                        let os = SshConnection::detect_os(&entry.ip, entry.user.as_deref(), entry.port);
                        println!("   OS Detected: {}", os.as_deref().unwrap_or("Unknown"));
                        entry.os_type = os;
                        entry.last_success = chrono::Local::now().to_rfc3339();
                        kb.add(&name, entry);
                        let _ = kb.save();
                        println!("✅ Knowledge Base Updated.");
                    }
                    Err(e) => println!("❌ Host Unreachable: {}", e.1),
                }
            } else {
                println!("❌ Target '{}' not found in Knowledge Base.", name);
            }
        } else {
            // Global Refresh: Discover and register all hosts
            println!("🔄 Initiating Global Refresh...");
            let ctx = SystemContext::collect(true);
            let mut registered_count = 0;

            for node in ctx.remotes {
                if node.r#type == crate::context::RemoteType::Host {
                    println!("📡 Verifying: {} ({})", node.name, node.real_name);
                    if SshConnection::check_connection(&node.real_name, None, None).is_ok() {
                        let os = SshConnection::detect_os(&node.real_name, None, None);
                        kb.add(
                            &node.name,
                            crate::knowledge::KnowledgeEntry {
                                ip: node.real_name.clone(),
                                user: None,
                                protocol: "ssh".to_string(),
                                port: Some(22),
                                os_type: os,
                                kernel: None,
                                cpu_load: None,
                                tags: Vec::new(),
                                password: None,
                                last_success: chrono::Local::now().to_rfc3339(),
                            },
                        );
                        registered_count += 1;
                    }
                }
            }
            if registered_count > 0 {
                let _ = kb.save();
                println!("✅ Global Refresh Complete. Registered {} nodes.", registered_count);
            } else {
                println!("⚠️  Global Refresh Complete. No new hosts registered.");
            }
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

            if SshConnection::check_connection(&entry.ip, entry.user.as_deref(), entry.port).is_ok() {
                println!("OK ✅");
                SshConnection::connect(&entry.ip, entry.user.as_deref(), entry.port);
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
                if SshConnection::check_connection(ip, None, None).is_ok() {
                    println!("OK ✅");
                    println!("💾 Persistence: Updating State DB for '{}'...", target_name);

                    let os_detected = SshConnection::detect_os(ip, None, None);
                    kb.add(
                        target_name,
                        KnowledgeEntry {
                            ip: ip.to_string(),
                            user: None,
                            protocol: "ssh".to_string(),
                            port: Some(22),
                            os_type: os_detected,
                            kernel: None,
                            cpu_load: None,
                            tags: Vec::new(),
                            password: None,
                            last_success: chrono::Local::now().to_rfc3339(),
                        },
                    );
                    let _ = kb.save();

                    SshConnection::connect(ip, None, None);
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

    let mut _success = true;

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
            println!("Context: {:?}", SystemContext::collect(true).load_avg);
            println!("Executing: sudo apt update && sudo apt upgrade");
        }
        Action::SshConnect(ref target) => {
            println!("🔌 [Hybrid] Detected SSH intent to '{}'", target);
            let status = Command::new("ssh").arg(target).status();

            match status {
                Ok(s) => {
                    if !s.success() {
                        _success = false;
                    }
                }
                Err(e) => {
                    println!("SSH Failed: {}", e);
                    _success = false;
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
                let ctx = SystemContext::collect(false);
                let preferred_engine = config.ai.as_ref().map(|a| a.provider.clone());

                // 🏗️ Build Detailed Inventory Context (Port, OS awareness)
                let mut inventory_context = String::new();
                for (name, entry) in &kb.targets {
                    inventory_context.push_str(&format!(
                        "- {}: {} (Port: {}, OS: {:?})\n",
                        name, entry.ip, entry.port.unwrap_or(22), entry.os_type
                    ));
                }

                let enriched_query = if !inventory_context.is_empty() {
                    format!("DETAILED INVENTORY:\n{}\n\nUSER REQUEST: {}", inventory_context, full_input)
                } else {
                    full_input.to_string()
                };

                // AI Reasoning Engine (Goal & Intent Inference)
                let response_str = match crate::ai::router::SmartRouter::generate_with_fallback(
                    &ctx,
                    &enriched_query,
                    preferred_engine,
                ).await {
                    Ok(res) => res,
                    Err(e) => {
                        eprintln!("❌ AI Routing Failed: {}", e);
                        return;
                    }
                };

                // Parse AI Response (Robust extraction)
                use crate::ai::AiResponse;
                let ai_res = match AiResponse::extract_json(&response_str) {
                    Some(res) => res,
                    None => {
                        eprintln!("⚠️ AI Parsing Failed. Raw Response: {}", response_str);
                        return;
                    }
                };

                // 🧠 [Cognition Engine V2] - Unified Single-Action Pipeline
                let raw_target = &ai_res.target;
                
                // 🧼 [Target Canonicalization] - Strip ports, user info, and ornaments
                // "dogsinatas@192.168.0.150:22" -> "192.168.0.150"
                let cleansed_target = raw_target
                    .split('@').last().unwrap_or(raw_target) // Remove user@
                    .split(':').next().unwrap_or(raw_target) // Remove :port
                    .trim()
                    .to_string();

                // Resolve Real Target: Strict Resolution (No Hallucination, No Fallback)
                let execution_target: Option<crate::executor::orchestrator::ExecutionTarget> = if cleansed_target == "localhost" || cleansed_target == "127.0.0.1" {
                    // 💡 [Heuristic Fallback] If user mentioned "ssh" or "remote" but AI returned "localhost"
                    let user_input_lower = full_input.to_lowercase();
                    let abstract_keywords = vec!["ssh", "remote", "원격", "연결된", "타겟", "서버", "노드", "target", "node", "server"];
                    
                    if abstract_keywords.iter().any(|k| user_input_lower.contains(k)) && kb.targets.len() == 1 {
                        let entry = kb.targets.values().next().unwrap();
                        println!("💡 [Resolver] Heuristic: Mapping abstract request to the only managed host: {}", entry.ip.cyan());
                        Some(crate::executor::orchestrator::ExecutionTarget::RemoteSsh {
                            host: entry.ip.clone(),
                            user: entry.user.clone(),
                            port: entry.port.unwrap_or(22),
                            password: entry.password.clone(),
                        })
                    } else {
                        Some(crate::executor::orchestrator::ExecutionTarget::Local)
                    }
                } else if ["ssh", "remote", "원격", "target", "server"].contains(&cleansed_target.as_str()) {
                    // 💡 [Heuristic Direct] AI returned a keyword as target
                    if let Some(entry) = kb.targets.values().next() {
                        println!("💡 [Resolver] Heuristic: Grounding abstract target '{}' to: {}", cleansed_target, entry.ip.cyan());
                        Some(crate::executor::orchestrator::ExecutionTarget::RemoteSsh {
                            host: entry.ip.clone(),
                            user: entry.user.clone(),
                            port: entry.port.unwrap_or(22),
                            password: entry.password.clone(),
                        })
                    } else {
                        None
                    }
                } else {
                    // 1. Try to find by Name (Alias) or IP in KnowledgeBase
                    if let Some(entry) = kb.targets.get(&cleansed_target) {
                        Some(crate::executor::orchestrator::ExecutionTarget::RemoteSsh {
                            host: entry.ip.clone(),
                            user: entry.user.clone(),
                            port: entry.port.unwrap_or(22),
                            password: entry.password.clone(),
                        })
                    } else if let Some(entry) = kb.targets.values().find(|v| v.ip == cleansed_target) {
                        Some(crate::executor::orchestrator::ExecutionTarget::RemoteSsh {
                            host: entry.ip.clone(),
                            user: entry.user.clone(),
                            port: entry.port.unwrap_or(22),
                            password: entry.password.clone(),
                        })
                    } else {
                        // 2. Direct IP support (if it looks like an IP)
                        if cleansed_target.chars().all(|c| c.is_digit(10) || c == '.') {
                            Some(crate::executor::orchestrator::ExecutionTarget::RemoteSsh {
                                host: cleansed_target.clone(),
                                user: None,
                                port: 22,
                                password: None,
                            })
                        } else {
                            None // Hard resolution failure
                        }
                    }
                };

                let target = match execution_target {
                    Some(t) => t,
                    None => {
                        eprintln!("❌ [Safety] Target resolution failed: '{}' (Raw: '{}'). Action aborted to prevent unintended local impact.", cleansed_target.red().bold(), raw_target);
                        return;
                    }
                };
                
                println!("📡 [Target] Resolved to canonical: {}", target.identifier().cyan());

                // Map action + params to Intent for template building
                let intent = crate::executor::pipeline::Intent {
                    action: ai_res.action.clone(),
                    target: target.identifier().to_string(),
                    params: ai_res.params.clone(),
                    thought: ai_res.thought.clone(),
                };

                // 🧩 [Deterministic Explanation] Use struct-defined template instead of manual match
                let safe_explanation = intent.get_explanation(&target.identifier());
                
                println!("📝 Explanation: {}", safe_explanation.green());
                println!("🎯 Action: {} on {}", ai_res.action.yellow().bold(), target.identifier().cyan());

                if Interactor::confirm("Execute this semantic action?") {
                    println!("⚡ Executing...");
                    
                    // Create Action via Factory
                    let action = match &target {
                        crate::executor::orchestrator::ExecutionTarget::Local => {
                            crate::executor::action::ActionFactory::create_action(
                                &intent,
                                "localhost",
                                None,
                                Some(22),
                                None,
                            )
                        }
                        crate::executor::orchestrator::ExecutionTarget::RemoteSsh { host, user, port, password } => {
                            crate::executor::action::ActionFactory::create_action(
                                &intent,
                                host,
                                user.clone(),
                                Some(*port),
                                password.clone(),
                            )
                        }
                    };

                    match action {
                        Some(action) => {
                            let db = Arc::new(crate::storage::db::Database::new().unwrap());
                            let executor = crate::executor::orchestrator::ActionExecutor::new(db);

                            // Convert Box<dyn Action> to Arc<dyn Action> using .into()
                            let action_arc: Arc<dyn crate::executor::action::Action> = action.into();

                            match executor.run(action_arc, target, false).await {
                                Ok(res) => {
                                    if res.success {
                                        println!("✅ Success: {}", res.stdout);
                                    } else {
                                        eprintln!("❌ Error: {}", res.stderr);
                                    }
                                }
                                Err(e) => eprintln!("❌ Execution Failed: {}", e),
                            }
                        }
                        None => eprintln!("❌ [Factory] Failed to create action from intent."),
                    }
                }
                } else {
                // Short command / Alias processing
                let db = Arc::new(crate::storage::db::Database::new().unwrap());
                let executor = crate::executor::orchestrator::ActionExecutor::new(db);
                
                // Use ShellAction for tracking
                let shell_action = crate::executor::action::ActionFactory::create_shell_action(&full_input);
                let action_arc: Arc<dyn crate::executor::action::Action> = shell_action.into();
                
                match executor.run(action_arc, crate::executor::orchestrator::ExecutionTarget::Local, false).await {
                    Ok(res) => {
                        if res.success {
                            println!("{}", res.stdout);
                        } else {
                            eprintln!("{}", res.stderr);
                        }
                    }
                    Err(e) => eprintln!("❌ Execution Failed: {}", e),
                }
            }
        }
    }
}
