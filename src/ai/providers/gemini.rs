use crate::ai::{AiProvider, QuotaStatus};
use crate::security::keyring;
use crate::context::SystemContext;
use async_trait::async_trait;
use std::error::Error;
// use std::env; // Replaced by keyring
use reqwest::Client;
use serde_json::json;

pub struct GeminiProvider {
    api_key: Option<String>,
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
        let names: Vec<String> = json["models"].as_array()
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

    async fn generate_response(&self, context: &SystemContext, prompt: &str) -> Result<String, Box<dyn Error>> {
        let clean_model = self.model.trim_start_matches("models/");
        
        let mut url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            clean_model
        );

        let mut builder = self.client.post(&url);

        // Auth Logic
        if let Some(token) = &self.oauth_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
             // Optional: Add x-goog-user-project if needed
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
            .await?;

        let status = res.status();
        if !status.is_success() {
            let error_text = res.text().await?;
            let debug_url = if let Some(key) = &self.api_key {
                url.replace(key, "[REDACTED]")
            } else {
                url.clone()
            };
            
            // Auto-Discovery on 404
            let available_models = if status.as_u16() == 404 {
                self.list_models().await.unwrap_or_else(|_| "Could not list models".to_string())
            } else {
                "N/A".to_string()
            };

            // Hint for Quota Exceeded (429)
            if status.as_u16() == 429 && self.oauth_token.is_none() {
                eprintln!("\n⚠️  API Key Quota Exceeded!");
                eprintln!("💡 Tip: Run 'vega login' to switch to Google OAuth (Higher Quota).\n");
            }

            return Err(format!("Gemini API Error (URL: {})\nError: {}\nAvailable Models: {}", debug_url, error_text, available_models).into());
        }

        let json_res: serde_json::Value = res.json().await?;
        
        // Debug
        // println!("DEBUG Response: {:?}", json_res);

        let mut output = json_res["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("{}")
            .to_string();

        // Cleanup potential markdown formatting if model ignores rule 5
        output = output.replace("```json", "").replace("```", "").trim().to_string();

        Ok(output)
    }
}
