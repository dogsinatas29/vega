use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub enum LlmProvider {
    Gemini,
    ChatGPT,
    Claude,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiResponse {
    pub command: String,
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
