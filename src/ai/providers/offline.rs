use crate::ai::{AiProvider, QuotaStatus};
use crate::context::SystemContext;
use crate::system::sanitizer::sanitize_string;
use async_trait::async_trait;
use serde_json::json;

pub struct OfflineEngine;

impl OfflineEngine {
    pub fn new() -> Self {
        Self {}
    }

    fn map_query(&self, query: &str) -> Option<String> {
        let q = query.to_lowercase();

        // Simple Rule-based Mapping
        if q.contains("disk") || q.contains("space") {
            return Some("df -h".to_string());
        }
        if q.contains("ip") || q.contains("network") {
            return Some("ip a".to_string());
        }
        if q.contains("list") || q.contains("files") {
            return Some("ls -lah".to_string());
        }
        if q.contains("memory") || q.contains("ram") {
            return Some("free -h".to_string());
        }
        if q.contains("process") || q.contains("top") {
            return Some("ps aux | head -n 10".to_string());
        }

        None
    }
}

#[async_trait]
impl AiProvider for OfflineEngine {
    fn name(&self) -> &str {
        "Vega Offline (Rule-based)"
    }

    fn get_quota_status(&self) -> QuotaStatus {
        QuotaStatus::Unlimited
    }

    async fn generate_response(
        &self,
        _ctx: &SystemContext,
        user_input: &str,
    ) -> Result<String, crate::ai::AiError> {
        let mut command = "ls -la".to_string();
        let mut explanation =
            "I am in offline mode. I couldn't find a specific rule, so here is a default command."
                .to_string();
        let mut risk_level = "INFO";

        // 1. Try Rule-based Mapping
        if let Some(cmd) = self.map_query(user_input) {
            command = cmd;
            explanation = format!("Offline mode: Found a local rule matching your request.");
        } else {
            // 2. Try History-based Suggestion
            if let Some(history_cmd) = self.search_history(user_input) {
                command = history_cmd;
                explanation =
                    format!("Offline mode: I found a similar command you've executed before.");
                risk_level = "WARNING"; // History might be outdated, warn user
            }
        }

        let res = json!({
            "command": command,
            "explanation": explanation,
            "risk_level": risk_level,
            "needs_clarification": false
        });

        Ok(serde_json::to_string(&res).map_err(|e| crate::ai::AiError::Unknown(e.to_string()))?)
    }
}

impl OfflineEngine {
    fn search_history(&self, query: &str) -> Option<String> {
        let history_path = dirs::data_local_dir()
            .map(|mut p| {
                p.push("vega");
                p.push("history.jsonl");
                p
            })
            .unwrap_or_else(|| std::path::PathBuf::from("logs/history.jsonl"));

        if let Ok(file) = std::fs::File::open(history_path) {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(file);
            let q = query.to_lowercase();

            // Search backwards for the most recent similar command
            let mut matches = Vec::new();
            for line in reader.lines().flatten() {
                if let Ok(entry) = serde_json::from_str::<serde_json::Value>(&line) {
                    let cmd = entry["command"].as_str().unwrap_or("");
                    if cmd.to_lowercase().contains(&q) {
                        // Senior Tip: Sanitize history before suggesting to protect privacy
                        matches.push(sanitize_string(cmd));
                    }
                }
            }
            return matches.last().cloned();
        }
        None
    }
}
