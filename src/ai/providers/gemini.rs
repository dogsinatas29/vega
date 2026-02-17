use crate::ai::{AiProvider, QuotaStatus};
use crate::context::SystemContext;
use crate::security::keyring;
use async_trait::async_trait;
use std::error::Error;
// use std::env; // Replaced by keyring
use reqwest::Client;
use serde_json::json;

pub struct GeminiProvider {
    api_key: Option<String>,
    #[allow(dead_code)]
    oauth_token: Option<String>,
    client: Client,
    model: String,
}

impl GeminiProvider {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // 1. Try API Key
        let api_key = keyring::get_api_key("GEMINI_API_KEY");

        // 2. Try OAuth Token
        let oauth_token = keyring::get_token("google_oauth_token");

        if let Some(_) = &oauth_token {
            println!("🔑 DEBUG: Found OAuth Token in Keyring. Using it.");
        } else {
            println!("ℹ️  DEBUG: No OAuth Token found. Falling back to API Key.");
        }

        if api_key.is_none() && oauth_token.is_none() {
            return Err("No authentication found (API Key or OAuth Token)".into());
        }

        Ok(Self {
            api_key,
            oauth_token,
            client: Client::new(),
            model: "gemini-2.5-flash".to_string(),
        })
    }

    #[allow(dead_code)]
    pub async fn list_models(&self) -> Result<String, Box<dyn Error>> {
        // Listing models requires authentication.
        // If OAuth, use Bearer. If key, use param.
        let mut url = "https://generativelanguage.googleapis.com/v1beta/models".to_string();

        let mut builder = self.client.get(&url);

        if let Some(token) = &self.oauth_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        } else if let Some(key) = &self.api_key {
            url.push_str(&format!("?key={}", key));
            // Re-create builder because URL changed
            builder = self.client.get(&url);
        }

        let res = builder.send().await?;
        let json: serde_json::Value = res.json().await?;

        // Extract model names
        let names: Vec<String> = json["models"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
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

    async fn generate_response(
        &self,
        context: &SystemContext,
        prompt: &str,
    ) -> Result<String, crate::ai::AiError> {
        let clean_model = self.model.trim_start_matches("models/");

        let mut url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            clean_model
        );

        let mut builder = self.client.post(&url);

        // Auth Logic
        if let Ok(token) = crate::ai::auth_manager::AuthManager::get_bearer_token().await {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        } else if let Some(key) = &self.api_key {
            url.push_str(&format!("?key={}", key));
            builder = self.client.post(&url);
        }

        // Use centralized System Prompt
        let system_persona = crate::ai::prompts::SystemPrompt::build(context);

        let body = json!({
            "contents": [{
                "parts": [{
                    "text": format!("{}\n\nUser Request: \"{}\"", system_persona, prompt)
                }]
            }]
        });

        let res = builder
            .json(&body)
            .send()
            .await
            .map_err(|e| crate::ai::AiError::NetworkError(e.to_string()))?;

        let status = res.status();
        if !status.is_success() {
            let error_text = res
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse error body for specific reasons (Google API format)
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                let status_str = error_json["error"]["status"].as_str().unwrap_or("");
                let message = error_json["error"]["message"].as_str().unwrap_or("");

                // Senior Tip: Map 403/503 to QuotaExceeded if reason matches
                if status_str == "RESOURCE_EXHAUSTED"
                    || status_str == "RATE_LIMIT_EXCEEDED"
                    || message.to_lowercase().contains("quota")
                    || message.to_lowercase().contains("rate limit")
                {
                    return Err(crate::ai::AiError::QuotaExceeded);
                }
            }

            // Fallback status code mapping
            if status.as_u16() == 429 {
                return Err(crate::ai::AiError::QuotaExceeded);
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(crate::ai::AiError::AuthError(error_text));
            }

            return Err(crate::ai::AiError::Unknown(format!(
                "Status: {}, Body: {}",
                status, error_text
            )));
        }

        let json_res: serde_json::Value = res
            .json()
            .await
            .map_err(|e| crate::ai::AiError::Unknown(format!("JSON Parse Error: {}", e)))?;

        // Debug
        // println!("DEBUG Response: {:?}", json_res);

        let mut output = json_res["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("{}")
            .to_string();

        // Cleanup potential markdown formatting if model ignores rule 5
        output = output
            .replace("```json", "")
            .replace("```", "")
            .trim()
            .to_string();

        Ok(output)
    }
}
