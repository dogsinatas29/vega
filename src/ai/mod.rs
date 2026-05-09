use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub enum LlmProvider {
    Gemini,
    ChatGPT,
    Claude,
    Ollama,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AiError {
    QuotaExceeded,
    AuthError(String),
    NetworkError(String),
    Unknown(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QuotaExceeded => write!(f, "AI Quota Exceeded (429)"),
            Self::AuthError(e) => write!(f, "Auth Error: {}", e),
            Self::NetworkError(e) => write!(f, "Network Error: {}", e),
            Self::Unknown(e) => write!(f, "AI Error: {}", e),
        }
    }
}

impl Error for AiError {}

use crate::context::SystemContext;
use async_trait::async_trait;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RiskLevel {
    INFO,
    WARNING,
    CRITICAL,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Domain {
    System,
    AiModels,
    Infrastructure,
    PackageManagement,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResponse {
    pub domain: Domain,
    pub confidence: f64,
    pub thought: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntentAction {
    OllamaPull,
    OllamaRemove,
    OllamaListInstalled,
    OllamaListRunning,
    OllamaVersion,
    OllamaStop,
    OllamaStart,
    SystemDiagnostic,
    SystemShutdown,
    SystemUpdate,
    SystemListProcesses,
    InfraListRemotes,
    SshListTargets,
    Unknown,
}

impl IntentAction {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "OLLAMA_PULL" => Self::OllamaPull,
            "OLLAMA_REMOVE" => Self::OllamaRemove,
            "OLLAMA_LIST_INSTALLED" => Self::OllamaListInstalled,
            "OLLAMA_LIST_RUNNING" => Self::OllamaListRunning,
            "OLLAMA_VERSION" => Self::OllamaVersion,
            "OLLAMA_STOP" => Self::OllamaStop,
            "OLLAMA_START" => Self::OllamaStart,
            "SYSTEM_DIAGNOSTIC" => Self::SystemDiagnostic,
            "SYSTEM_SHUTDOWN" => Self::SystemShutdown,
            "SYSTEM_UPDATE" => Self::SystemUpdate,
            "SYSTEM_LIST_PROCESSES" => Self::SystemListProcesses,
            "INFRA_LIST_REMOTES" => Self::InfraListRemotes,
            "SSH_LIST_TARGETS" => Self::SshListTargets,
            _ => Self::Unknown,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::OllamaPull => "OLLAMA_PULL",
            Self::OllamaRemove => "OLLAMA_REMOVE",
            Self::OllamaListInstalled => "OLLAMA_LIST_INSTALLED",
            Self::OllamaListRunning => "OLLAMA_LIST_RUNNING",
            Self::OllamaVersion => "OLLAMA_VERSION",
            Self::OllamaStop => "OLLAMA_STOP",
            Self::OllamaStart => "OLLAMA_START",
            Self::SystemDiagnostic => "SYSTEM_DIAGNOSTIC",
            Self::SystemShutdown => "SYSTEM_SHUTDOWN",
            Self::SystemUpdate => "SYSTEM_UPDATE",
            Self::SystemListProcesses => "SYSTEM_LIST_PROCESSES",
            Self::InfraListRemotes => "INFRA_LIST_REMOTES",
            Self::SshListTargets => "SSH_LIST_TARGETS",
            Self::Unknown => "UNKNOWN",
        }.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub thought: String,
    pub action: String, // Kept for JSON compatibility with LLM, but mapped to IntentAction
    pub target: Option<String>, 
    pub params: Value, 
    pub explanation: String,
    pub risk_level: String,
    pub needs_clarification: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum QuotaStatus {
    Unknown,
    Unlimited,
    Exceeded,
    Remaining(i64),
}

#[async_trait]
#[allow(dead_code)]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    fn get_quota_status(&self) -> QuotaStatus;
    async fn generate_response(
        &self,
        context: &SystemContext,
        prompt: &str,
    ) -> Result<String, AiError>;

    async fn generate_response_with_system(
        &self,
        context: &SystemContext,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, AiError> {
        // Default implementation fallback
        let combined = format!("### SYSTEM INSTRUCTION\n{}\n\n### USER REQUEST\n{}", system_prompt, user_prompt);
        self.generate_response(context, &combined).await
    }
}

pub mod auth_manager;
pub mod prompts;
pub mod providers;
pub mod router;
pub mod intent;
pub mod generator;
pub mod validator;

// --- Tolerant Parser Implementation ---

fn clean_json(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    
    // 1. Markdown Guard
    if let Some(start) = s.find("```json") {
        if let Some(end) = s[start + 7..].find("```") {
            s = s[start + 7..start + 7 + end].trim().to_string();
        }
    } else if let Some(start) = s.find("```") {
        if let Some(end) = s[start + 3..].find("```") {
            s = s[start + 3..start + 3 + end].trim().to_string();
        }
    }

    // 2. Brute-force curly brace find (if still not parsed)
    if !s.starts_with('{') {
        if let Some(start) = s.find('{') {
            if let Some(end) = s.rfind('}') {
                s = s[start..=end].to_string();
            }
        }
    }

    // 3. Common malformation repair
    s.replace("\\\"", "\"")
     .replace("\n", " ")
}

impl DomainResponse {
    pub fn extract_json(raw: &str) -> Option<Self> {
        let cleaned = clean_json(raw);
        serde_json::from_str(&cleaned).ok()
    }
}

impl AiResponse {
    pub fn extract_json(raw: &str) -> Option<Self> {
        let cleaned = clean_json(raw);
        serde_json::from_str(&cleaned).ok()
    }
}
