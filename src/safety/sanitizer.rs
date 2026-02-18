use regex::Regex;
use std::sync::OnceLock;

static IP_REGEX: OnceLock<Regex> = OnceLock::new();
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
static SECRET_REGEX: OnceLock<Regex> = OnceLock::new();
static KEY_VALUE_REGEX: OnceLock<Regex> = OnceLock::new();
static EXPORT_REGEX: OnceLock<Regex> = OnceLock::new();
static FLAG_REGEX: OnceLock<Regex> = OnceLock::new();
static WAD_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn sanitize_input(input: &str) -> String {
    let mut sanitized = input.to_string();

    let ip_re = IP_REGEX.get_or_init(|| Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap());

    let email_re = EMAIL_REGEX
        .get_or_init(|| Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap());

    // Combined Secret Regex: sk- keys, Bearer tokens, generic API keys
    let secret_re = SECRET_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(sk-[a-zA-Z0-9_\-]{20,})|(Bearer [a-zA-Z0-9\-\._~+/]+=*)|((api[_-]?key|token|auth|secret)[:=]\s*[a-zA-Z0-9_-]{30,})").unwrap()
    });

    // Key-Value pairs (password=123)
    let kv_re = KEY_VALUE_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(password|pass|token|key|secret)(\s*=\s*|\s+)(\S+)").unwrap()
    });

    // Export commands
    let export_re =
        EXPORT_REGEX.get_or_init(|| Regex::new(r"(?i)export\s+\w+=(.+)[ \t\n]*").unwrap());

    // Flags (-p 1234)
    let flag_re = FLAG_REGEX.get_or_init(|| Regex::new(r"(-p|--password)(\s+)(\S+)").unwrap());

    // Sensitive Files (.wad)
    let wad_re = WAD_REGEX.get_or_init(|| Regex::new(r"/(?:[^/]+/)*[^/]+\.wad").unwrap());

    sanitized = ip_re.replace_all(&sanitized, "[REDACTED_IP]").to_string();
    sanitized = email_re
        .replace_all(&sanitized, "[REDACTED_EMAIL]")
        .to_string();
    sanitized = secret_re
        .replace_all(&sanitized, "[REDACTED_SECRET]")
        .to_string();
    sanitized = kv_re.replace_all(&sanitized, "$1$2[REDACTED]").to_string();
    // For export, we might want to be careful not to redact the variable name if possible,
    // but the regex captures the value in group 1.
    // The replace_all behavior depends on the regex.
    // The current regex `export\s+\w+=(.+)` matches the whole string?
    // Let's stick to the previous implementation style.
    // Actually, `replace_all` replaces the *match*.
    // If I want to keep "export VAR=", I should change the regex or the replacement.
    // `export_re` is `export\s+\w+=(.+)`. replacement: `export [REDACTED_EXPORT]`.
    // That replaces the whole line. Acceptable for now as per previous impl.
    sanitized = export_re
        .replace_all(&sanitized, "export [REDACTED_EXPORT]")
        .to_string();
    sanitized = flag_re
        .replace_all(&sanitized, "$1$2[REDACTED]")
        .to_string();
    sanitized = wad_re
        .replace_all(&sanitized, "[REDACTED_FILE]")
        .to_string();

    sanitized
}
