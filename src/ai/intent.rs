use crate::executor::pipeline::{Intent, IntentResolver};
use async_trait::async_trait;
use regex::Regex;

pub struct LocalIntentResolver;

#[async_trait]
impl IntentResolver for LocalIntentResolver {
    async fn resolve(&self, input: &str) -> anyhow::Result<Intent> {
        let input_lower = input.to_lowercase();

        // Backup
        if input_lower.contains("backup") {
            let re = Regex::new(r"backup (?:of |server )?(\S+)").unwrap();
            let target = re.captures(&input_lower).map(|c| c[1].to_string());
            return Ok(Intent {
                tool: "rclone".to_string(),
                operation: "sync".to_string(),
                target,
            });
        }

        // SSH / Connect
        if input_lower.contains("connect") || input_lower.contains("ssh") {
            let re = Regex::new(r"(?:connect to |ssh to |ssh )(\S+)").unwrap();
            let target = re.captures(&input_lower).map(|c| c[1].to_string());
            return Ok(Intent {
                tool: "ssh".to_string(),
                operation: "connect".to_string(),
                target,
            });
        }

        // Install
        if input_lower.contains("install") {
            let re = Regex::new(r"install (\S+)").unwrap();
            let target = re.captures(&input_lower).map(|c| c[1].to_string());
            return Ok(Intent {
                tool: "pkg".to_string(),
                operation: "install".to_string(),
                target,
            });
        }

        // Update
        if input_lower.contains("update") || input_lower.contains("upgrade") {
             return Ok(Intent {
                tool: "pkg".to_string(),
                operation: "update".to_string(),
                target: None,
            });
        }

        anyhow::bail!("Unknown intent")
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
            return Ok(intent);
        }
        self.ai.resolve(input).await
    }
}

#[async_trait]
impl IntentResolver for AiIntentResolver {
    async fn resolve(&self, input: &str) -> anyhow::Result<Intent> {
        let ctx = crate::context::SystemContext::collect();
        let query = format!(
            "TASK: Resolve structured operational intent from natural language.
             INPUT: \"{}\"
             
             RULES:
             1. Return ONLY a JSON object with fields: tool, operation, target.
             2. Do not explain.
             Example: {{\"tool\": \"rclone\", \"operation\": \"sync\", \"target\": \"server_a\"}}",
            input
        );

        match crate::ai::router::SmartRouter::generate_with_fallback(&ctx, &query, None).await {
            Ok(res) => {
                let res_trimmed = res.trim();
                // Attempt to parse JSON
                if let Ok(intent) = serde_json::from_str::<Intent>(res_trimmed) {
                    Ok(intent)
                } else {
                    // Fallback regex if AI results are slightly malformed
                    anyhow::bail!("AI returned malformed intent: {}", res_trimmed)
                }
            },
            Err(e) => anyhow::bail!("AI Intent Resolution failed: {}", e),
        }
    }
}
