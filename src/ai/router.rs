use crate::ai::providers::gemini::GeminiProvider;
use crate::ai::providers::offline::OfflineEngine;
use crate::ai::providers::vertex_ai::VertexAiProvider;
use crate::ai::AiProvider;
use log::{debug, info, warn};

pub struct SmartRouter;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EngineType {
    Gemini,
    VertexAI,
    Claude,
    OpenAI,
    Offline,
    WebSession,
    #[allow(dead_code)]
    Mock,
}

impl SmartRouter {
    pub fn determine_engine(query: &str, preferred: Option<String>) -> EngineType {
        // 1. User Preference
        if let Some(pref) = preferred {
            match pref.to_lowercase().as_str() {
                "gemini" => {
                    debug!("🎯 Router: User forced Gemini");
                    return EngineType::Gemini;
                }
                "vertex_ai" | "vertexai" => {
                    debug!("🎯 Router: User forced Vertex AI");
                    return EngineType::VertexAI;
                }
                "claude" => {
                    debug!("🎯 Router: User forced Claude");
                    return EngineType::Claude;
                }
                "openai" | "gpt" => {
                    debug!("🎯 Router: User forced OpenAI");
                    return EngineType::OpenAI;
                }
                "offline" => {
                    debug!("🎯 Router: User forced Offline");
                    return EngineType::Offline;
                }
                "web" | "websession" => {
                    debug!("🎯 Router: User forced Web Session");
                    return EngineType::WebSession;
                }
                _ => {
                    warn!("⚠️ Invalid engine '{}', ignoring.", pref);
                }
            }
        }

        // 2. Context Analysis
        let refined_query = query.to_lowercase();

        // Deep Analysis -> Claude
        if refined_query.contains("analyze")
            || refined_query.contains("debug")
            || refined_query.contains("why")
        {
            // Mock fallback to Gemini for now but log intention
            debug!("🧠 Router: Deep analysis detected. Preferred: Claude (Falling back to Gemini mock)");
            return EngineType::Gemini;
        }

        // Long Context -> Gemini
        if query.len() > 1000 {
            debug!("📜 Router: Long context (>1000 chars). Selected: Gemini");
            return EngineType::Gemini;
        }

        // Simple Command -> Gemini/GPT
        debug!("⚡ Router: Default/Simple query. Selected: Gemini");
        EngineType::Gemini
    }

    pub fn get_provider(engine: EngineType) -> Result<Box<dyn AiProvider>, String> {
        info!("🤖 Initializing Provider: {:?}", engine);
        match engine {
            EngineType::Gemini => {
                match GeminiProvider::new() {
                    Ok(p) => Ok(Box::new(p)),
                    Err(e) => {
                        warn!("⚠️ Gemini Init Failed: {}. Falling back to Offline.", e);
                        // Automatic Fallback Strategy
                        Ok(Box::new(OfflineEngine::new()))
                    }
                }
            }
            EngineType::VertexAI => {
                // Load config to get project_id and region
                let config_path = crate::init::get_config_path();
                let config = crate::config::VegaConfig::load(config_path.to_str().unwrap())
                    .map_err(|e| format!("Failed to load config: {}", e))?;

                let vertex_config = config.ai
                    .and_then(|ai| ai.vertex_ai)
                    .ok_or("Vertex AI not configured. Please run 'vega setup' and configure project_id and region.")?;

                match VertexAiProvider::new(vertex_config.project_id, vertex_config.region) {
                    Ok(p) => Ok(Box::new(p)),
                    Err(e) => Err(format!("Vertex AI Init Failed: {}", e)),
                }
            }
            EngineType::Claude => Err("Claude Provider not yet implemented".to_string()),
            EngineType::OpenAI => Err("OpenAI Provider not yet implemented".to_string()),
            EngineType::Offline => Ok(Box::new(OfflineEngine::new())),
            EngineType::Mock => Ok(Box::new(crate::ai::providers::mock::MockProvider::new())),
            EngineType::WebSession => {
                match crate::ai::providers::web_session::WebSessionProvider::new() {
                    Ok(p) => Ok(Box::new(p)),
                    Err(e) => Err(format!("Web Session Init Failed: {}", e)),
                }
            }
        }
    }

    fn get_cache_path() -> std::path::PathBuf {
        let mut path = dirs::cache_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        path.push("vega");
        let _ = std::fs::create_dir_all(&path);
        path.push("quota_state.json");
        path
    }

    fn load_quota_state() -> u64 {
        let path = Self::get_cache_path();
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                return json["last_quota_error"].as_u64().unwrap_or(0);
            }
        }
        0
    }

    fn save_quota_state(timestamp: u64) {
        let path = Self::get_cache_path();
        let json = serde_json::json!({ "last_quota_error": timestamp });
        if let Ok(data) = serde_json::to_string(&json) {
            // Race Condition: Atomic write using tempfile + rename
            let mut temp_path = path.clone();
            temp_path.set_extension("tmp");
            if std::fs::write(&temp_path, data).is_ok() {
                let _ = std::fs::rename(temp_path, path);
            }
        }
    }

    pub async fn generate_with_fallback(
        ctx: &crate::context::SystemContext,
        query: &str,
        preferred: Option<String>,
    ) -> Result<String, crate::ai::AiError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let last_error_time = Self::load_quota_state();
        // Senior Tip: 1 hour (3600s) retry-after for better quota management
        let in_cooldown = last_error_time > 0 && (now - last_error_time) < 3600;

        let engine = if in_cooldown {
            println!("🔄 [Router] Persistent quota cooldown (1h). Using Web Session...");
            EngineType::WebSession
        } else {
            if last_error_time > 0 {
                println!("♻️  [Router] Quota reset window reached. Attempting primary API...");
            }
            Self::determine_engine(query, preferred)
        };

        let provider = Self::get_provider(engine).map_err(|e| crate::ai::AiError::Unknown(e))?;
        println!("⚡ [Router] Routing to: {:?}", engine);

        // Context Sync: Summary Injection logic
        let mut final_query = query.to_string();
        if engine == EngineType::WebSession && in_cooldown {
            if let Some(summary) = Self::get_context_summary(3) {
                println!("🧠 [Router] Injecting context summary from API -> Web Session sync...");
                final_query = format!(
                    "Previous context summary: [ {} ]\n\nContinuing with: {}",
                    summary, query
                );
            }
        }

        match provider.generate_response(ctx, &final_query).await {
            Ok(res) => {
                if !in_cooldown && last_error_time > 0 {
                    Self::save_quota_state(0);
                }
                Ok(res)
            }
            Err(crate::ai::AiError::QuotaExceeded) => {
                if engine == EngineType::WebSession {
                    return Err(crate::ai::AiError::QuotaExceeded);
                }

                eprintln!("🚨 [Quota Exceeded] API limit reached. Persistence enabled for 1 hour.");
                Self::save_quota_state(now);

                let web_provider = Self::get_provider(EngineType::WebSession)
                    .map_err(|e| crate::ai::AiError::Unknown(e))?;

                // When falling back, also try to inject summary
                let mut fallback_query = query.to_string();
                if let Some(summary) = Self::get_context_summary(3) {
                    fallback_query = format!(
                        "Previous context summary: [ {} ]\n\nContinuing with: {}",
                        summary, query
                    );
                }

                web_provider.generate_response(ctx, &fallback_query).await
            }
            Err(e) => Err(e),
        }
    }

    fn get_context_summary(limit: usize) -> Option<String> {
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
            let mut recent = Vec::new();
            for line in reader.lines().flatten() {
                if let Ok(entry) = serde_json::from_str::<serde_json::Value>(&line) {
                    let cmd = entry["command"].as_str().unwrap_or("");
                    recent.push(cmd.to_string());
                }
            }

            if recent.is_empty() {
                return None;
            }

            let take = if recent.len() > limit {
                limit
            } else {
                recent.len()
            };
            let summary = recent[recent.len() - take..].join(" -> ");
            return Some(summary);
        }
        None
    }
}
