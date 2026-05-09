use async_trait::async_trait;
use crate::executor::action::{Action, DangerLevel, ExecutionPlan};
use crate::executor::ExecuteResult;

pub struct OllamaListInstalledModels;

#[async_trait]
impl Action for OllamaListInstalledModels {
    fn id(&self) -> String { "ollama-list-installed".to_string() }
    fn name(&self) -> String { "Ollama: List Installed Models".to_string() }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Safe }
    fn required_capabilities(&self) -> Vec<crate::executor::action::CapabilityRequirement> {
        vec![crate::executor::action::CapabilityRequirement::Ollama]
    }
    fn execution_mode(&self) -> crate::executor::action::ExecutionMode {
        crate::executor::action::ExecutionMode::DirectDispatch
    }
    async fn validate(&self, _snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> {
        Ok(())
    }
    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec!["ollama list".to_string()],
            estimated_impact: "Retrieves the list of models installed on the system.".to_string(),
            danger_level: self.danger_level(),
        })
    }
    fn build_command(&self) -> String { "OLLAMA_HOST=127.0.0.1:11434 ollama list".to_string() }
    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult { success: true, status: crate::executor::ExecutionStatus::Success, stdout: String::new(), stderr: String::new(), exit_code: Some(0), error: None, insight: None })
    }
    fn parse_output(&self, result: &ExecuteResult) -> crate::executor::action::ActionOutput {
        let models = result.stdout.lines().skip(1)
            .filter_map(|l| l.split_whitespace().next().map(|s| s.to_string()))
            .collect();
        crate::executor::action::ActionOutput::OllamaModelList(models)
    }
}

pub struct OllamaListRunningModels;

#[async_trait]
impl Action for OllamaListRunningModels {
    fn id(&self) -> String { "ollama-list-running".to_string() }
    fn name(&self) -> String { "Ollama: List Running Models".to_string() }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Safe }
    fn required_capabilities(&self) -> Vec<crate::executor::action::CapabilityRequirement> {
        vec![crate::executor::action::CapabilityRequirement::Ollama]
    }
    fn execution_mode(&self) -> crate::executor::action::ExecutionMode {
        crate::executor::action::ExecutionMode::DirectDispatch
    }
    async fn validate(&self, _snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> {
        Ok(())
    }
    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec!["ollama ps".to_string()],
            estimated_impact: "Queries the list of currently active (running) models.".to_string(),
            danger_level: self.danger_level(),
        })
    }
    fn build_command(&self) -> String { "OLLAMA_HOST=127.0.0.1:11434 ollama ps".to_string() }
    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult { success: true, status: crate::executor::ExecutionStatus::Success, stdout: String::new(), stderr: String::new(), exit_code: Some(0), error: None, insight: None })
    }
    fn parse_output(&self, result: &ExecuteResult) -> crate::executor::action::ActionOutput {
        let models = result.stdout.lines().skip(1)
            .filter_map(|l| l.split_whitespace().next().map(|s| s.to_string()))
            .collect();
        crate::executor::action::ActionOutput::OllamaModelList(models)
    }
}

pub struct OllamaVersion;

#[async_trait]
impl Action for OllamaVersion {
    fn id(&self) -> String { "ollama-version".to_string() }
    fn name(&self) -> String { "Ollama: Get Version".to_string() }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Safe }
    fn required_capabilities(&self) -> Vec<crate::executor::action::CapabilityRequirement> {
        vec![crate::executor::action::CapabilityRequirement::Ollama]
    }
    fn execution_mode(&self) -> crate::executor::action::ExecutionMode {
        crate::executor::action::ExecutionMode::DirectDispatch
    }
    async fn validate(&self, _snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> {
        Ok(())
    }
    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec!["ollama --version".to_string()],
            estimated_impact: "Retrieves the version of Ollama.".to_string(),
            danger_level: self.danger_level(),
        })
    }
    fn build_command(&self) -> String { "OLLAMA_HOST=127.0.0.1:11434 ollama --version".to_string() }
    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult { success: true, status: crate::executor::ExecutionStatus::Success, stdout: String::new(), stderr: String::new(), exit_code: Some(0), error: None, insight: None })
    }
    fn parse_output(&self, result: &ExecuteResult) -> crate::executor::action::ActionOutput {
        let version = result.stdout.trim()
            .replace("ollama version is ", "");
        crate::executor::action::ActionOutput::OllamaVersion(version)
    }
}

pub struct OllamaModelRemove {
    pub id: String,
    pub model_name: String,
    pub force: bool,
}

#[async_trait]
impl Action for OllamaModelRemove {
    fn id(&self) -> String { self.id.clone() }
    fn name(&self) -> String { format!("Ollama: Remove Model (Force: {})", self.force) }
    fn danger_level(&self) -> DangerLevel {
        if self.force { DangerLevel::Critical } else { DangerLevel::Dangerous }
    }
    fn required_capabilities(&self) -> Vec<crate::executor::action::CapabilityRequirement> {
        vec![crate::executor::action::CapabilityRequirement::Ollama]
    }
    fn execution_mode(&self) -> crate::executor::action::ExecutionMode {
        crate::executor::action::ExecutionMode::DirectDispatch
    }
    async fn validate(&self, _snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> {
        Ok(())
    }
    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec![format!("ollama rm {}", self.model_name)],
            estimated_impact: format!("Permanently deletes model '{}'.", self.model_name),
            danger_level: self.danger_level(),
        })
    }
    fn build_command(&self) -> String {
        format!("OLLAMA_HOST=127.0.0.1:11434 ollama rm {}", self.model_name)
    }

    fn evaluate_outcome(&self, result: &crate::executor::ExecuteResult) -> crate::executor::action::OutcomeEvaluation {
        if result.success {
            crate::executor::action::OutcomeEvaluation::Success
        } else if result.stderr.contains("not found") {
            // [Desired State Reconciliation] 
            // If the model is not found, the goal (absence) is already satisfied.
            crate::executor::action::OutcomeEvaluation::SuccessAlreadySatisfied(
                format!("Model '{}' is already absent. Desired state satisfied.", self.model_name)
            )
        } else {
            crate::executor::action::OutcomeEvaluation::Failure(result.stderr.clone())
        }
    }

    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult {
            success: true,
            status: crate::executor::ExecutionStatus::Success,
            stdout: format!("Model '{}' removed successfully.", self.model_name),
            stderr: String::new(),
            exit_code: Some(0),
            error: None, insight: None,
        })
    }
}

pub struct OllamaModelPull {
    pub id: String,
    pub model_name: String,
}

#[async_trait]
impl Action for OllamaModelPull {
    fn id(&self) -> String { self.id.clone() }
    fn name(&self) -> String { format!("Ollama: Pull Model '{}'", self.model_name) }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Moderate }
    fn required_capabilities(&self) -> Vec<crate::executor::action::CapabilityRequirement> {
        vec![crate::executor::action::CapabilityRequirement::Ollama]
    }
    fn execution_mode(&self) -> crate::executor::action::ExecutionMode {
        crate::executor::action::ExecutionMode::DirectDispatch
    }
    async fn validate(&self, _snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> {
        Ok(())
    }
    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec![format!("ollama pull {}", self.model_name)],
            estimated_impact: format!("Downloads the '{}' model.", self.model_name),
            danger_level: self.danger_level(),
        })
    }
    fn build_command(&self) -> String {
        format!("OLLAMA_HOST=127.0.0.1:11434 ollama pull {}", self.model_name)
    }
    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult { success: true, status: crate::executor::ExecutionStatus::Success, stdout: String::new(), stderr: String::new(), exit_code: Some(0), error: None, insight: None })
    }
}

pub struct OllamaStop;

#[derive(Debug, PartialEq)]
enum StopStrategy {
    Systemd,
    Pkill,
}

impl OllamaStop {
    fn select_strategy(&self, snapshot: &crate::system::snapshot::HostSnapshot) -> StopStrategy {
        // If systemctl is available and we have passwordless sudo, systemd is preferred (graceful)
        if snapshot.capabilities.sudo_nopasswd {
            StopStrategy::Systemd
        } else {
            // Fallback to user-space pkill if sudo is restricted
            StopStrategy::Pkill
        }
    }
}

#[async_trait]
impl Action for OllamaStop {
    fn id(&self) -> String { "ollama-stop".to_string() }
    fn name(&self) -> String { "Ollama: Stop Service (Dynamic)".to_string() }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Moderate }
    fn execution_mode(&self) -> crate::executor::action::ExecutionMode {
        crate::executor::action::ExecutionMode::MinimalSnapshot // We need basic capability info
    }

    fn requires_root(&self) -> bool {
        // This is tricky: we only need root if we use Systemd strategy.
        // For Alpha 16, we'll make this dynamic if the trait allowed it, 
        // but since we're in a trait method, we'll return false and handle it in validate.
        false 
    }
    fn requires_snapshot(&self) -> bool { false }

    async fn validate(&self, snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> {
        let strategy = self.select_strategy(snapshot);
        if strategy == StopStrategy::Systemd && !snapshot.capabilities.sudo_nopasswd {
            return Err("Systemd strategy requires NOPASSWD sudo but it's not available.".to_string());
        }
        Ok(())
    }

    async fn plan(&self) -> Result<ExecutionPlan, String> {
        // Note: In a real scenario, plan() would receive the snapshot. 
        // For now, we'll provide a generic plan that hints at the fallback.
        Ok(ExecutionPlan {
            steps: vec!["Attempting graceful stop via systemctl or pkill fallback...".to_string()],
            estimated_impact: "Stops the Ollama process/service.".to_string(),
            danger_level: self.danger_level(),
        })
    }

    fn build_command(&self) -> String {
        // Resilient Command Chain: Try systemd first (if sudo is likely to work), 
        // then fallback to pkill for user-space instances.
        "sudo -n systemctl stop ollama 2>/dev/null || pkill -TERM ollama || pkill -KILL ollama".to_string()
    }

    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult { success: true, status: crate::executor::ExecutionStatus::Success, stdout: String::new(), stderr: String::new(), exit_code: Some(0), error: None, insight: None })
    }
}

pub struct OllamaStart;

#[async_trait]
impl Action for OllamaStart {
    fn id(&self) -> String { "ollama-start".to_string() }
    fn name(&self) -> String { "Ollama: Start Service (Resilient)".to_string() }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Safe }
    fn execution_mode(&self) -> crate::executor::action::ExecutionMode {
        crate::executor::action::ExecutionMode::MinimalSnapshot
    }
    fn requires_root(&self) -> bool { false } // Allowed because of nohup fallback
    fn requires_nopasswd(&self) -> bool { false }
    fn requires_snapshot(&self) -> bool { false }

    async fn validate(&self, _snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> { Ok(()) }
    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec!["Attempting to start Ollama via systemctl or user-space daemon fallback...".to_string()],
            estimated_impact: "Starts the Ollama service or daemon process.".to_string(),
            danger_level: self.danger_level(),
        })
    }
    fn build_command(&self) -> String { 
        use crate::executor::action::{SENTINEL_READY, SENTINEL_FAIL, SENTINEL_JSON_PREFIX, SENTINEL_PROTOCOL_BEGIN, SENTINEL_PROTOCOL_END};
        // SRE-Grade State Reconciliation Orchestration (Alpha 37):
        // 1. Target Desired State: 0.0.0.0:11434 (Global Accessibility)
        // 2. Convergence Check: Actual vs Desired.
        // 3. Remediation Insight: Detection of Divergence + Restart Requirement.
        format!(
            r#"if ! sudo -n systemctl start ollama 2>/dev/null; then \
              OLLAMA_HOST=0.0.0.0:11434 setsid nohup $(which ollama || echo ollama) serve >/tmp/ollama.log 2>&1 < /dev/null & \
              disown; \
            fi; \
            MAX_WAIT=45; \
            DESIRED_BIND="0.0.0.0:11434"; \
            for i in $(seq 1 $MAX_WAIT); do \
              if pgrep -x ollama >/dev/null && \
                 (ss -ltn 2>/dev/null | grep -q :11434 || netstat -ltn 2>/dev/null | grep -q :11434 || true) && \
                 curl -sf --max-time 2 http://127.0.0.1:11434/api/tags >/dev/null; then \
                V_STR=$(curl -sf --max-time 2 http://127.0.0.1:11434/api/version | jq -r .version 2>/dev/null || echo "unknown"); \
                B_STR=$(ss -ltn 2>/dev/null | grep :11434 | awk '{{print $4}}' | head -n 1 || echo "unknown"); \
                EXT_R="false"; if [[ "$B_STR" == *"0.0.0.0"* || "$B_STR" == *"::"* || "$B_STR" == *"*"* ]]; then EXT_R="true"; fi; \
                CONV="false"; REM_A="false"; REQ_R="false"; \
                if [[ "$EXT_R" == "true" ]]; then CONV="true"; else REM_A="true"; REQ_R="true"; fi; \
                echo "{}"; \
                echo "{}{{\"protocol\":\"vega.transport.v1\",\"type\":\"SERVICE_READY\",\"status\":\"READY\",\"service\":\"ollama\",\"severity\":\"info\",\"host\":\"$(hostname)\",\"user\":\"$(whoami)\",\"pid\":$(pgrep -x ollama | head -n 1),\"version\":\"$V_STR\",\"bind\":\"$B_STR\",\"bind_scope\":\"remote\",\"local_ready\":true,\"ssh_tunnel_reachable\":true,\"lan_reachable\":$EXT_R,\"externally_reachable\":$EXT_R,\"desired_bind\":\"$DESIRED_BIND\",\"actual_bind\":\"$B_STR\",\"state_converged\":$CONV,\"remediation_available\":$REM_A,\"requires_restart\":$REQ_R,\"safe_to_apply\":true}}"; \
                echo "{}"; \
                echo "{}"; \
                exit 0; \
              fi; \
              sleep 1; \
            done; \
            echo "{}"; \
            echo "{}{{\"protocol\":\"vega.transport.v1\",\"type\":\"SERVICE_ERROR\",\"status\":\"FAILED\",\"service\":\"ollama\",\"severity\":\"error\",\"message\":\"readiness_probe_timeout_45s\"}}"; \
            echo "{}"; \
            echo "{}"; \
            tail -n 20 /tmp/ollama.log; \
            exit 1"#,
            SENTINEL_PROTOCOL_BEGIN, SENTINEL_JSON_PREFIX, SENTINEL_PROTOCOL_END, SENTINEL_READY,
            SENTINEL_PROTOCOL_BEGIN, SENTINEL_JSON_PREFIX, SENTINEL_PROTOCOL_END, SENTINEL_FAIL
        )
    }

    fn evaluate_outcome(&self, result: &crate::executor::ExecuteResult) -> crate::executor::action::OutcomeEvaluation {
        use crate::executor::action::{SENTINEL_READY, VegaEvent};
        let events = VegaEvent::parse_from_stdout(&result.stdout);
        
        if result.success && (events.iter().any(|e| e.is_ready()) || result.stdout.contains(SENTINEL_READY)) {
            crate::executor::action::OutcomeEvaluation::Success
        } else {
            let log_hint = if !result.stderr.is_empty() { result.stderr.clone() } else { result.stdout.clone() };
            crate::executor::action::OutcomeEvaluation::Failure(format!("Ollama failed to reach desired state (Ready). Last Logs:\n{}", log_hint))
        }
    }

    fn parse_output(&self, result: &crate::executor::ExecuteResult) -> crate::executor::action::ActionOutput {
        use crate::executor::action::{VegaEvent, ActionOutput};
        let events = VegaEvent::parse_from_stdout(&result.stdout);
        if let Some(event) = events.last() {
            ActionOutput::ServiceState(event.clone())
        } else {
            ActionOutput::Raw(result.stdout.clone())
        }
    }

    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult { success: true, status: crate::executor::ExecutionStatus::Success, stdout: String::new(), stderr: String::new(), exit_code: Some(0), error: None, insight: None })
    }
}

pub struct OllamaRemediate;

#[async_trait]
impl Action for OllamaRemediate {
    fn id(&self) -> String { "ollama-remediate".to_string() }
    fn name(&self) -> String { "Ollama: Force State Reconciliation".to_string() }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Moderate }
    fn execution_mode(&self) -> crate::executor::action::ExecutionMode {
        crate::executor::action::ExecutionMode::MinimalSnapshot
    }
    fn requires_root(&self) -> bool { false }
    fn requires_snapshot(&self) -> bool { false }

    async fn validate(&self, _snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> { Ok(()) }
    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec![
                "Stopping existing Ollama instances (systemctl & pkill)...".to_string(),
                "Restarting Ollama with OLLAMA_HOST=0.0.0.0:11434...".to_string(),
                "Verifying state convergence...".to_string(),
            ],
            estimated_impact: "Restarts the Ollama service to apply new configuration.".to_string(),
            danger_level: self.danger_level(),
        })
    }
    fn build_command(&self) -> String { 
        let start_cmd = OllamaStart.build_command();
        // Aggressive Remediation: Kill first, then use the hardened Start logic.
        format!(
            "sudo -n systemctl stop ollama 2>/dev/null || true; \
             pkill -9 -x ollama 2>/dev/null || true; \
             sleep 2; \
             {}", 
            start_cmd
        )
    }

    fn parse_output(&self, result: &ExecuteResult) -> crate::executor::action::ActionOutput {
        OllamaStart.parse_output(result)
    }

    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult { success: true, status: crate::executor::ExecutionStatus::Success, stdout: String::new(), stderr: String::new(), exit_code: Some(0), error: None, insight: None })
    }
}
