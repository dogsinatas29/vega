use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct VegaConfig {
    pub system: SystemConfig,
    pub execution: ExecutionConfig,
    pub optimization: Option<OptimizationConfig>,
    pub ai: Option<AiConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AiConfig {
    pub provider: String,       // gemini, chatgpt, claude, vertex_ai
    pub api_key_source: String, // env_var, manual
    pub model: Option<String>,
    pub vertex_ai: Option<VertexAiConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct VertexAiConfig {
    pub project_id: String,
    pub region: String, // e.g., us-central1
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct SystemConfig {
    pub debug_mode: Option<bool>,
    pub log_level: Option<String>,
    pub dependencies: Option<DependenciesConfig>,
    pub scan_exclude: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct DependenciesConfig {
    pub interesting_libs: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct ExecutionConfig {
    pub max_retries: Option<u32>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct OptimizationConfig {
    pub cache_enabled: Option<bool>,
    pub system_prompt_version: Option<String>,
    pub local_keywords: Option<Vec<String>>,
    pub shell_snapshot_path: Option<String>,
    pub auto_sync: Option<bool>,
    pub primary_remote: Option<String>,
}

impl VegaConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: VegaConfig = toml::from_str(&content)?;
        Ok(config)
    }

    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), String> {
        // Example validation: Check if API key is present (if we had that field)
        // For now, checks structural integrity
        if let Some(opt) = &self.optimization {
            if opt.cache_enabled.unwrap_or(false) && opt.system_prompt_version.is_none() {
                return Err("Optimization enabled but system_prompt_version missing".to_string());
            }
        }
        Ok(())
    }
}
