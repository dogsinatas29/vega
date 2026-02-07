use regex::Regex;
use std::fs;
use std::env;

/// Scans standard shell configuration files for a specific API Key pattern.
/// Returns the first match found, MASKED for security.
/// Returns Tuple: (MaskedKey, SourcePath)
pub fn find_key(provider: &str) -> Option<(String, String)> {
    let target_var = match provider {
        "gemini" => "GEMINI_API_KEY",
        "chatgpt" => "OPENAI_API_KEY",
        "claude" => "ANTHROPIC_API_KEY",
        _ => return None,
    };

    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    // Shell config files to scan (zsh, bash, fish, profile)
    let candidates = vec![
        format!("{}/.zshrc", home),
        format!("{}/.bashrc", home),
        format!("{}/.config/fish/config.fish", home),
        format!("{}/.profile", home),
        format!("{}/.bash_profile", home),
    ];

    // Regex Explanation:
    // export\s+          : Match "export" followed by whitespace (optional in some shells, but standard for export)
    //                      Wait, some shells might just do KEY=VAL. 
    //                      Let's make export optional: (?:export\s+)?
    // {target_var}       : The variable name (e.g. GEMINI_API_KEY)
    // \s*=\s*            : Equals sign with optional whitespace
    // ["']?              : Optional opening quote
    // ([a-zA-Z0-9_\-]+)  : Capture group 1: The key itself (alphanumeric + _ + -)
    // ["']?              : Optional closing quote
    let pattern = format!(r#"(?:export\s+)?{}\s*=\s*["']?([a-zA-Z0-9_\-]+)["']?"#, target_var);
    let re = Regex::new(&pattern).ok()?;

    for path in candidates {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Some(caps) = re.captures(&content) {
                if let Some(matched) = caps.get(1) {
                    let raw_key = matched.as_str().to_string();
                    // We only return it if it looks "key-like" (e.g. not empty)
                    if !raw_key.is_empty() {
                         // Masking logic: Show first 3 chars, then ***
                         let masked = if raw_key.len() > 6 {
                             format!("{}*******{}", &raw_key[0..3], &raw_key[raw_key.len()-3..])
                         } else {
                             "******".to_string()
                         };
                         return Some((masked, path));
                    }
                }
            }
        }
    }

    None
}
