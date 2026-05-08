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
    fn skip_snapshot(&self) -> bool { true }
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
        Ok(ExecuteResult { success: true, stdout: String::new(), stderr: String::new(), exit_code: Some(0) })
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
    fn skip_snapshot(&self) -> bool { true }
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
        Ok(ExecuteResult { success: true, stdout: String::new(), stderr: String::new(), exit_code: Some(0) })
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
    fn skip_snapshot(&self) -> bool { true }
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
        Ok(ExecuteResult { success: true, stdout: String::new(), stderr: String::new(), exit_code: Some(0) })
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
    fn skip_snapshot(&self) -> bool { true }
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
    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult {
            success: true,
            stdout: format!("Model '{}' removed successfully.", self.model_name),
            stderr: String::new(),
            exit_code: Some(0),
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
    fn skip_snapshot(&self) -> bool { true }
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
        Ok(ExecuteResult { success: true, stdout: String::new(), stderr: String::new(), exit_code: Some(0) })
    }
}
