use std::sync::Arc;
use crate::executor::ast::CommandAst;
use crate::executor::ExecuteResult;
use crate::executor::action::Action;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub action: String,
    pub target: String,
    pub params: serde_json::Value,
    #[serde(default)]
    pub thought: String,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

fn default_confidence() -> f32 { 1.0 }

// Legacy Intent enum will be phased out in favor of the structured Intent struct
// but for compatibility during migration, we keep the mapping logic.

impl Intent {
    pub fn get_explanation(&self, _target_host: &str) -> String {
        let params = &self.params;
        match self.action.as_str() {
            "OLLAMA_LIST_INSTALLED" => format!("{} 호스트에 설치된 모든 Ollama 모델 목록을 조회합니다.", self.target),
            "OLLAMA_LIST_RUNNING" => format!("{} 호스트에서 현재 실행 중인 Ollama 모델 목록을 조회합니다.", self.target),
            "OLLAMA_VERSION" => format!("{} 호스트의 Ollama 버전을 확인합니다.", self.target),
            "OLLAMA_START" => format!("원격 호스트 '{}'의 Ollama 서비스를 시작(Start)합니다.", self.target),
            "OLLAMA_STOP" => format!("원격 호스트 '{}'의 Ollama 서비스를 중단(Stop)합니다.", self.target),
            "OLLAMA_PULL" => format!("원격 호스트 '{}'에 모델 '{}'을(를) 다운로드합니다.", self.target, params["model"].as_str().unwrap_or("unknown")),
            "OLLAMA_REMOVE" => format!("{} 호스트에서 모델 '{}'을(를) 삭제합니다.", self.target, params["model"].as_str().unwrap_or("unknown")),
            "INSTALL_APT" => format!("{} 호스트에 패키지 '{}'을(를) 설치합니다.", self.target, params["name"].as_str().unwrap_or("unknown")),
            "SYSTEM_UPDATE" => format!("{} 호스트의 시스템을 업데이트합니다.", self.target),
            "SYSTEM_DIAGNOSTIC" => format!("{} 호스트의 시스템 상태를 진단합니다.", self.target),
            "SYSTEM_SHUTDOWN" => format!("⚠️ {} 호스트를 종료(SHUTDOWN)합니다.", self.target),
            "SSH_CONNECT" => format!("원격 호스트 {}에 연결을 시도합니다.", params["host"].as_str().unwrap_or(&self.target)),
            _ => format!("결정론적 의도('{}')를 파악하여 오케스트레이션을 준비합니다.", self.action),
        }
    }

    pub fn is_unknown(&self) -> bool {
        self.action == "UNKNOWN" || self.action.is_empty()
    }

    pub fn unknown() -> Self {
        Self {
            action: "UNKNOWN".to_string(),
            target: "localhost".to_string(),
            params: serde_json::json!({}),
            thought: "".to_string(),
            confidence: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimLog {
    pub is_safe: bool,
    pub predicted_impact: String,
    pub risk_score: i32,
    pub suggestion: Option<String>,
}

#[async_trait]
pub trait IntentResolver {
    async fn resolve(&self, input: &str) -> anyhow::Result<Intent>;
}

pub trait TemplateBuilder {
    fn build(&self, intent: &Intent) -> anyhow::Result<CommandAst>;
}

#[async_trait]
pub trait OptionGenerator {
    async fn generate_options(&self, ast: &mut CommandAst) -> anyhow::Result<()>;
}

pub trait VirtualExecutionEngine {
    fn simulate(&self, ast: &CommandAst) -> anyhow::Result<SimLog>;
}

pub trait RiskEvaluator {
    fn evaluate(&self, sim_log: &SimLog) -> bool; // returns true if safe to execute
}

#[async_trait]
pub trait ExecutionProvider {
    async fn execute(&self, ast: &CommandAst) -> anyhow::Result<ExecuteResult>;
}

use colored::Colorize;

pub struct PipelineOrchestrator {
    pub intent_resolver: Box<dyn IntentResolver + Send + Sync>,
    pub template_builder: Box<dyn TemplateBuilder + Send + Sync>,
    pub option_generator: Box<dyn OptionGenerator + Send + Sync>,
    pub vee: Box<dyn VirtualExecutionEngine + Send + Sync>,
    pub risk_evaluator: Box<dyn RiskEvaluator + Send + Sync>,
    pub execution_provider: Box<dyn ExecutionProvider + Send + Sync>,
}

impl PipelineOrchestrator {
    pub fn new_default() -> Self {
        Self {
            intent_resolver: Box::new(crate::ai::intent::HybridIntentResolver {
                local: crate::ai::intent::LocalIntentResolver,
                ai: crate::ai::intent::AiIntentResolver,
            }),
            template_builder: Box::new(crate::executor::template::BasicTemplateBuilder),
            option_generator: Box::new(crate::executor::template::BasicTemplateBuilder), 
            vee: Box::new(crate::executor::template::BasicTemplateBuilder),
            risk_evaluator: Box::new(crate::executor::template::BasicTemplateBuilder),
            execution_provider: Box::new(crate::executor::pipeline::LocalExecutionProvider),
        }
    }

    pub async fn run_pipeline(
        &self, 
        input: &str, 
        intent: crate::executor::pipeline::Intent, 
        target_host: &str, 
        user: Option<String>, 
        port: Option<u16>, 
        password: Option<String>
    ) -> anyhow::Result<ExecuteResult> {
        let intent_str = format!("{:?}", intent);
        println!("🎯 [Semantic Intent] Recognized: {}", intent_str.cyan());
        
        // 🛡️ Hard Fail: target_host validation
        if target_host.is_empty() {
             return Err(anyhow::anyhow!("Sovereign Violation: Action target host cannot be empty or default."));
        }

        // 2. Action Creation via Factory
        let action = crate::executor::action::ActionFactory::create_action(&intent, target_host, user.clone(), port, password.clone())
            .ok_or_else(|| anyhow::anyhow!("No semantic action mapping found for intent: {}", intent_str))?;
        
        println!("🏗️  [Action] Resolved to: {}", action.name().yellow().bold());

        // 3. Pre-flight Validation
        println!("🔍 [Validation] Running capability checks for: {}", target_host.cyan());
        let snapshot = if target_host == "localhost" || target_host == "127.0.0.1" {
            crate::system::snapshot::HostSnapshot::collect_local()
        } else {
            crate::system::snapshot::HostSnapshot::collect_remote(
                target_host, 
                user.as_deref(), 
                port, 
                password.as_deref(),
                &action.required_capabilities()
            ).await
                .map_err(|e| anyhow::anyhow!("Remote Capability Discovery Failed: {}", e))?
        };

        action.validate(&snapshot).await.map_err(|e| anyhow::anyhow!("Validation Failed: {}", e))?;
        println!("✅ [Validation] All checks passed.");

        // 4. Execution Plan
        let plan = action.plan().await.map_err(|e| anyhow::anyhow!(e))?;
        println!("📝 [Execution Plan]");
        for step in &plan.steps {
            println!("   - {}", step);
        }
        println!("   Impact: {}", plan.estimated_impact.blue());
        
        // 5. Simulation (Legacy bridge for risk score)
        let risk_score = match plan.danger_level {
            crate::executor::action::DangerLevel::Safe => 0,
            crate::executor::action::DangerLevel::Moderate => 30,
            crate::executor::action::DangerLevel::Dangerous => 70,
            crate::executor::action::DangerLevel::Critical => 100,
        };
        
        // 6. Execution
        println!("⚡ [Execution] Initiating semantic action...");
        let result = action.execute().await.map_err(|e| anyhow::anyhow!(e))?;
        
        // 7. Result Presentation (Semantics)
        let output = action.parse_output(&result);
        self.render_result(&output);

        // 8. Autonomous Remediation Loop (Alpha 39)
        if let crate::executor::action::ActionOutput::ServiceState(state) = &output {
            if !state.state_converged.unwrap_or(true) 
               && state.remediation_available.unwrap_or(false) 
               && state.safe_to_apply.unwrap_or(false) {
                
                if crate::safety::confirm_action(crate::safety::RiskLevel::Warning, "Apply remediation to converge system state?") {
                    println!("🔧 [Remediation] Initiating state reconciliation...");
                    let remediate_action: Arc<dyn Action> = Arc::new(crate::executor::ollama::OllamaRemediate);
                    
                    // Re-dispatch via execution provider (Simplified for Alpha 39)
                    let rem_res = remediate_action.execute().await.map_err(|e| anyhow::anyhow!(e))?;
                    let rem_output = remediate_action.parse_output(&rem_res);
                    self.render_result(&rem_output);
                    
                    println!("✅ [Remediation] Convergence loop completed.");
                }
            }
        }

        // 9. Decision Lineage Persistence
        if let Ok(db) = crate::storage::db::Database::new() {
            let sim_log_str = format!("Semantic Action: {}", action.name());
            let final_cmd = "Semantic Internal".to_string();
            let res_str = format!("Success: {}, ExitCode: {:?}", result.success, result.exit_code);
            let _ = db.log_decision_lineage(input, &intent_str, &final_cmd, &sim_log_str, risk_score, &res_str);
        }
        
        Ok(result)
    }

    pub fn render_result(&self, output: &crate::executor::action::ActionOutput) {
        use crate::executor::action::ActionOutput;
        println!("\n{}", "📊 [Result Presentation]".green().bold());
        match output {
            ActionOutput::ServiceState(state) => {
                let service_name = state.service.clone().unwrap_or("service".to_string()).to_uppercase();
                if state.is_ready() {
                    println!("   {} {} ready", "✅".green(), service_name.cyan().bold());
                } else {
                    println!("   {} {} state: {}", "⚠️".yellow(), service_name.cyan().bold(), state.status.yellow());
                }

                println!("   {} Host:    {} ({})", "📍".blue(), state.host.clone().unwrap_or("unknown".to_string()), state.user.clone().unwrap_or("unknown".to_string()));
                if let Some(v) = &state.version {
                    println!("   {} Version: {}", "🔢".magenta(), v.cyan());
                }
                if let Some(pid) = state.pid {
                    println!("   {} PID:     {}", "🆔".blue(), pid);
                }
                
                let bind = state.bind.clone().unwrap_or("unknown".to_string());
                let scope = state.bind_scope.clone().unwrap_or("local".to_string());
                println!("   {} Bind:    {} ({})", "🔌".yellow(), bind.cyan(), scope.yellow());

                let tunnel = if state.ssh_tunnel_reachable.unwrap_or(false) { "Accessible".green() } else { "Unavailable".red() };
                println!("   {} SSH Tunnel: {}", "🔗".blue(), tunnel);

                let lan = if state.lan_reachable.unwrap_or(false) { "Exposed".green() } else { "Disabled (Loopback)".yellow() };
                println!("   {} LAN Direct:  {}", "🌐".blue(), lan);

                if !state.state_converged.unwrap_or(true) {
                    println!("\n{} {}", "⚠️".yellow().bold(), "State Divergence Detected".yellow().bold());
                    if let Some(desired) = &state.desired_bind {
                        println!("   Desired State: {}", desired.cyan());
                    }
                    if let Some(actual) = &state.actual_bind {
                        println!("   Current State: {}", actual.yellow());
                    }
                    
                    if state.remediation_available.unwrap_or(false) && state.safe_to_apply.unwrap_or(false) {
                        println!("\n{} {}", "🔧".green().bold(), "Remediation Available".green().bold());
                        if state.requires_restart.unwrap_or(false) {
                            println!("   {} Ollama requires a restart to apply the new bind configuration.", "💡".blue());
                            println!("   {} To fix, run: {} or follow the prompts in future versions.", "👉".green(), "sudo systemctl edit ollama".yellow());
                        }
                    }
                }
            },
            ActionOutput::OllamaVersion(v) => {
                println!("   ✨ Ollama Version: {}", v.cyan().bold());
            },
            ActionOutput::OllamaModelList(models) => {
                println!("   📦 Installed Models ({}):", models.len());
                if models.is_empty() {
                    println!("      (No models found)");
                } else {
                    for m in models {
                        println!("      - {}", m.cyan());
                    }
                }
            },
            ActionOutput::GenericSuccess(msg) => {
                println!("   ✅ Success: {}", msg);
            },
            ActionOutput::Raw(raw) => {
                if !raw.is_empty() {
                    println!("{}", raw.cyan());
                }
            }
        }
        println!("");
    }
}

pub struct LocalExecutionProvider;

#[async_trait]
impl ExecutionProvider for LocalExecutionProvider {
    async fn execute(&self, ast: &CommandAst) -> anyhow::Result<ExecuteResult> {
        let cmd_str = ast.to_shell_command();
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd_str)
            .output()?;
        
        Ok(ExecuteResult {
            success: output.status.success(),
            status: if output.status.success() { crate::executor::ExecutionStatus::Success } else { crate::executor::ExecutionStatus::FatalFailure },
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
            error: if output.status.success() { None } else { 
                Some(crate::executor::OrchestrationError::Execution(String::from_utf8_lossy(&output.stderr).to_string()))
            },
            insight: None,
        })
    }
}

pub struct RemoteExecutionProvider {
    pub ip: String,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub password: Option<String>,
}

#[async_trait]
impl ExecutionProvider for RemoteExecutionProvider {
    async fn execute(&self, ast: &CommandAst) -> anyhow::Result<ExecuteResult> {
        let cmd_str = ast.to_shell_command();
        match crate::connection::ssh::SshConnection::execute_remote_async(&self.ip, self.user.as_deref(), self.port, self.password.as_deref(), &cmd_str).await {
            Ok(stdout) => {
                Ok(ExecuteResult {
                    success: true,
                    status: crate::executor::ExecutionStatus::Success,
                    stdout,
                    stderr: "".to_string(),
                    exit_code: Some(0),
                    error: None, insight: None,
                })
            }
            Err(stderr) => {
                Ok(ExecuteResult {
                    success: false,
                    status: crate::executor::ExecutionStatus::FatalFailure,
                    stdout: "".to_string(),
                    stderr: stderr.clone(),
                    exit_code: Some(255), // SSH general failure code
                    error: Some(crate::executor::OrchestrationError::Transport(stderr)),
                    insight: None,
                })
            }
        }
    }
}
