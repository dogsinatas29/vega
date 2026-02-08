use std::path::PathBuf;
use std::fs;
use crate::config::VegaConfig;
use crate::context::SystemContext;
use dirs;

pub fn bootstrap() -> Result<VegaConfig, Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    if !config_path.exists() {
        println!("🚀 VEGA: First Run Detected. Analyzing System Environment...");
        
        // 1. Fingerprint
        let ctx = SystemContext::collect();
        println!("   - Pkg Manager: {}", ctx.pkg_manager);
        println!("   - Virtualization: {}", if ctx.is_vm { "Yes" } else { "No" });
        println!("   - Git User: {}", ctx.git_user);
        
        // 2. Create Default Config
        let config = VegaConfig::default();
        // pre-populate based on scan?
        // For now just basic default + save it.
        
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let toml_str = toml::to_string(&config)?;
        fs::write(&config_path, toml_str)?;
        
        println!("✅ Config initialized at {:?}", config_path);
        return Ok(config);
    }

    VegaConfig::load(config_path.to_str().unwrap())
}

pub fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("vega");
    path.push("config.toml");
    path
}
