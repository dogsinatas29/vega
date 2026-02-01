pub mod router;
pub mod providers;
pub mod prompts;

use async_trait::async_trait;
use crate::system::context::SystemContext;
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum QuotaStatus {
    Remaining(u32),
    Exhausted,
    Unlimited,
    Unknown,
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn generate_response(&self, context: &SystemContext, prompt: &str) -> Result<String, Box<dyn Error>>;
    fn get_quota_status(&self) -> QuotaStatus;
}
