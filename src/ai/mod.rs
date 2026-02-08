use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub enum LlmProvider {
    Gemini,
    ChatGPT,
    Claude,
}

use async_trait::async_trait;
use crate::context::SystemContext;
use std::error::Error;

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
    async fn generate_response(&self, context: &SystemContext, prompt: &str) -> Result<String, Box<dyn Error>>;
}
pub mod router;
pub mod providers;
pub mod prompts;
