use async_trait::async_trait;
use crate::executor::action::{Action, DangerLevel, ExecutionMode, ExecutionPlan};
use crate::executor::ExecuteResult;

pub struct SshListTargetsAction;

#[async_trait]
impl Action for SshListTargetsAction {
    fn id(&self) -> String { "infra-list-ssh".to_string() }
    fn name(&self) -> String { "List SSH Targets".to_string() }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Safe }
    fn execution_mode(&self) -> ExecutionMode { ExecutionMode::DirectDispatch }

    async fn validate(&self, _snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> { Ok(()) }

    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec!["Scan Knowledge Base for registered SSH targets".to_string(), "Scan SSH config for discovered hosts".to_string()],
            estimated_impact: "Lists available remote management targets.".to_string(),
            danger_level: DangerLevel::Safe,
        })
    }

    fn build_command(&self) -> String { "echo 'Listing SSH targets from context...'".to_string() }

    async fn execute(&self) -> Result<ExecuteResult, String> {
        // 이 액션은 오케스트레이터의 'Sovereign Discovery' 단계에서 이미 수집된 정보를 출력합니다.
        // 실제 실행은 로컬에서 컨텍스트를 포맷팅하는 것으로 대체됩니다.
        Ok(ExecuteResult {
            success: true,
            status: crate::executor::ExecutionStatus::Success,
            stdout: "인벤토리에서 관리 중인 SSH 타겟 목록입니다.".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            error: None,
            insight: Some("Discovery Context를 통해 타겟을 확인하세요.".to_string()),
        })
    }
}

pub struct InfraListRemotesAction;

#[async_trait]
impl Action for InfraListRemotesAction {
    fn id(&self) -> String { "infra-list-remotes".to_string() }
    fn name(&self) -> String { "List Infrastructure Remotes".to_string() }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Safe }
    fn execution_mode(&self) -> ExecutionMode { ExecutionMode::DirectDispatch }

    async fn validate(&self, _snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> { Ok(()) }

    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec!["List rclone remotes".to_string()],
            estimated_impact: "Lists registered cloud storage and remote endpoints.".to_string(),
            danger_level: DangerLevel::Safe,
        })
    }

    fn build_command(&self) -> String { "rclone listremotes".to_string() }

    async fn execute(&self) -> Result<ExecuteResult, String> {
        let output = std::process::Command::new("rclone")
            .arg("listremotes")
            .output()
            .map_err(|e| format!("rclone 실행 실패: {}", e))?;

        Ok(ExecuteResult {
            success: output.status.success(),
            status: if output.status.success() { crate::executor::ExecutionStatus::Success } else { crate::executor::ExecutionStatus::FatalFailure },
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
            error: None,
            insight: None,
        })
    }
}
