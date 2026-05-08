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
            return Ok(Intent::BackupData {
                source: ".".to_string(),
                target: "remote".to_string(),
            });
        }

        // SSH / Connect
        if input_lower.contains("connect") || input_lower.contains("ssh") {
            let re = Regex::new(r"(?:connect to |ssh to |ssh )(\S+)").unwrap();
            let target = re.captures(&input_lower).map(|c| c[1].to_string()).unwrap_or_default();
            return Ok(Intent::SshConnect { host: target });
        }

        // Ollama Logic
        if input_lower.contains("ollama") {
            let install_verbs = vec!["설치", "받아", "다운로드", "추가", "pull", "install", "download", "add"];
            let remove_verbs = vec!["삭제", "지워", "제거", "uninstall", "remove", "delete", "rm"];
            
            if install_verbs.iter().any(|&v| input_lower.contains(v)) {
                let models = vec!["mistral", "llama", "phi", "gemma", "qwen"];
                for model in models {
                    if input_lower.contains(model) {
                        return Ok(Intent::OllamaPull { model: model.to_string() });
                    }
                }
            }

            if remove_verbs.iter().any(|&v| input_lower.contains(v)) {
                let re = Regex::new(r"(?:rm|remove|삭제|제거) (\S+)").unwrap();
                let target = re.captures(&input_lower).map(|c| c[1].to_string()).unwrap_or_default();
                if !target.is_empty() && target != "ollama" {
                    return Ok(Intent::OllamaRemove { model: target, force: input_lower.contains("force") });
                }
            }

            if input_lower.contains("목록") || input_lower.contains("list") {
                return Ok(Intent::OllamaListInstalled {});
            }
        }

        // Update
        if input_lower.contains("update") || input_lower.contains("upgrade") {
             return Ok(Intent::SystemUpdate {});
        }

        Ok(Intent::Unknown)
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
            if !matches!(intent, Intent::Unknown) {
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
             
             SCHEMA:
             {{
               \"action\": \"OLLAMA_LIST_INSTALLED\" | \"OLLAMA_PULL\" | \"OLLAMA_REMOVE\" | \"INSTALL_APT\" | \"CONNECT\" | \"UPDATE\",
               \"params\": {{ \"model\": \"...\", \"name\": \"...\", \"force\": false }}
             }}",
            input
        );

        match crate::ai::router::SmartRouter::generate_with_fallback(&ctx, &query, None).await {
            Ok(res) => {
                let res_trimmed = res.trim();
                if let Ok(intent) = serde_json::from_str::<Intent>(res_trimmed) {
                    Ok(intent)
                } else {
                    anyhow::bail!("AI returned malformed intent JSON: {}", res_trimmed)
                }
            },
            Err(e) => anyhow::bail!("AI Intent Resolution failed: {}", e),
        }
    }
}
