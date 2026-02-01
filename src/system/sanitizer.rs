use regex::Regex;

pub struct Sanitizer {
    patterns: Vec<Regex>,
}

impl Sanitizer {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // 1. General API Key Pattern (Alphanumeric 30+ chars prefixed by key/token/auth/secret)
                Regex::new(r"(?i)(api[_-]?key|token|auth|secret)[:=]\s*[a-zA-Z0-9_-]{30,}").unwrap(),
                // 2. Export command pattern
                Regex::new(r"(?i)export\s+\w+=(.+)[ \t\n]*").unwrap(),
                // 3. Flags (e.g. -p 1234)
                Regex::new(r"(-p|--password)(\s+)(\S+)").unwrap(),
                // 4. Specific Sensitive Files (e.g. .wad) per user request
                Regex::new(r"/(?:[^/]+/)*[^/]+\.wad").unwrap(),
            ],
        }
    }

    pub fn clean(&self, input: &str) -> String {
        let mut output = input.to_string();
        for re in &self.patterns {
            output = re.replace_all(&output, "[REDACTED]").to_string();
        }
        output
    }
}

// Global static helper for ease of use if needed, but struct is better for pre-compiling regex
pub fn sanitize_string(input: &str) -> String {
    let s = Sanitizer::new();
    s.clean(input)
}
