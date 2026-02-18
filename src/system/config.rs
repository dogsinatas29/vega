use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub default_engine: String,
    // Keys are NOT stored here. They must be provided via usage of Environment Variables.
    // We only track quotas/preferences here.
    pub daily_quota: u32,
    pub usage_today: u32,
    pub api_key_source: String, // "Environment"
}

impl Default for ProviderConfig {
    fn default() -> Self {
        ProviderConfig {
            default_engine: "gemini".to_string(),
            daily_quota: 150000,
            usage_today: 0,
            api_key_source: "Environment".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub description: String,
    pub git_check: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub providers: ProviderConfig,
    #[serde(default)]
    pub projects: Vec<Project>,
}

impl AppConfig {
    pub fn get_config_path() -> PathBuf {
        // 1. Try XDG Config Home (~/.config/vega/config.toml)
        if let Ok(home) = env::var("HOME") {
            let xdg_path = Path::new(&home)
                .join(".config")
                .join("vega")
                .join("config.toml");
            // If parent dir exists, use it. Or if we are creating it later.
            return xdg_path;
        }
        // Fallback
        PathBuf::from("config.toml")
    }

    pub fn load() -> Option<Self> {
        // Strategy: Check CWD first for dev convenience, then XDG.
        // Senior advice: "Dev in CWD, but verify XDG".
        // Let's check XDG first as the "Real" path, then CWD as fallback/override?
        // User said: "CWD is convenient for test, but eventually go to XDG."
        // "Load() should check global path first with fallback" -> implied precedence?
        // Let's check CWD. IF exists, use it. ELSE check XDG.

        let cwd_path = PathBuf::from("config.toml");
        if cwd_path.exists() {
            return Self::load_from_path(&cwd_path);
        }

        let xdg_path = Self::get_config_path();
        if xdg_path.exists() {
            return Self::load_from_path(&xdg_path);
        }

        None
    }

    fn load_from_path(path: &Path) -> Option<Self> {
        match fs::read_to_string(path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => return Some(config),
                Err(e) => eprintln!("⚠️ Failed to parse {:?}: {}", path, e),
            },
            Err(e) => eprintln!("⚠️ Failed to read {:?}: {}", path, e),
        }
        None
    }

    pub fn save(&self) {
        let path = Self::get_config_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                let _ = fs::create_dir_all(parent);
            }
        }

        // Add comments manually since TOML crate doesn't support comment preservation well
        let toml_str = toml::to_string_pretty(self).unwrap_or_default();

        let commented_config = format!(
            r#"# 🌌 VEGA Configuration File
# ⚠️  DO NOT EDIT MANUALLY unless you know what you are doing.
# 🔑 API Keys are NOT stored here. They are read from your Environment Variables (e.g. GEMINI_API_KEY).

{}"#,
            toml_str
        );

        if let Err(e) = fs::write(&path, commented_config) {
            eprintln!("⚠️ Failed to write config to {:?}: {}", path, e);
        } else {
            println!("💾 Config saved to {:?}", path);
        }
    }
}
