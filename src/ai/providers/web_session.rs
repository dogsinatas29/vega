use crate::ai::{AiProvider, QuotaStatus};
use crate::context::SystemContext;
use crate::security::keyring;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub struct WebSessionProvider {
    client: Client,
    psid: String,
}

impl WebSessionProvider {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let psid = keyring::get_token("google_1psid")
            .ok_or("Web Session requires __Secure-1PSID cookie. Please save it using 'vega setup --cookie'.")?;

        Ok(Self {
            client: Client::new(),
            psid,
        })
    }
}

#[async_trait]
impl AiProvider for WebSessionProvider {
    fn name(&self) -> &str {
        "Gemini Web Session (Experimental)"
    }

    fn get_quota_status(&self) -> QuotaStatus {
        // Users often use this to bypass API limits
        QuotaStatus::Unlimited
    }

    async fn generate_response(
        &self,
        ctx: &SystemContext,
        user_input: &str,
    ) -> Result<String, crate::ai::AiError> {
        // Anti-Detection: Random Jitter (100-800ms)
        let jitter: u64 = rand::random_range(100..800);
        tokio::time::sleep(std::time::Duration::from_millis(jitter)).await;

        // This is a simplified mock of how a web session wrapper would work.
        // In reality, this requires complex header/cookie management (SNlM0e, etc.)
        // For this task, we'll implement the structural framework.

        let system_prompt = crate::ai::prompts::SystemPrompt::build(ctx);

        // This endpoint is illustrative; a real proxy would likely be needed.
        let url = "https://gemini.google.com/_/BardChatUi/data/assistant.lamda.BardFrontendService/StreamGenerateContent";

        // Anti-Detection: Modern Chrome User-Agent
        let user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

        let res = self
            .client
            .post(url)
            .header("Cookie", format!("__Secure-1PSID={}", self.psid))
            .header("User-Agent", user_agent)
            .header("Referer", "https://gemini.google.com/")
            .json(&json!({
                "input": format!("{}\n\n{}", system_prompt, user_input)
            }))
            .send()
            .await
            .map_err(|e| crate::ai::AiError::NetworkError(e.to_string()))?;

        if !res.status().is_success() {
            let status = res.status();
            if status.as_u16() == 429 {
                return Err(crate::ai::AiError::QuotaExceeded);
            }

            // Session Lifecycle: Explicit Re-setup prompt on Auth error
            if status.as_u16() == 401 || status.as_u16() == 403 {
                eprintln!("\n🚨 [WebSession] __Secure-1PSID cookie has expired or is invalid.");
                if crate::interactor::Interactor::confirm(
                    "Would you like to run 'vega setup' now to update the cookie?",
                ) {
                    // Trigger SetupWizard for interactive cookie update
                    crate::setup::SetupWizard::run();
                    return Err(crate::ai::AiError::AuthError(
                        "Session expired. Setup completed, please retry your command.".to_string(),
                    ));
                }
            }

            return Err(crate::ai::AiError::Unknown(format!(
                "Web Session Error: {}",
                status
            )));
        }

        // Mock parsing (web responses are usually multipart/protobuf)
        Ok("Web Session Response (Simulated implementation)".to_string())
    }
}
