use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use crate::ai::AiProvider;
use crate::context::SystemContext;
use crate::security::keyring;

pub struct VertexAiProvider {
    oauth_token: Option<String>,
    client: Client,
    project_id: String,
    region: String,
    model: String,
}

impl VertexAiProvider {
    pub fn new(project_id: String, region: String) -> Result<Self, Box<dyn std::error::Error>> {
        // Vertex AI requires OAuth token (no API key support)
        let oauth_token = keyring::get_token("google_oauth_token");

        if oauth_token.is_none() {
            return Err("Vertex AI requires OAuth authentication. Please run 'vega login' first.".into());
        }

        println!("🔑 DEBUG: Using OAuth Token for Vertex AI.");

        Ok(Self {
            oauth_token,
            client: Client::new(),
            project_id,
            region,
            model: "gemini-2.5-flash".to_string(),
        })
    }

    fn build_url(&self) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.region, self.project_id, self.region, self.model
        )
    }
}

#[async_trait]
impl AiProvider for VertexAiProvider {
    fn name(&self) -> &str {
        "Vertex AI"
    }

    fn get_quota_status(&self) -> crate::ai::QuotaStatus {
        crate::ai::QuotaStatus::Unlimited
    }

    async fn generate_response(&self, ctx: &SystemContext, user_input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = self.build_url();
        let system_prompt = crate::ai::prompts::SystemPrompt::build(ctx);

        let payload = json!({
            "contents": [{
                "role": "user",
                "parts": [{
                    "text": format!("{}\n\nUser Request: {}", system_prompt, user_input)
                }]
            }],
            "generationConfig": {
                "temperature": 0.7,
                "maxOutputTokens": 2048
            }
        });

        let mut req = self.client.post(&url).json(&payload);

        // Vertex AI uses OAuth token only
        if let Some(token) = &self.oauth_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let res = req.send().await?;
        let status = res.status();

        if !status.is_success() {
            let error_text = res.text().await?;
            return Err(format!("Vertex AI Error (Status: {})\nError: {}", status, error_text).into());
        }

        let json_res: serde_json::Value = res.json().await?;

        // Parse Vertex AI response
        let text = json_res["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("No response from Vertex AI")
            .to_string();

        Ok(text)
    }
}
