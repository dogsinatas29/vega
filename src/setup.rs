use crate::config::{AiConfig, OptimizationConfig, VegaConfig};
use crate::context::SystemContext;
use std::env;
use std::fs;
use std::io::{self, Write};

pub struct SetupWizard;

impl SetupWizard {
    pub fn run() {
        // 0. Check for Silent Mode (--yes / -y)
        let args: Vec<String> = env::args().collect();
        let silent_mode = args.contains(&"--yes".to_string()) || args.contains(&"-y".to_string());

        if silent_mode {
            println!("🚀 Vega Setup (Silent Mode)");
        } else {
            println!("🚀 Vega Setup Wizard");
            println!("--------------------");
        }

        // 1. System Scan
        if !silent_mode {
            println!("🔍 Scanning System...");
            let context = SystemContext::collect();
            println!("🏠 OS: {}", context.os_name);
        }

        // 2. AI Onboarding
        if !silent_mode {
            println!("\n[1] Setup Intelligence (LLM)");
        }

        let provider = if silent_mode {
            "gemini".to_string() // Default for silent
        } else {
            Self::select_provider()
        };

        let (_api_key, source) = if silent_mode {
            // In silent mode, try to find key, if not, fail or use dummy?
            // Design decision: Try to find env var first.
            let env_key = match provider.as_str() {
                "gemini" => env::var("GEMINI_API_KEY"),
                "chatgpt" => env::var("OPENAI_API_KEY"),
                "claude" => env::var("ANTHROPIC_API_KEY"),
                _ => Err(std::env::VarError::NotPresent),
            };

            if let Ok(_) = env_key {
                ("***MASKED***".to_string(), "env_var".to_string())
            } else {
                // Try file scan
                if let Some((_, path)) = crate::system::env_scanner::find_key(&provider) {
                    ("***MASKED***".to_string(), format!("file:{}", path))
                } else {
                    println!(
                        "❌ Silent Mode Error: No API Key found for {}. Please set env var.",
                        provider
                    );
                    return;
                }
            }
        } else {
            Self::discover_and_confirm_key(&provider)
        };

        if !silent_mode {
            println!("   - Provider: {}", provider);
            println!("   - API Key Source: {}", source);
        }

        // 3. Generate Config
        let mut config = VegaConfig::default();
        config.system.log_level = Some("INFO".to_string());

        config.ai = Some(AiConfig {
            provider: provider.clone(),
            api_key_source: source,
            model: None,
            vertex_ai: None,
        });

        config.optimization = Some(OptimizationConfig {
            cache_enabled: Some(true),
            system_prompt_version: Some("1.0".to_string()),
            local_keywords: Some(vec!["update".to_string(), "ssh".to_string()]),
            shell_snapshot_path: None, // Use default logic in main.rs
        });

        // Save
        let config_path = crate::init::get_config_path();

        // Ensure parent dir exists
        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let toml_str = toml::to_string(&config).unwrap();
        fs::write(&config_path, toml_str).expect("Failed to write config");

        println!(
            "✨ Setup Complete. Configuration saved to {:?}",
            config_path
        );
    }

    pub fn setup_cookie() {
        println!("🍪 Gemini Web Session Setup (Hardened)");
        println!("---------------------------");
        println!("To bypass API quotas, VEGA can use your browser session.");
        println!("⚠️  Note: Ensure your terminal IP matches your browser's IP.");
        println!("\n1. Open https://gemini.google.com in Chrome/Firefox.");
        println!("2. F12 -> Application/Storage -> Cookies.");
        println!("3. Copy the values for the following keys:");

        // 1. __Secure-1PSID
        let psid = Self::prompt("\n🔑 [Required] Enter __Secure-1PSID: ", None);
        if psid.is_empty() || psid.len() < 20 {
            println!("❌ Invalid cookie format.");
            return;
        }

        // 2. __Secure-1PAPISID
        let papisid = Self::prompt(
            "🔑 [Recommended] Enter __Secure-1PAPISID (Enter to skip): ",
            None,
        );

        // 3. User-Agent
        println!("\n🌐 To avoid session hijacking protection, we need your Browser's User-Agent.");
        println!("   Tip: Type 'my user agent' in Google or check DevTools -> Console -> navigator.userAgent");
        let ua = Self::prompt("🖥️  Enter User-Agent: ", Some("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));

        // Save tokens
        let mut results = vec![];
        results.push(crate::security::keyring::set_token("google_1psid", &psid));

        if !papisid.is_empty() {
            results.push(crate::security::keyring::set_token(
                "google_1papisid",
                &papisid,
            ));
        }

        results.push(crate::security::keyring::set_token("google_ua", &ua));

        if results.iter().all(|r| r.is_ok()) {
            println!("\n✅ All session tokens saved securely to keyring.");
            println!("   VEGA will now use these for fallback routing.");
        } else {
            println!("\n❌ Some tokens failed to save. Check system keyring state.");
        }
    }

    fn select_provider() -> String {
        println!("   Select LLM Provider:");
        println!("   1) Gemini (Recommended)");
        println!("   2) ChatGPT");
        println!("   3) Claude");

        loop {
            let choice = Self::prompt("   Select (1-3): ", Some("1"));
            match choice.as_str() {
                "1" | "gemini" => return "gemini".to_string(),
                "2" | "chatgpt" => return "chatgpt".to_string(),
                "3" | "claude" => return "claude".to_string(),
                _ => println!("   ❌ Invalid choice."),
            }
        }
    }

    fn discover_and_confirm_key(provider: &str) -> (String, String) {
        let env_var = match provider {
            "gemini" => "GEMINI_API_KEY",
            "chatgpt" => "OPENAI_API_KEY",
            "claude" => "ANTHROPIC_API_KEY",
            _ => "UNKNOWN_KEY",
        };

        // 1. Check current env
        if env::var(env_var).is_ok() {
            println!("   🔍 Detected API Key in environment ({})", env_var);
            if Self::confirm("   Use this key? (Y/n): ", Some("Y")) {
                return ("***MASKED***".to_string(), "env_var".to_string());
            }
        }

        // 2. Advanced Regex Scan
        if let Some((masked_key, path)) = crate::system::env_scanner::find_key(provider) {
            println!("   🔍 Detected API Key in {} ({})", path, masked_key);
            if Self::confirm(&format!("   Use key from {}? (Y/n): ", path), Some("Y")) {
                return (masked_key, format!("file:{}", path));
            }
        }

        // 3. Manual Fallback
        loop {
            let key = Self::prompt("   🔑 Enter API Key: ", None);
            if !key.is_empty() {
                // Basic Validation
                if key.len() < 10 {
                    println!("   ❌ Invalid Key Format (Too short).");
                    continue;
                }
                return (key, "manual".to_string());
            }
            println!("   ❌ API Key cannot be empty.");
        }
    }

    fn prompt(msg: &str, default: Option<&str>) -> String {
        print!("{}", msg);
        if let Some(d) = default {
            print!("(default: {}) ", d);
        }
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim();

        if trimmed.is_empty() && default.is_some() {
            default.unwrap().to_string()
        } else {
            trimmed.to_string()
        }
    }

    fn confirm(msg: &str, default: Option<&str>) -> bool {
        let input = Self::prompt(msg, default);
        input.eq_ignore_ascii_case("y") || input.eq_ignore_ascii_case("yes")
    }
}
