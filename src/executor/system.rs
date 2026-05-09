use async_trait::async_trait;
use crate::executor::action::{Action, DangerLevel, ActionOutput, CapabilityRequirement};
use crate::executor::ExecuteResult;
use crate::system::snapshot::HostSnapshot;
use crate::executor::action::ExecutionPlan;
use uuid::Uuid;

pub struct SystemDiagnostic {
    pub id: String,
    pub target_host: String,
}

impl SystemDiagnostic {
    pub fn new(target_host: String) -> Self {
        Self {
            id: format!("diag-{}", Uuid::new_v4().to_string()[..8].to_string()),
            target_host,
        }
    }
}

#[async_trait]
impl Action for SystemDiagnostic {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> String {
        "System Diagnostic".to_string()
    }

    fn danger_level(&self) -> DangerLevel {
        DangerLevel::Safe
    }

    fn required_capabilities(&self) -> Vec<CapabilityRequirement> {
        vec![CapabilityRequirement::Cpu, CapabilityRequirement::Ram, CapabilityRequirement::Disk]
    }

    fn execution_mode(&self) -> crate::executor::action::ExecutionMode {
        crate::executor::action::ExecutionMode::DirectDispatch
    }

    async fn validate(&self, _snapshot: &HostSnapshot) -> Result<(), String> {
        Ok(())
    }

    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec![
                "Gathering CPU information...".to_string(),
                "Checking Memory usage...".to_string(),
                "Scanning Disk partitions...".to_string(),
                "Probing network interfaces...".to_string(),
            ],
            estimated_impact: "Retrieves comprehensive system health metrics.".to_string(),
            danger_level: DangerLevel::Safe,
        })
    }

    fn build_command(&self) -> String {
        // Combined command for comprehensive system info
        if self.target_host == "localhost" {
            "uname -a && uptime && free -h && df -h / && ip -brief addr".to_string()
        } else {
            "uname -a && uptime && free -h && df -h / && ip -brief addr".to_string()
        }
    }

    async fn execute(&self) -> Result<ExecuteResult, String> {
        // The actual execution is handled by the orchestrator using build_command
        Ok(ExecuteResult {
            success: true,
            status: crate::executor::ExecutionStatus::Success,
            stdout: "".to_string(),
            stderr: "".to_string(),
            exit_code: Some(0),
            error: None, insight: None,
        })
    }

    fn parse_output(&self, result: &ExecuteResult) -> ActionOutput {
        ActionOutput::Raw(result.stdout.clone())
    }
}

pub struct SystemShutdown {
    pub id: String,
    pub target_host: String,
}

impl SystemShutdown {
    pub fn new(target_host: String) -> Self {
        Self {
            id: format!("halt-{}", Uuid::new_v4().to_string()[..8].to_string()),
            target_host,
        }
    }
}

#[async_trait]
impl Action for SystemShutdown {
    fn id(&self) -> String { self.id.clone() }
    fn name(&self) -> String { "System Shutdown".to_string() }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Critical }
    fn execution_mode(&self) -> crate::executor::action::ExecutionMode {
        crate::executor::action::ExecutionMode::DirectDispatch
    }
    fn requires_root(&self) -> bool { true }
    fn requires_nopasswd(&self) -> bool { true }

    async fn validate(&self, _snapshot: &HostSnapshot) -> Result<(), String> {
        Ok(())
    }

    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec![
                "Broadcasting shutdown message to all users...".to_string(),
                "Stopping system services...".to_string(),
                "Unmounting file systems...".to_string(),
                "Powering off...".to_string(),
            ],
            estimated_impact: "Immediate system shutdown and power off.".to_string(),
            danger_level: DangerLevel::Critical,
        })
    }

    fn build_command(&self) -> String {
        "sudo shutdown -h now".to_string()
    }

    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult {
            success: true,
            status: crate::executor::ExecutionStatus::Success,
            stdout: "Shutdown initiated.".to_string(),
            stderr: "".to_string(),
            exit_code: Some(0),
            error: None, insight: None,
        })
    }
}
