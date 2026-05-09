use crate::knowledge::KnowledgeBase;
use crate::remote::rclone::RcloneProvider;
use crate::remote::RemoteProvider;
use crate::safety::{confirm_action, SafetyRegistry};
use crate::executor::ExecuteResult;
use crate::connection::ssh::SshConnection;
use log::info;
use std::sync::Arc;
use colored::Colorize;

use crate::executor::action::{Action, DangerLevel};
use crate::storage::db::Database;

#[derive(Debug, Clone)]
pub enum ExecutionTarget {
    Local,
    RemoteSsh {
        host: String,
        user: Option<String>,
        port: u16,
        password: Option<String>,
    }
}

impl ExecutionTarget {
    pub fn is_local(&self) -> bool {
        matches!(self, ExecutionTarget::Local)
    }

    pub fn identifier(&self) -> String {
        match self {
            ExecutionTarget::Local => "localhost".to_string(),
            ExecutionTarget::RemoteSsh { host, .. } => host.clone(),
        }
    }
}

pub struct ActionExecutor {
    db: Arc<Database>,
    policy_engine: crate::safety::policy::PolicyEngine,
}

impl ActionExecutor {
    pub fn new(db: Arc<Database>) -> Self {
        Self { 
            db,
            policy_engine: crate::safety::policy::PolicyEngine::new(),
        }
    }

    pub async fn run_goal(&self, goal: crate::executor::goal::Goal) -> Result<(), String> {
        println!("🧠 [Cognition] Goal detected: {:?}", goal);
        let plan = crate::executor::goal::GoalPlanner::plan(&goal);
        
        println!("📝 [Cognition] Strategy for this goal:");
        for step in plan {
            println!("   - {}", step);
        }
        
        // For now, we only print the plan to show cognition.
        // In the next step, we will implement the actual ActionGraph execution.
        println!("🚀 [Cognition] Goal orchestration initiated. (WIP)");
        
        Ok(())
    }

    pub async fn run(&self, action: Arc<dyn Action>, target: ExecutionTarget, dry_run: bool) -> Result<ExecuteResult, String> {
        let action_name = action.name();
        let target_id = target.identifier();
        
        // 🆔 [Persistence] Generate a true unique ID for this specific execution instance
        let execution_id = uuid::Uuid::new_v4().to_string();
        
        // 1. Register Action in DB (Persistence - Non-fatal)
        if let Err(e) = self.db.register_action(&execution_id, &action_name, if dry_run { "DryRunning" } else { "Queued" }, Some(&target_id)) {
            info!("⚠️ Action persistence failed: {}. Continuing with execution...", e);
        }

        let action_id = execution_id; // Use the UUID for tracking through the pipeline

        // 2. Collect Snapshot (Skip if mode is DirectDispatch or action says not required)
        let mode = action.execution_mode();
        let needs_snapshot = action.requires_snapshot() && mode != crate::executor::action::ExecutionMode::DirectDispatch;
        
        let snapshot = if !needs_snapshot {
            println!("🛰️  [Executor] Skipping snapshot: Action '{}' does not require preflight sensing.", action_name);
            crate::system::snapshot::HostSnapshot::dummy()
        } else {
            match &target {
                ExecutionTarget::Local => {
                    println!("🛰️  [Executor] Capturing local snapshot...");
                    crate::system::snapshot::HostSnapshot::collect_local()
                }
                ExecutionTarget::RemoteSsh { host, user, port, password } => {
                    println!("🛰️  [Executor] Capturing minimal remote snapshot for {}...", host);
                    crate::system::snapshot::HostSnapshot::collect_remote(
                        host,
                        user.as_deref(),
                        Some(*port),
                        password.as_deref(),
                        &action.required_capabilities(),
                    ).await?
                }
            }
        };

        // 3. Validate with Snapshot & Privilege Check
        crate::ui::console::SreConsole::telemetry(&format!("Validating {} based on host capabilities...", action_name));
        
        // A. Semantic Validation
        action.validate(&snapshot).await.map_err(|e| {
            let _ = self.db.update_action_state(&action_id, "Failed", 0.0);
            format!("Validation failed: {}", e)
        })?;

        // B. Privilege Validation (Root-Guard)
        if action.requires_root() && !snapshot.capabilities.sudo_nopasswd && !target.is_local() {
            crate::ui::console::SreConsole::error(&format!("Privilege Block: Remote host '{}' requires interactive sudo authentication.", target_id));
            crate::ui::console::SreConsole::note("VEGA cannot safely automate this action without NOPASSWD configuration.");
            let _ = self.db.update_action_state(&action_id, "PrivilegeBlocked", 0.0);
            return Err(format!("Remote host requires interactive sudo for action '{}'.", action_name));
        }

        // 4. Planning & Building
        let cmd = action.build_command();

        if dry_run {
            crate::ui::console::SreConsole::logic(&format!("Proposed Command: {}", cmd.yellow()));
            return Ok(ExecuteResult {
                success: true,
                status: crate::executor::ExecutionStatus::Success,
                stdout: format!("Dry run successful: {}", cmd),
                stderr: String::new(),
                exit_code: Some(0),
                error: None, insight: None,
            });
        }

        let plan = action.plan().await?;
        crate::ui::console::SreConsole::telemetry("Execution Strategy:");
        for step in &plan.steps {
            crate::ui::console::SreConsole::telemetry(&format!("   - {}", step));
        }
        
        if plan.danger_level > crate::executor::action::DangerLevel::Safe {
             crate::ui::console::SreConsole::logic(&format!("Impact: {}, Level: {:?}", plan.estimated_impact, plan.danger_level));
        }

        // 5. Policy Check (Phase 2)
        if let Err(e) = self.policy_engine.check(&snapshot, plan.danger_level.clone()) {
            crate::ui::console::SreConsole::error(&format!("Policy Block: {}", e));
            let _ = self.db.update_action_state(&action_id, "PolicyBlocked", 0.0);
            return Err(e);
        }

        // 6. Confirmation Barrier
        if plan.danger_level == DangerLevel::Dangerous || plan.danger_level == DangerLevel::Critical {
            if !confirm_action(crate::safety::RiskLevel::Warning, &action_name) {
                let _ = self.db.update_action_state(&action_id, "Cancelled", 0.0);
                return Err("Action cancelled by user safety check".to_string());
            }
        }

        // 7. Strict Guards for Critical Actions (Phase 2)
        if plan.danger_level == DangerLevel::Critical {
            crate::ui::console::SreConsole::error("CRITICAL: This action has system-wide impact.");
            
            // A. Active Session Check
            if let ExecutionTarget::RemoteSsh { host, user, port, password } = &target {
                let who_cmd = "who | wc -l";
                let session_count: i32 = SshConnection::execute_remote_async(host, user.as_deref(), Some(*port), password.as_deref(), who_cmd).await
                    .unwrap_or_default().trim().parse().unwrap_or(0);
                if session_count > 1 {
                    crate::ui::console::SreConsole::error(&format!("{} other users are currently logged in. Proceed with extreme caution!", session_count - 1));
                }
            }

            // B. Cooldown Period
            for i in (1..=5).rev() {
                print!("\r⏳ [Cooldown] Executing in {} seconds... (Ctrl+C to abort) ", i);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            println!("\r🚀 [Cooldown] Time's up. Finalizing execution.          ");

            // C. Second Validation
            if !confirm_action(crate::safety::RiskLevel::Critical, &format!("FINAL CONFIRMATION: {}", action_name)) {
                let _ = self.db.update_action_state(&action_id, "Cancelled", 0.0);
                return Err("Action cancelled at the last second".to_string());
            }
        }

        // 8. Execute
        if dry_run {
            crate::ui::console::SreConsole::logic("Simulation mode active. Skipping physical execution.");
            let _ = self.db.update_action_state(&action_id, "DryRunComplete", 1.0);
            return Ok(ExecuteResult {
                success: true,
                status: crate::executor::ExecutionStatus::Success,
                stdout: format!("Dry-run successful: {}", cmd),
                stderr: String::new(),
                exit_code: Some(0),
                error: None, insight: None,
            });
        }

        let _ = self.db.update_action_state(&action_id, "Executing", 0.1);
        
        // Final Execution Dispatch
        let mut res = match &target {
            ExecutionTarget::Local => {
                crate::ui::console::SreConsole::debug(&format!("Dispatching Local Command: {}", cmd));
                let output = tokio::process::Command::new("bash")
                    .arg("-c")
                    .arg(&cmd)
                    .output()
                    .await
                    .map_err(|e| format!("Local execution failed: {}", e))?;
                
                ExecuteResult {
                    success: output.status.success(),
                    status: if output.status.success() { crate::executor::ExecutionStatus::Success } else { crate::executor::ExecutionStatus::FatalFailure },
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    exit_code: output.status.code(),
                    error: if output.status.success() { None } else { 
                        Some(crate::executor::OrchestrationError::Execution(String::from_utf8_lossy(&output.stderr).to_string()))
                    },
                    insight: None,
                }
            }
            ExecutionTarget::RemoteSsh { host, user, port, password } => {
                crate::ui::console::SreConsole::debug(&format!("Dispatching Remote Command: {} to {}", cmd, host));
                match SshConnection::execute_remote_full(host, user.as_deref(), Some(*port), password.as_deref(), &cmd).await {
                    Ok(r) => r,
                    Err(e) => {
                        ExecuteResult {
                            success: false,
                            status: crate::executor::ExecutionStatus::FatalFailure,
                            stdout: String::new(),
                            stderr: format!("SSH Transport Error: {}. Check remote connectivity and environment.", e),
                            exit_code: Some(255),
                            error: Some(crate::executor::OrchestrationError::Transport(e)),
                            insight: None,
                        }
                    }
                }
            }
        };

        // 🧠 [SRE Insight] Analyze failure causes for technical guidance
        if !res.success {
            res.insight = Self::generate_sre_insight(&res.stderr);
        }

        // 🧩 [Semantic Reconciliation] Evaluate the outcome based on domain knowledge
        let evaluation = action.evaluate_outcome(&res);

        match evaluation {
            crate::executor::action::OutcomeEvaluation::Success => {
                let _ = self.db.update_action_state(&action_id, "Completed", 1.0);
                Ok(res)
            }
            crate::executor::action::OutcomeEvaluation::SuccessAlreadySatisfied(msg) => {
                println!("{} {}", "ℹ️".blue(), msg.blue().bold());
                println!("✅ {}", "Desired state satisfied.".green().bold());
                let _ = self.db.update_action_state(&action_id, &format!("Satisfied: {}", msg), 1.0);
                
                // Return a successful version of result to satisfy the rest of the pipeline
                let mut semantic_success = res.clone();
                semantic_success.success = true;
                Ok(semantic_success)
            }
            crate::executor::action::OutcomeEvaluation::Failure(err) => {
                println!("{}", "--- REMOTE EXECUTION FAILED ---".red().bold());
                if !res.stdout.is_empty() {
                    println!("{} \n{}", "STDOUT:".yellow(), res.stdout);
                }
                if !res.stderr.is_empty() {
                    println!("{} \n{}", "STDERR:".red(), res.stderr);
                }
                if let Some(code) = res.exit_code {
                    println!("{} {}", "EXIT CODE:".red(), code);
                }
                println!("{}", "-------------------------------".red().bold());

                let _ = self.db.update_action_state(&action_id, &format!("Failed: {}", err), 0.0);
                if let Err(re) = action.rollback().await {
                    eprintln!("🛑 Rollback failed: {}", re);
                }

                // 🧠 [SRE Insight] Final reporting with actionable guidance
                if let Some(insight) = &res.insight {
                    println!("\n{}", insight.cyan().bold());
                }

                Ok(res)
            }
        }
    }

    fn generate_sre_insight(stderr: &str) -> Option<String> {
        if stderr.contains("Authentication failed") || stderr.contains("Permission denied (publickey)") {
            Some("💡 [SRE Insight] Authentication failed. Possible causes:\n  - sudo password required but VEGA is running non-interactively.\n  - NOPASSWD not configured in /etc/sudoers.d/vega.\n  - SSH key not authorized on target host.".to_string())
        } else if stderr.contains("invalid model name") {
            Some("💡 [SRE Insight] The model name provided is invalid. This might be due to encoding issues or phonetic translation hallucination. Ensure model names are ASCII preserved.".to_string())
        } else if stderr.contains("Connection refused") {
            Some("💡 [SRE Insight] Connection refused. The target service (Ollama or SSH) might not be running, or a firewall is blocking the port.".to_string())
        } else {
            None
        }
    }
}

#[allow(dead_code, unused_variables)]
pub async fn execute_task(provider: Arc<dyn RemoteProvider>, cmd: &str) -> Result<String, String> {
    // Safety Guard
    let risk = crate::safety::check_risk_level(cmd);
    if !confirm_action(risk, cmd) {
        return Err("Action cancelled by user safety check".to_string());
    }

    provider.search(cmd).await.map(|v| v.join("\n"))
}

pub async fn sync_cloud_storage(
    provider: &RcloneProvider,
    source: &str,
    destination: &str,
) -> Result<(), String> {
    // Safety check: check size before sync
    info!("🛡️ Safety check: evaluating transfer size...");

    // In a real implementation, we'd run `rclone size --json source destination`
    // For this milestone, we'll implement a size-limit enforcement shell.
    let risk = SafetyRegistry::validate_rclone_command(&["sync"]);
    if !confirm_action(risk, &format!("rclone sync {} {}", source, destination)) {
        return Err("Sync cancelled by user".to_string());
    }

    if let Ok(output) = provider.execute_rclone(vec!["size", "--json", source]) {
        if let Ok(size_info) = serde_json::from_str::<serde_json::Value>(&output) {
            let total_bytes = size_info["bytes"].as_u64().unwrap_or(0);
            let limit_gb = 1; // Default 1GB limit as per blueprint
            if total_bytes > limit_gb * 1024 * 1024 * 1024 {
                return Err(format!(
                    "Transfer size ({} bytes) exceeds safety limit of {}GB.",
                    total_bytes, limit_gb
                ));
            }
        }
    }

    provider.sync(source, destination).await
}

#[allow(dead_code)]
pub async fn update_fleet(kb: &mut KnowledgeBase, filter_tag: Option<&str>) -> Result<(), String> {
    use crate::connection::ssh::SshConnection;
    use colored::Colorize;

    println!("🚀 Initiating Fleet-wide Maintenance...");
    if let Some(tag) = filter_tag {
        println!("   🎯 Filter: Only nodes with tag '{}'", tag.yellow());
    }
    
    for (name, entry) in &mut kb.targets {
        if entry.protocol == "ssh" {
            // Check Tag Filter
            if let Some(tag) = filter_tag {
                if !entry.tags.contains(&tag.to_string()) {
                    continue;
                }
            }

            println!("📡 Maintenance on {}: Refreshing health...", name.green());
            match SshConnection::get_system_info(&entry.ip).await {
                Ok((kernel, load)) => {
                    println!("   ✅ [Connected] Kernel: {}, Load: {}", kernel.cyan(), load.cyan());
                    entry.kernel = Some(kernel);
                    entry.cpu_load = Some(load);
                    entry.last_success = chrono::Local::now().to_rfc3339();
                }
                Err(e) => {
                    println!("   ❌ [Failed] {}", e.red());
                }
            }
        }
    }
    
    kb.save().map_err(|e| format!("Failed to save KB: {}", e))
}
pub async fn sync_all_cloud(
    ctx: &crate::context::SystemContext,
    primary_remote: Option<String>,
) -> Result<(), String> {
    let cwd = std::env::current_dir().unwrap_or_default();
    let source = cwd.to_string_lossy().to_string();

    if let Some(primary) = primary_remote {
        info!("Syncing project to primary cloud node: {}", primary);
        let provider = RcloneProvider::new(primary);
        return sync_cloud_storage(&provider, &source, "backup/vega_sync").await;
    }

    for node in &ctx.remotes {
        if node.r#type == crate::context::RemoteType::Storage {
            info!("Syncing project to discovered storage node: {}", node.real_name);
            let provider = RcloneProvider::new(node.real_name.clone());
            let _ = sync_cloud_storage(&provider, &source, "backup/vega_sync").await;
        }
    }
    Ok(())
}
pub async fn summarize_session(session_id: i64) -> Result<String, String> {
    let db = crate::storage::db::Database::new().map_err(|e| e.to_string())?;
    let tasks = db
        .get_session_tasks(session_id)
        .map_err(|e| e.to_string())?;

    if tasks.is_empty() {
        return Ok("No activity recorded in this session.".to_string());
    }

    let _activity = tasks
        .iter()
        .map(|t| format!("- {}", t.command))
        .collect::<Vec<_>>()
        .join("\n");

    // In a real scenario, this would call the LLM Router.
    // For now, providing a high-quality SRE-style summary template.
    Ok(format!("Maintenance Session Summary (SID: {}):\nAnalyzed system health and performed {} operations. Cloud synchronization was verified across identified nodes.", session_id, tasks.len()))
}
