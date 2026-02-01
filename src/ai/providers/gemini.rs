use crate::ai::{AiProvider, QuotaStatus};
use crate::security::keyring;
use crate::system::context::SystemContext;
use async_trait::async_trait;
use std::error::Error;
// use std::env; // Replaced by keyring
use reqwest::Client;
use serde_json::json;

pub struct GeminiProvider {
    api_key: String,
    client: Client,
    model: String,
}

impl GeminiProvider {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // use keyring with env fallback
        let api_key = keyring::get_api_key("GEMINI_API_KEY")
            .ok_or("GEMINI_API_KEY not found in keyring or environment")?;
            
        Ok(Self {
            api_key,
            client: Client::new(),
            model: "gemini-2.5-flash".to_string(), 
        })
    }
    pub async fn list_models(&self) -> Result<String, Box<dyn Error>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            self.api_key
        );
        let res = self.client.get(&url).send().await?;
        let json: serde_json::Value = res.json().await?;
        
        // Extract model names
        let names: Vec<String> = json["models"].as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
            .filter(|n| n.contains("flash")) // Filter for flash models
            .collect();
            
        Ok(format!("{:?}", names))
    }
}

#[async_trait]
impl AiProvider for GeminiProvider {
    fn name(&self) -> &str {
        "Google Gemini"
    }
    
    fn get_quota_status(&self) -> QuotaStatus {
        QuotaStatus::Unknown
    }

    async fn generate_response(&self, context: &SystemContext, prompt: &str) -> Result<String, Box<dyn Error>> {
        // Sanitize model name: ensure we don't duplicate 'models/'
        // If self.model is "gemini-1.5-flash", we want "models/gemini-1.5-flash"
        // If self.model is "models/gemini-1.5-flash", we want "models/gemini-1.5-flash"
        let clean_model = self.model.trim_start_matches("models/");
        
        // Revert to v1beta as requested for AI Studio keys
        // Endpoint: https://generativelanguage.googleapis.com/v1beta/models/{clean_model}:generateContent
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            clean_model, self.api_key
        );


        
        // Use centralized System Prompt
        let system_persona = crate::ai::prompts::SystemPrompt::build(context);

        let body = json!({
            "contents": [{
                "parts": [{
                    "text": format!("{}\n\nUser Request: \"{}\"", system_persona, prompt)
                }]
            }]
        });

        let res = self.client.post(&url)
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let error_text = res.text().await?;
            let debug_url = url.replace(&self.api_key, "[REDACTED]");
            
            // Auto-Discovery on 404
            let available_models = if status.as_u16() == 404 {
                self.list_models().await.unwrap_or_else(|_| "Could not list models".to_string())
            } else {
                "N/A".to_string()
            };

            return Err(format!("Gemini API Error (URL: {})\nError: {}\nAvailable Models: {}", debug_url, error_text, available_models).into());
        }

        let json_res: serde_json::Value = res.json().await?;
        
        let mut output = json_res["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("{}")
            .to_string();

        // Cleanup potential markdown formatting if model ignores rule 5
        output = output.replace("```json", "").replace("```", "").trim().to_string();

        Ok(output)
    }


}
