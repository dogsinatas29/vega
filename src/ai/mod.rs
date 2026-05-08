use serde::{Deserialize, Serialize};

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
use std::error::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RiskLevel {
    INFO,
    WARNING,
    CRITICAL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub thought: String,
    pub action: String, // OLLAMA_REMOVE, OLLAMA_PULL, etc.
    pub target: String, // IP or hostname
    pub params: serde_json::Value, // Dynamic parameters
    pub explanation: String,
    pub risk_level: RiskLevel,
    pub needs_clarification: bool,
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
}
pub mod auth_manager;
pub mod prompts;
pub mod providers;
pub mod router;
pub mod intent;
pub mod generator;impl AiResponse {
    pub fn extract_json(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        
        // 1. Direct parse attempt
        if let Ok(res) = serde_json::from_str::<Self>(trimmed) {
            return Some(res);
        }

        // 2. Markdown Block extraction
        if let Some(start) = trimmed.find("```json") {
            if let Some(end) = trimmed[start + 7..].find("```") {
                let json_content = &trimmed[start + 7..start + 7 + end].trim();
                if let Ok(res) = serde_json::from_str::<Self>(json_content) {
                    return Some(res);
                }
            }
        }

        // 3. Brute-force curly brace find
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                let json_content = &trimmed[start..=end];
                if let Ok(res) = serde_json::from_str::<Self>(json_content) {
                    return Some(res);
                }
            }
        }

        None
    }
}
