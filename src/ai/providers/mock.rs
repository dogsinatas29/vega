use crate::ai::{AiProvider, QuotaStatus};
use crate::context::SystemContext;
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;

pub struct MockProvider {
    responses: HashMap<String, String>,
}

impl MockProvider {
    pub fn new() -> Self {
        let mut responses = HashMap::new();

        // Default Canned Responses (JSON formatted with 'thought' field)
        let hello_res = json!({
            "thought": "User said hello. Providing a friendly mock response.",
            "command": "echo 'Hello from MockProvider!'",
            "explanation": "Simple greeting response.",
            "risk_level": "INFO",
            "needs_clarification": false
        });
        responses.insert("hello".to_string(), hello_res.to_string());

        let status_res = json!({
            "thought": "Checking mock system status.",
            "command": "echo 'Mock Systems Operational.'",
            "explanation": "System health check simulation.",
            "risk_level": "INFO",
            "needs_clarification": false
        });
        responses.insert("status".to_string(), status_res.to_string());

        Self { responses }
    }

    #[allow(dead_code)]
    pub fn add_response(&mut self, trigger: &str, response_json: &str) {
        self.responses
            .insert(trigger.to_lowercase(), response_json.to_string());
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

        let default_res = json!({
            "thought": "No specific trigger found in prompt. Falling back to default mock response.",
            "command": "ls -la",
            "explanation": "Default mock command.",
            "risk_level": "INFO",
            "needs_clarification": false
        });
        Ok(default_res.to_string())
    }
}
