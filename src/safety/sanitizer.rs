use regex::Regex;
use std::sync::OnceLock;

static IP_REGEX: OnceLock<Regex> = OnceLock::new();
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
// Simple heuristic for API keys or Secrets: High entropy or common prefixes
static SECRET_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn sanitize_input(input: &str) -> String {
    let mut sanitized = input.to_string();

    let ip_re = IP_REGEX.get_or_init(|| {
        Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap()
    });

    let email_re = EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap()
    });

    // Basic heuristic for Bearer tokens or keys starting with 'sk-' (OpenAI) or similar
    let secret_re = SECRET_REGEX.get_or_init(|| {
        Regex::new(r"(sk-[a-zA-Z0-9]{20,})|(Bearer [a-zA-Z0-9\-\._~+/]+=*)").unwrap()
    });

    sanitized = ip_re.replace_all(&sanitized, "[REDACTED_IP]").to_string();
    sanitized = email_re.replace_all(&sanitized, "[REDACTED_EMAIL]").to_string();
    sanitized = secret_re.replace_all(&sanitized, "[REDACTED_SECRET]").to_string();

    sanitized
}
