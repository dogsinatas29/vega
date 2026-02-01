use crate::system::config::{AppConfig, ProviderConfig};
use crate::system::os::OsInfo;
use crate::system::env_scanner::EnvScanner;
use colored::*;
use std::env;
use std::io::{self, Write};

pub fn initialize_system() -> AppConfig {
    // 1. Try to load existing config
    if let Some(config) = AppConfig::load() {
        // Check quota (Day check logic would go here, simplified for now)
        if config.providers.usage_today > config.providers.daily_quota {
             println!("{}", "⚠️  DAILY QUOTA EXCEEDED (Warning)".yellow().bold());
        }
        return config;
    }

    // 2. First Run Initialization
    println!("{}", "🚀 Initializing VEGA System Profile...".green().bold());
    
    // A. Detect OS
    print!("   🔍 Fingerprinting System... ");
    io::stdout().flush().unwrap();
    let os_info = OsInfo::detect();
    println!("✅ {} {} ({}) - {:?}", os_info.id, os_info.version, os_info.arch, os_info.pkg_manager);

    // B. Credential Discovery (The Detective)
    println!("   🕵️  Scanning for Credentials...");
    
    // Check Active Env First
    let env_gemini = env::var("GEMINI_API_KEY").ok();
    let env_anthropic = env::var("ANTHROPIC_API_KEY").ok();
    let env_openai = env::var("OPENAI_API_KEY").ok(); // Added for OpenAI

    let mut detected_gemini = env_gemini.is_some();
    let mut detected_anthropic = env_anthropic.is_some();
    let mut detected_openai = env_openai.is_some(); // Added for OpenAI
    
    // If not in Env, scan RC files
    let rc_keys = EnvScanner::scan_shell_configs();
    if !detected_gemini {
        if let Some(k) = rc_keys.get("GEMINI_API_KEY") {
             println!("      ✅ Found GEMINI_API_KEY in {:?} (Not active in current shell)", k.source_file);
             detected_gemini = true;
        }
    } else {
        println!("      ✅ GEMINI_API_KEY is active in environment.");
    }
    
    if !detected_anthropic {
        if let Some(k) = rc_keys.get("ANTHROPIC_API_KEY") {
             println!("      ✅ Found ANTHROPIC_API_KEY in {:?} (Not active in current shell)", k.source_file);
             detected_anthropic = true;
        }
    } else {
        println!("      ✅ ANTHROPIC_API_KEY is active in environment.");
    }

    // Added for OpenAI
    if !detected_openai {
        if let Some(k) = rc_keys.get("OPENAI_API_KEY") {
             println!("      ✅ Found OPENAI_API_KEY in {:?} (Not active in current shell)", k.source_file);
             detected_openai = true;
        }
    } else {
        println!("      ✅ OPENAI_API_KEY is active in environment.");
    }

    // C. Dry Run Confirmation
    println!("\n{}", "📋 Initialization Summary:".bold());
    println!("   - System: {} {} ({})", os_info.id, os_info.version, os_info.arch);
    println!("   - Pkg Manager: {:?}", os_info.pkg_manager);
    println!("   - Config Path: {:?}", AppConfig::get_config_path());
    println!("   - Gemini Key: {}", if detected_gemini { "Detected".green() } else { "Missing".red() });
    println!("   - Anthropic Key: {}", if detected_anthropic { "Detected".green() } else { "Missing".red() }); // Added
    println!("   - OpenAI Key: {}", if detected_openai { "Detected".green() } else { "Missing".red() }); // Added

    // D. Create Config
    let config = AppConfig {
        os_info,
        providers: ProviderConfig {
            default_engine: "gemini".to_string(),
            daily_quota: 100000,
            usage_today: 0,
            api_key_source: "Environment".to_string(),
        },
        projects: Vec::new(),
    };

    // E. Save
    config.save();
    println!("   💾 Configuration saved to config.toml");

    config
}


