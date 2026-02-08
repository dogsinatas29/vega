use crate::ai::AiProvider;
use crate::ai::providers::gemini::GeminiProvider;
use crate::ai::providers::vertex_ai::VertexAiProvider;
use crate::ai::providers::offline::OfflineEngine;
use log::{info, warn, debug};

pub struct SmartRouter;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EngineType {
    Gemini,
    VertexAI,
    Claude,
    OpenAI,
    Offline,
    #[allow(dead_code)]
    Mock,
}

impl SmartRouter {
    pub fn determine_engine(query: &str, preferred: Option<String>) -> EngineType {
        // 1. User Preference
        if let Some(pref) = preferred {
            match pref.to_lowercase().as_str() {
                "gemini" => { debug!("🎯 Router: User forced Gemini"); return EngineType::Gemini; },
                "vertex_ai" | "vertexai" => { debug!("🎯 Router: User forced Vertex AI"); return EngineType::VertexAI; },
                "claude" => { debug!("🎯 Router: User forced Claude"); return EngineType::Claude; },
                "openai" | "gpt" => { debug!("🎯 Router: User forced OpenAI"); return EngineType::OpenAI; },
                "offline" => { debug!("🎯 Router: User forced Offline"); return EngineType::Offline; },
                _ => { warn!("⚠️ Invalid engine '{}', ignoring.", pref); }
            }
        }

        // 2. Context Analysis
        let refined_query = query.to_lowercase();
        
        // Deep Analysis -> Claude
        if refined_query.contains("analyze") || refined_query.contains("debug") || refined_query.contains("why") {
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
            },
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
                    Err(e) => Err(format!("Vertex AI Init Failed: {}", e))
                }
            },
            EngineType::Claude => Err("Claude Provider not yet implemented".to_string()), 
            EngineType::OpenAI => Err("OpenAI Provider not yet implemented".to_string()),
            EngineType::Offline => Ok(Box::new(OfflineEngine::new())),
            EngineType::Mock => Ok(Box::new(crate::ai::providers::mock::MockProvider::new())),
        }
    }
}
