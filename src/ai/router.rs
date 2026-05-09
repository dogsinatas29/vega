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
    Ollama,
    #[allow(dead_code)]
    Mock,
}

impl SmartRouter {
    pub fn determine_engine(query: &str, preferred: Option<String>) -> EngineType {
        use colored::Colorize;

        // 1. Resolve Engine Preference
        let mut target_engine = None;

        if let Some(pref) = preferred {
            target_engine = Some(pref.to_lowercase());
        } else {
            // 💡 Milestone v0.0.14.4: Auto-Discovery from config.toml
            let config_path = crate::init::get_config_path();
            if let Ok(config) = crate::config::VegaConfig::load(config_path.to_str().unwrap()) {
                if let Some(ai) = config.ai {
                    target_engine = Some(ai.provider.to_lowercase());
                }
            }
        }

        if let Some(engine_str) = target_engine {
            match engine_str.as_str() {
                "gemini" => {
                    debug!("🎯 Router: Using Gemini");
                    return EngineType::Gemini;
                }
                "ollama" => {
                    debug!("🎯 Router: Using Local LLM (Ollama)");
                    return EngineType::Ollama;
                }
                "vertex_ai" | "vertexai" => {
                    debug!("🎯 Router: Using Vertex AI");
                    return EngineType::VertexAI;
                }
                "claude" => {
                    debug!("🎯 Router: Using Claude");
                    return EngineType::Claude;
                }
                "openai" | "gpt" => {
                    debug!("🎯 Router: Using OpenAI");
                    return EngineType::OpenAI;
                }
                "offline" => {
                    debug!("🎯 Router: Using Offline mode");
                    return EngineType::Offline;
                }
                "web" | "websession" => {
                    debug!("🎯 Router: Using Web Session");
                    return EngineType::WebSession;
                }
                _ => {
                    warn!("⚠️ Invalid engine '{}' in config, using intelligent routing.", engine_str);
                }
            }
        } else {
            // 2. No preference set - Warn user to run setup
            println!("{}", "⚠️  LLM Provider not configured. Run 'vega setup' to select an engine.".yellow());
            println!("   (Attempting fallback to default/offline engine...)");
        }

        // 3. Intelligent Routing (Fallback when no preference or invalid)
        let refined_query = query.to_lowercase();

        // Deep Analysis -> Gemini (as current high-performance default)
        if refined_query.contains("analyze")
            || refined_query.contains("debug")
            || refined_query.contains("why")
        {
            debug!("🧠 Router: Deep analysis detected. Routing to Gemini fallback.");
            return EngineType::Gemini;
        }

        // Default to Offline if no preference and not a complex query
        debug!("🛡️  Router: No preference and simple query. Selected: Offline");
        EngineType::Offline
    }

    pub fn get_provider(engine: EngineType) -> Result<Box<dyn AiProvider>, String> {
        info!("🤖 Initializing Provider: {:?}", engine);
        match engine {
            EngineType::Gemini => {
                match GeminiProvider::new() {
                    Ok(p) => Ok(Box::new(p)),
                    Err(e) => {
                        warn!(
                            "⚠️ Gemini Init Failed: {}. Checking for Web Session fallback...",
                            e
                        );
                        // Automatic Fallback Strategy: Cookie -> Offline
                        if crate::security::keyring::get_token("google_1psid").is_some() {
                            info!("♻️  Found '__Secure-1PSID' cookie. Routing to Web Session fallback.");
                            match crate::ai::providers::web_session::WebSessionProvider::new() {
                                Ok(p) => Ok(Box::new(p)),
                                Err(e2) => {
                                    warn!(
                                        "⚠️ Web Session fallback failed: {}. Using Offline engine.",
                                        e2
                                    );
                                    Ok(Box::new(OfflineEngine::new()))
                                }
                            }
                        } else {
                            warn!(
                                "ℹ️  No Web Session cookie found. Falling back to Offline engine."
                            );
                            Ok(Box::new(OfflineEngine::new()))
                        }
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
            EngineType::Ollama => {
                let config_path = crate::init::get_config_path();
                let config = crate::config::VegaConfig::load(config_path.to_str().unwrap())
                    .map_err(|e| format!("Failed to load config: {}", e))?;

                let ollama_config = config.ai
                    .and_then(|ai| ai.ollama)
                    .ok_or("Ollama not configured. Please run 'vega setup' and select Local LLM.")?;

                Ok(Box::new(crate::ai::providers::ollama::OllamaProvider::new(
                    ollama_config.endpoint,
                    ollama_config.model,
                )))
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

    pub async fn orchestrate_intent(
        ctx: &crate::context::SystemContext,
        query: &str,
        preferred: Option<String>,
    ) -> Result<crate::ai::AiResponse, crate::ai::AiError> {
        // 🔑 [Sovereign Step -1] - Extract Discovered Context
        let discovered_hosts: Vec<String> = ctx.remotes.iter()
            .filter(|r| r.r#type == crate::context::RemoteType::Host)
            .map(|r| r.real_name.clone())
            .collect();

        // 👑 [Sovereign Step 0] - Deterministic Pre-parsing with Discovery Context
        let sovereign_res = crate::ai::validator::SovereignParser::parse_sovereign(query, &discovered_hosts);
        if sovereign_res.confidence >= 1.0 {
            crate::ui::console::SreConsole::logic("Sovereign: Deterministic match. Skipping LLM.");
            return Ok(sovereign_res);
        }

        // Stage 1: Domain Classification
        let domain_prompt = crate::ai::prompts::PromptBuilder::domain_prompt(ctx);
        let domain_raw = Self::generate_with_fallback_raw(ctx, &domain_prompt, query, preferred.clone()).await?;
        
        let domain_res = crate::ai::DomainResponse::extract_json(&domain_raw)
            .ok_or_else(|| crate::ai::AiError::Unknown("Failed to parse Domain Classification".to_string()))?;
        
        crate::ui::console::SreConsole::telemetry(&format!("Stage 1: Domain: {:?}, Confidence: {:.2}", domain_res.domain, domain_res.confidence));

        if domain_res.domain == crate::ai::Domain::Unknown || domain_res.confidence < 0.6 {
            // Even if LLM fails, if Sovereign had some idea, try it
            if sovereign_res.action != "UNKNOWN" {
                crate::ui::console::SreConsole::logic("Recovery: LLM ambiguous, falling back to Sovereign partial parse.");
                return Ok(sovereign_res);
            }

            return Err(crate::ai::AiError::Unknown(format!(
                "Ambiguous intent detected (Domain: {:?}, Confidence: {:.2}). Action aborted for safety.",
                domain_res.domain, domain_res.confidence
            )));
        }

        // Stage 2: Action Parsing
        let action_prompt = crate::ai::prompts::PromptBuilder::action_prompt(domain_res.domain);
        let action_raw = match Self::generate_with_fallback_raw(ctx, &action_prompt, query, preferred).await {
            Ok(raw) => raw,
            Err(e) => {
                if sovereign_res.action != "UNKNOWN" {
                    crate::ui::console::SreConsole::logic("Recovery: LLM failed, using Sovereign results.");
                    return Ok(sovereign_res);
                }
                return Err(e);
            }
        };

        let llm_res = match crate::ai::AiResponse::extract_json(&action_raw) {
            Some(res) => res,
            None => {
                if sovereign_res.action != "UNKNOWN" {
                    crate::ui::console::SreConsole::logic("Recovery: LLM malformed JSON, using Sovereign results.");
                    return Ok(sovereign_res);
                }
                return Err(crate::ai::AiError::Unknown("Failed to parse Action Parsing".to_string()));
            }
        };

        // 👑 [Sovereign Step 3] - Authoritative Reconciliation (Ownership Inversion)
        let final_res = crate::ai::validator::SovereignParser::reconcile_to_intent(sovereign_res, llm_res, &discovered_hosts);
        crate::ui::console::SreConsole::telemetry(&format!("Stage 2: Action: {}, Confidence: {:.2}", final_res.action, final_res.confidence));

        Ok(final_res)
    }

    async fn generate_with_fallback_raw(
        ctx: &crate::context::SystemContext,
        system_prompt: &str,
        query: &str,
        preferred: Option<String>,
    ) -> Result<String, crate::ai::AiError> {
        let engine = Self::determine_engine(query, preferred);
        let provider = Self::get_provider(engine).map_err(|e| crate::ai::AiError::Unknown(e))?;
        
        provider.generate_response_with_system(ctx, system_prompt, query).await
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
        println!("📡 [Classifier] Provider: {:?}, Context: {} targets", engine, ctx.remotes.len());

        // Context Sync: Summary Injection logic
        let mut final_query = query.to_string();
        if engine == EngineType::WebSession && in_cooldown {
            if let Some(summary) = Self::get_context_summary(query, 3) {
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
                if let Some(summary) = Self::get_context_summary(query, 3) {
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

    fn get_context_summary(query: &str, limit: usize) -> Option<String> {
        // Local RAG: Search relevant history using SQLite FTS5
        if let Ok(db) = crate::storage::db::Database::new() {
            if let Ok(matches) = db.search_knowledge(query, limit) {
                if !matches.is_empty() {
                    let mut summary = String::from("\n[Relevant Context from History]:\n");
                    for (content, _) in matches {
                        summary.push_str(&format!("- {}\n", content));
                    }
                    return Some(summary);
                }
            }
        }

        // Fallback to simple tailing if DB search fails or returns nothing
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
