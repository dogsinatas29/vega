use crate::executor::pipeline::{Intent, IntentResolver};
use async_trait::async_trait;
use regex::Regex;

pub struct LocalIntentResolver;

#[async_trait]
impl IntentResolver for LocalIntentResolver {
    async fn resolve(&self, input: &str) -> anyhow::Result<Intent> {
        let input_lower = input.to_lowercase();

        // SSH / Connect
        if input_lower.contains("connect") || input_lower.contains("ssh") {
            let re = Regex::new(r"(?:connect to |ssh to |ssh )(\S+)").unwrap();
            let target = re.captures(&input_lower).map(|c| c[1].to_string()).unwrap_or_default();
            return Ok(Intent {
                action: "SSH_CONNECT".to_string(),
                target: "localhost".to_string(),
                params: serde_json::json!({ "host": target }),
                thought: "Local pattern match for SSH".to_string(),
                confidence: 1.0,
            });
        }

        // Ollama Logic
        if input_lower.contains("ollama") {
            if input_lower.contains("목록") || input_lower.contains("list") {
                return Ok(Intent {
                    action: "OLLAMA_LIST_INSTALLED".to_string(),
                    target: "localhost".to_string(),
                    params: serde_json::json!({}),
                    thought: "Local pattern match for Ollama List".to_string(),
                    confidence: 1.0,
                });
            }
        }

        // Update
        if input_lower.contains("update") || input_lower.contains("upgrade") {
             return Ok(Intent {
                 action: "SYSTEM_UPDATE".to_string(),
                 target: "localhost".to_string(),
                 params: serde_json::json!({}),
                 thought: "Local pattern match for Update".to_string(),
                 confidence: 1.0,
             });
        }

        Ok(Intent::unknown())
    }
}

pub struct AiIntentResolver;

pub struct HybridIntentResolver {
    pub local: LocalIntentResolver,
    pub ai: AiIntentResolver,
}

#[async_trait]
impl IntentResolver for HybridIntentResolver {
    async fn resolve(&self, input: &str) -> anyhow::Result<Intent> {
        if let Ok(intent) = self.local.resolve(input).await {
            if !intent.is_unknown() {
                return Ok(intent);
            }
        }
        self.ai.resolve(input).await
    }
}

#[async_trait]
impl IntentResolver for AiIntentResolver {
    async fn resolve(&self, input: &str) -> anyhow::Result<Intent> {
        let ctx = crate::context::SystemContext::collect(true);
        let query = format!(
            "TASK: Resolve structured operational intent from natural language.
             INPUT: \"{}\"
             
             STRICT SCHEMA (RESPOND ONLY IN THIS JSON):
             {{
               \"thought\": \"Reasoning in Korean\",
               \"action\": \"OLLAMA_LIST_INSTALLED\" | \"OLLAMA_PULL\" | \"OLLAMA_REMOVE\" | \"INSTALL_APT\" | \"CONNECT\" | \"UPDATE\" | \"SYSTEM_DIAGNOSTIC\" | \"SYSTEM_SHUTDOWN\",
               \"target\": \"localhost\" | \"specific IP/Hostname from World State\",
               \"params\": {{ \"model\": \"...\", \"name\": \"...\", \"force\": false, \"host\": \"...\" }},
               \"confidence\": 0.0 to 1.0
             }}",
            input
        );

        match crate::ai::router::SmartRouter::generate_with_fallback(&ctx, &query, None).await {
            Ok(res) => {
                let res_trimmed = res.trim();
                // Extract JSON if wrapped in markdown
                let cleaned = if let Some(start) = res_trimmed.find('{') {
                    if let Some(end) = res_trimmed.rfind('}') {
                        &res_trimmed[start..=end]
                    } else { res_trimmed }
                } else { res_trimmed };

                if let Ok(intent) = serde_json::from_str::<Intent>(cleaned) {
                    Ok(intent)
                } else {
                    anyhow::bail!("AI returned malformed intent JSON: {}", cleaned)
                }
            },
            Err(e) => anyhow::bail!("AI Intent Resolution failed: {}", e),
        }
    }
}
