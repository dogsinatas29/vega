use std::collections::{HashMap, HashSet};
use std::fs;

use serde::{Deserialize, Serialize};
use regex::Regex;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    SystemUpdate,
    SshConnect(String), // target
    ShowLog,
    Unknown,
}

pub struct TokenSaver {
    cache_path: String,
    cache: HashMap<String, String>, // Prompt -> Action/Response
    #[allow(dead_code)]
    local_keywords: Vec<String>,
    history: HashSet<String>,
}

impl TokenSaver {
    pub fn new(cache_path: &str, history_path: &str, keywords: Vec<String>) -> Self {
        let mut saver = TokenSaver {
            cache_path: cache_path.to_string(),
            cache: HashMap::new(),
            local_keywords: keywords,
            history: HashSet::new(),
        };
        saver.load_cache();
        saver.load_history(history_path);
        saver
    }

    fn load_cache(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.cache_path) {
            if let Ok(map) = serde_json::from_str(&content) {
                self.cache = map;
            }
        }
    }

    fn load_history(&mut self, path: &str) {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                if let Ok(v) = serde_json::from_str::<Value>(line) {
                    if let Some(cmd) = v.get("command").and_then(|c| c.as_str()) {
                        self.history.insert(cmd.to_string());
                    }
                }
            }
        }
    }

    pub fn match_local_intent(&self, input: &str) -> Action {
        let input_lower = input.to_lowercase();
        
        // Regex based routing (Hybrid Reasoning)
        if input_lower.contains("update") || input_lower.contains("upgrade") {
             if input_lower.contains("system") || input_lower.contains("vega") {
                 return Action::SystemUpdate;
             }
        }
        
        if input_lower.starts_with("vega ssh") {
             // Extract target
             let re = Regex::new(r"vega ssh (?:to )?(\S+)").unwrap();
             if let Some(caps) = re.captures(&input_lower) {
                 return Action::SshConnect(caps[1].to_string());
             }
        }
        
        if input_lower.contains("log") && (input_lower.contains("show") || input_lower.contains("check")) {
            return Action::ShowLog;
        }

        Action::Unknown
    }

    #[allow(dead_code)]
    pub fn check_cache(&self, input: &str) -> Option<&String> {
        self.cache.get(input)
    }

    pub fn search_history(&self, query: &str) -> Vec<String> {
        // Simple case-insensitive substring match
        let query_lower = query.to_lowercase();
        
        // Combine cache keys and history
        let mut matches: Vec<String> = self.cache.keys()
            .chain(self.history.iter())
            .filter(|k| k.to_lowercase().contains(&query_lower))
            .cloned()
            .collect();
            
        // Deduplicate and limit
        matches.sort();
        matches.dedup();
        matches.truncate(10);
        matches
    }
}
