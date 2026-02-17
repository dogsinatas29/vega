use crate::ai::{AiProvider, QuotaStatus};
use crate::context::SystemContext;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct MockProvider {
    responses: HashMap<String, String>,
}

impl MockProvider {
    pub fn new() -> Self {
        let mut responses = HashMap::new();
        // Default Canned Responses
        responses.insert("hello".to_string(), "Hello from MockProvider!".to_string());
        responses.insert(
            "status".to_string(),
            "Mock Systems Operational.".to_string(),
        );

        Self { responses }
    }

    #[allow(dead_code)]
    pub fn add_response(&mut self, trigger: &str, response: &str) {
        self.responses
            .insert(trigger.to_lowercase(), response.to_string());
    }
}

#[async_trait]
impl AiProvider for MockProvider {
    fn name(&self) -> &str {
        "Mock AI Provider"
    }

    fn get_quota_status(&self) -> QuotaStatus {
        QuotaStatus::Unlimited
    }

    async fn generate_response(
        &self,
        _context: &SystemContext,
        prompt: &str,
    ) -> Result<String, crate::ai::AiError> {
        let p = prompt.to_lowercase();

        if p.contains("error") {
            return Err(crate::ai::AiError::Unknown(
                "Mock Simulated Error".to_string(),
            ));
        }

        for (trigger, resp) in &self.responses {
            if p.contains(trigger) {
                return Ok(resp.clone());
            }
        }

        Ok("Mock Default Response: I hear you.".to_string())
    }
}
