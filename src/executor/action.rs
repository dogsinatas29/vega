use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::executor::ExecuteResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    /// Minimum requirements for this action to even be considered
    fn required_capabilities(&self) -> Vec<CapabilityRequirement> {
        vec![] // Default: no specific requirements
    }

    /// Whether to skip system context snapshot for this action
    fn skip_snapshot(&self) -> bool {
        false
    }

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
        use crate::executor::pipeline::Intent;
        
        match intent {
            Intent::OllamaListInstalled {} => {
                Some(Box::new(crate::executor::ollama::OllamaListInstalledModels))
            },
            Intent::OllamaListRunning {} => {
                Some(Box::new(crate::executor::ollama::OllamaListRunningModels))
            },
            Intent::OllamaVersion {} => {
                Some(Box::new(crate::executor::ollama::OllamaVersion))
            },
            Intent::OllamaPull { model } => {
                Some(Box::new(crate::executor::ollama::OllamaModelPull {
                    id: format!("pull-{}", model),
                    model_name: model.clone(),
                }))
            },
            Intent::OllamaRemove { model, force } => {
                Some(Box::new(crate::executor::ollama::OllamaModelRemove {
                    id: format!("rm-{}", model),
                    model_name: model.clone(),
                    force: *force,
                }))
            },
            Intent::InstallApt { name } => {
                Some(Box::new(crate::executor::pkg::AptInstall {
                    package_name: name.clone(),
                }))
            },
            Intent::InstallDocker { name } => {
                Some(Box::new(crate::executor::pkg::DockerRun {
                    image_name: name.clone(),
                }))
            },
            Intent::SystemUpdate {} => None,
            Intent::SystemDiagnostic {} => {
                Some(Box::new(crate::executor::system::SystemDiagnostic::new(_target_host.to_string())))
            },
            Intent::SshConnect { host } => {
                Some(Box::new(crate::executor::action::ShellAction {
                    command: format!("ssh {}", host),
                }))
            },
            Intent::BackupData { .. } => None,
            Intent::Unknown => None,
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
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
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
