use crate::ai::{AiError, AiProvider, QuotaStatus};
use crate::context::SystemContext;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct OllamaProvider {
    client: Client,
    endpoint: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(endpoint: String, model: String) -> Self {
        Self {
            client: Client::new(),
            endpoint,
            model,
        }
    }

    pub async fn list_models(endpoint: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let client = Client::new();
        let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));
        
        let res = client.get(&url).send().await?;
        if !res.status().is_success() {
            return Err(format!("Failed to list models: {}", res.status()).into());
        }

        let json: serde_json::Value = res.json().await?;
        let models = json["models"]
            .as_array()
            .ok_or("Invalid response from Ollama")?
            .iter()
            .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
            .collect();

        Ok(models)
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    fn name(&self) -> &str {
        "Local LLM (Ollama)"
    }

    fn get_quota_status(&self) -> QuotaStatus {
        QuotaStatus::Unlimited
    }

    async fn generate_response(
        &self,
        context: &SystemContext,
        prompt: &str,
    ) -> Result<String, AiError> {
        let url = format!("{}/api/generate", self.endpoint.trim_end_matches('/'));

        let system_persona = crate::ai::prompts::SystemPrompt::build(context);
        
        // Stronger prompt for local models to force JSON
        let full_prompt = format!(
            "{}\n\nIMPORTANT: RESPOND ONLY IN VALID JSON. NO TALKING. NO EXPLANATION BEFORE JSON. START WITH '{{' AND END WITH '}}'.\n\nUser Request: \"{}\"",
            system_persona, prompt
        );

        let body = json!({
            "model": self.model,
            "prompt": full_prompt,
            "stream": false,
            "format": "json"
        });

        let res = self.client.post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(AiError::Unknown(format!("Ollama error: {}", error_text)));
        }

        let json_res: serde_json::Value = res.json().await
            .map_err(|e| AiError::Unknown(format!("JSON Parse Error: {}", e)))?;

        let output = json_res["response"]
            .as_str()
            .ok_or_else(|| AiError::Unknown("Missing 'response' field in Ollama output".to_string()))?;

        // Robust JSON extraction
        let cleaned = match output.find('{') {
            Some(start_idx) => {
                match output.rfind('}') {
                    Some(end_idx) if end_idx > start_idx => {
                        output[start_idx..=end_idx].to_string()
                    },
                    _ => output.trim().to_string(),
                }
            },
            None => output.trim().to_string(),
        };

        // Final cleanup of common markdown artifacts if extraction missed them
        let final_cleaned = cleaned
            .replace("```json", "")
            .replace("```", "")
            .trim()
            .to_string();

        Ok(final_cleaned)
    }
}
