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
    papisid: Option<String>,
    user_agent: String,
}

impl WebSessionProvider {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let psid = keyring::get_token("google_1psid")
            .ok_or("Web Session requires __Secure-1PSID cookie. Please save it using 'vega setup --cookie'.")?;

        let papisid = keyring::get_token("google_1papisid");
        let user_agent = keyring::get_token("google_ua")
            .unwrap_or_else(|| "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string());

        Ok(Self {
            client: Client::new(),
            psid,
            papisid,
            user_agent,
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
        let url = "https://gemini.google.com/app/_/BardChatUi/data/assistant.lamda.BardFrontendService/StreamGenerateContent";

        // Multi-Cookie Header Construction
        let mut cookie_str = format!("__Secure-1PSID={}", self.psid);
        if let Some(p) = &self.papisid {
            cookie_str.push_str(&format!("; __Secure-1PAPISID={}", p));
        }

        let res = self
            .client
            .post(url)
            .header("Cookie", cookie_str)
            .header("User-Agent", &self.user_agent)
            .header("Referer", "https://gemini.google.com/")
            .json(&json!({
                "input": format!("{}\n\n{}", system_prompt, user_input)
            }))
            .send()
            .await
            .map_err(|e| crate::ai::AiError::NetworkError(e.to_string()))?;

        if !res.status().is_success() {
            let status = res.status();
            let error_text = res
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Senior Diagnostic: Dump Raw Response on error if DEBUG is enabled
            if std::env::var("VEGA_DEBUG").is_ok() {
                let _ = std::fs::create_dir_all("logs");
                let _ = std::fs::write("logs/web_raw.html", &error_text);
                println!("🔍 [DEBUG] Raw response dumped to logs/web_raw.html");
            }

            if status.as_u16() == 429 {
                return Err(crate::ai::AiError::QuotaExceeded);
            }

            // Session Lifecycle: Explicit Re-setup prompt on Auth error
            if status.as_u16() == 401 || status.as_u16() == 403 {
                eprintln!("\n🚨 [WebSession] Session has expired or is invalid.");
                if crate::interactor::Interactor::confirm(
                    "Would you like to run 'vega setup --cookie' now to update the session?",
                ) {
                    crate::setup::SetupWizard::setup_cookie();
                    return Err(crate::ai::AiError::AuthError(
                        "Session expired. Setup completed, please retry your command.".to_string(),
                    ));
                }
            }

            let msg = if status.as_u16() == 404 {
                format!("404 Not Found. The internal Gemini endpoint may have changed. Run with 'VEGA_DEBUG=1' and check 'logs/web_raw.html' for clues.")
            } else {
                format!(
                    "HTTP {}: {}",
                    status,
                    error_text.chars().take(100).collect::<String>()
                )
            };

            return Err(crate::ai::AiError::Unknown(msg));
        }

        let text = res
            .text()
            .await
            .map_err(|e| crate::ai::AiError::NetworkError(e.to_string()))?;

        // Senior Diagnostic: Log Raw Response if DEBUG is enabled
        if std::env::var("VEGA_DEBUG").is_ok() {
            let _ = std::fs::create_dir_all("logs");
            let _ = std::fs::write("logs/web_raw.html", &text);
            println!("🔍 [DEBUG] Raw response dumped to logs/web_raw.html");
        }

        // Try to parse the text as JSON
        if let Ok(res_json) = serde_json::from_str::<serde_json::Value>(&text) {
            // ... existing parsing logic if needed
            // For now, it's still a simulation return:
            Ok(serde_json::to_string(&res_json).unwrap_or_else(|_| "{}".to_string()))
        } else {
            // If it's not JSON (likely HTML), return informative error
            Err(crate::ai::AiError::Unknown("Received non-JSON response from Gemini website. Check logs/web_raw.html for details.".to_string()))
        }
    }
}
