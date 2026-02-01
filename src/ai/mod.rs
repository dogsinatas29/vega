pub mod router;
pub mod providers;
pub mod prompts;

use async_trait::async_trait;
use crate::system::context::SystemContext;
use std::error::Error;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn generate_response(&self, context: &SystemContext, prompt: &str) -> Result<String, Box<dyn Error>>;
}
