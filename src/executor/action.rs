use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::executor::ExecuteResult;

pub const SENTINEL_READY: &str = "__VEGA_READY__";
pub const SENTINEL_FAIL: &str = "__VEGA_FAIL__";
pub const SENTINEL_TIMEOUT: &str = "__VEGA_TIMEOUT__";
pub const SENTINEL_JSON_PREFIX: &str = "__VEGA_JSON__";
pub const SENTINEL_PROTOCOL_BEGIN: &str = "__VEGA_PROTOCOL_BEGIN__";
pub const SENTINEL_PROTOCOL_END: &str = "__VEGA_PROTOCOL_END__";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VegaEvent {
    pub protocol: Option<String>,
    pub r#type: String,
    pub status: String,
    pub service: Option<String>,
    pub severity: Option<String>,
    pub message: Option<String>,
    pub host: Option<String>,
    pub user: Option<String>,
    pub pid: Option<u32>,
    pub version: Option<String>,
    pub bind: Option<String>,
    pub local_ready: Option<bool>,
    pub externally_reachable: Option<bool>,
    pub progress: Option<f32>,
    pub gpu_info: Option<serde_json::Value>,
    pub ack: Option<bool>,
    pub transaction: Option<String>,
    pub bind_scope: Option<String>,
    pub ssh_tunnel_reachable: Option<bool>,
    pub lan_reachable: Option<bool>,
    pub desired_bind: Option<String>,
    pub actual_bind: Option<String>,
    pub state_converged: Option<bool>,
    pub remediation_available: Option<bool>,
    pub requires_restart: Option<bool>,
    pub safe_to_apply: Option<bool>,
    // Session & Streaming (Alpha 35)
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub correlation_id: Option<String>,
    pub stream_source: Option<String>, // stdout, stderr, telemetry
    pub chunk: Option<String>,
    pub error_category: Option<String>, // AUTH, NETWORK, TIMEOUT, RESOURCE, etc.
    pub recoverable: Option<bool>,
    pub retry_hint: Option<String>,
}

impl VegaEvent {
    pub fn parse_from_stdout(stdout: &str) -> Vec<Self> {
        let mut events = Vec::new();
        let mut in_protocol = false;
        let mut current_frame = String::new();
        
        for line in stdout.lines() {
            let trimmed = line.trim();
            
            if trimmed == SENTINEL_PROTOCOL_BEGIN {
                in_protocol = true;
                current_frame.clear();
                continue;
            }
            
            if trimmed == SENTINEL_PROTOCOL_END {
                in_protocol = false;
                // Try parsing the accumulated frame content
                if !current_frame.is_empty() {
                    // Attempt to parse line by line in case multiple JSONs are inside one frame
                    for frame_line in current_frame.lines() {
                        let f_trimmed = frame_line.trim();
                        if f_trimmed.is_empty() { continue; }
                        
                        let clean_json = f_trimmed.strip_prefix(SENTINEL_JSON_PREFIX).unwrap_or(f_trimmed);
                        if let Ok(event) = serde_json::from_str::<Self>(clean_json) {
                            events.push(event);
                        }
                    }
                }
                current_frame.clear();
                continue;
            }
            
            if in_protocol {
                current_frame.push_str(line);
                current_frame.push('\n');
            } else if trimmed.starts_with(SENTINEL_JSON_PREFIX) {
                // Compatibility mode for direct JSON prefix without framing
                if let Some(json_str) = trimmed.strip_prefix(SENTINEL_JSON_PREFIX) {
                    if let Ok(event) = serde_json::from_str::<Self>(json_str) {
                        events.push(event);
                    }
                }
            }
        }
        events
    }

    pub fn is_ready(&self) -> bool {
        self.status == "READY" || self.status == "SUCCESS"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DangerLevel {
    Safe,      // No destructive impact (e.g., list, check)
    Moderate,  // Small changes (e.g., install new package)
    Dangerous, // Significant changes (e.g., remove package, delete model)
    Critical,  // System-wide impact (e.g., reboot, shutdown, rm -rf /)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallState {
    NotInstalled,
    Queued,
    Downloading,
    Installing,
    Verifying,
    Installed,
    Failed(String),
    RollingBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionOutput {
    OllamaVersion(String),
    OllamaModelList(Vec<String>),
    GenericSuccess(String),
    Raw(String),
    ServiceState(VegaEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub steps: Vec<String>,
    pub estimated_impact: String,
    pub danger_level: DangerLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityRequirement {
    Cpu,
    Ram,
    Disk,
    Gpu,
    Ollama,
    Docker,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    FullSnapshot,    // Full system sensing before execution
    MinimalSnapshot, // Basic reachability check only
    DirectDispatch,  // Skip all sensing, fire and forget
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutcomeEvaluation {
    Success,
    SuccessAlreadySatisfied(String), // e.g., "Model already absent"
    Failure(String),
}

#[async_trait]
pub trait Action: Send + Sync {
    /// Unique ID for the action instance
    fn id(&self) -> String;
    
    /// Human-readable name
    fn name(&self) -> String;

    /// Risk assessment
    fn danger_level(&self) -> DangerLevel;

    /// Execution strategy
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::FullSnapshot
    }

    /// Minimum requirements for this action to even be considered
    fn required_capabilities(&self) -> Vec<CapabilityRequirement> {
        vec![] // Default: no specific requirements
    }

    /// Privilege Requirements: Does this action need sudo/root?
    fn requires_root(&self) -> bool { false }

    /// Privilege Requirements: Does this action strictly require NOPASSWD?
    fn requires_nopasswd(&self) -> bool { false }

    /// Snapshot Requirements: Does this action need system sensing before run?
    fn requires_snapshot(&self) -> bool { true }

    /// 🧩 [Semantic Reconciliation] Evaluate if the desired state is satisfied
    /// regardless of raw exit codes. (e.g., 'already removed' is a success).
    fn evaluate_outcome(&self, result: &ExecuteResult) -> OutcomeEvaluation {
        if result.success {
            OutcomeEvaluation::Success
        } else {
            OutcomeEvaluation::Failure(result.stderr.clone())
        }
    }

    /// Validate if the action can be performed based on host capabilities
    async fn validate(&self, snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String>;

    /// Generate an execution plan before actual run
    async fn plan(&self) -> Result<ExecutionPlan, String>;

    /// Build the shell command for this action
    fn build_command(&self) -> String;

    /// Execute the action (Default implementation uses build_command via executor logic)
    async fn execute(&self) -> Result<ExecuteResult, String>;

    /// Parse raw execute result into structured ActionOutput
    fn parse_output(&self, result: &ExecuteResult) -> ActionOutput {
        ActionOutput::Raw(result.stdout.clone())
    }

    /// Rollback changes if possible
    async fn rollback(&self) -> Result<(), String> {
        Ok(())
    }
}

pub struct ActionFactory;

impl ActionFactory {
    pub fn create_action(intent: &crate::executor::pipeline::Intent, _target_host: &str, _user: Option<String>, _port: Option<u16>, _password: Option<String>) -> Option<Box<dyn Action>> {
        let params = &intent.params;
        let target = if intent.target == "localhost" { _target_host } else { &intent.target };

        match intent.action.as_str() {
            "OLLAMA_LIST_INSTALLED" => {
                Some(Box::new(crate::executor::ollama::OllamaListInstalledModels))
            },
            "OLLAMA_LIST_RUNNING" => {
                Some(Box::new(crate::executor::ollama::OllamaListRunningModels))
            },
            "OLLAMA_VERSION" => {
                Some(Box::new(crate::executor::ollama::OllamaVersion))
            },
            "OLLAMA_PULL" => {
                let model = params["model"].as_str()?.to_string();
                Some(Box::new(crate::executor::ollama::OllamaModelPull {
                    id: format!("pull-{}", model),
                    model_name: model,
                }))
            },
            "OLLAMA_REMOVE" => {
                let model = params["model"].as_str()?.to_string();
                let force = params["force"].as_bool().unwrap_or(false);
                Some(Box::new(crate::executor::ollama::OllamaModelRemove {
                    id: format!("rm-{}", model),
                    model_name: model,
                    force,
                }))
            },
            "OLLAMA_STOP" => {
                Some(Box::new(crate::executor::ollama::OllamaStop))
            },
            "OLLAMA_START" => {
                Some(Box::new(crate::executor::ollama::OllamaStart))
            },
            "INSTALL_APT" => {
                let name = params["name"].as_str()?.to_string();
                Some(Box::new(crate::executor::pkg::AptInstall {
                    package_name: name,
                }))
            },
            "INSTALL_DOCKER" => {
                let name = params["name"].as_str()?.to_string();
                Some(Box::new(crate::executor::pkg::DockerRun {
                    image_name: name,
                }))
            },
            "SYSTEM_UPDATE" => None,
            "SYSTEM_DIAGNOSTIC" => {
                Some(Box::new(crate::executor::system::SystemDiagnostic::new(target.to_string())))
            },
            "SYSTEM_SHUTDOWN" => {
                Some(Box::new(crate::executor::system::SystemShutdown::new(target.to_string())))
            },
            "SSH_LIST_TARGETS" => {
                Some(Box::new(crate::executor::discovery::SshListTargetsAction))
            },
            "INFRA_LIST_REMOTES" => {
                Some(Box::new(crate::executor::discovery::InfraListRemotesAction))
            },
            "SYSTEM_LIST_PROCESSES" => None, // TODO
            "SSH_CONNECT" => {
                let host = params["host"].as_str().unwrap_or(target);
                Some(Box::new(crate::executor::action::ShellAction {
                    command: format!("ssh {}", host),
                }))
            },
            _ => None,
        }
    }
    pub fn create_shell_action(command: &str) -> Box<dyn Action> {
        Box::new(ShellAction {
            command: command.to_string(),
        })
    }
}

pub struct ShellAction {
    pub command: String,
}

#[async_trait]
impl Action for ShellAction {
    fn id(&self) -> String { format!("shell-{}", self.command.len()) }
    fn name(&self) -> String { "Shell Fallback".to_string() }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Moderate }
    fn required_capabilities(&self) -> Vec<CapabilityRequirement> { vec![] }
    fn execution_mode(&self) -> ExecutionMode { ExecutionMode::DirectDispatch }
    
    // Privilege Requirements
    fn requires_root(&self) -> bool { false }
    fn requires_nopasswd(&self) -> bool { false }

    async fn validate(&self, _snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> {
        Ok(())
    }

    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec![format!("sh -c '{}'", self.command)],
            estimated_impact: "Direct shell execution".to_string(),
            danger_level: DangerLevel::Moderate,
        })
    }

    fn build_command(&self) -> String {
        self.command.clone()
    }

    async fn execute(&self) -> Result<ExecuteResult, String> {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .output()
            .map_err(|e| e.to_string())?;
        
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

    async fn rollback(&self) -> Result<(), String> {
        // Generic shell commands usually can't be automatically rolled back
        Err("Automatic rollback not supported for generic shell commands".to_string())
    }

    fn parse_output(&self, result: &ExecuteResult) -> ActionOutput {
        ActionOutput::Raw(result.stdout.clone())
    }
}
