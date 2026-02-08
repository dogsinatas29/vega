use std::process::Command;
use std::fs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ShellSnapshot {
    pub aliases: HashMap<String, String>,
    pub zoxide_paths: Vec<String>,
}

impl ShellSnapshot {
    pub fn new() -> Self {
        ShellSnapshot {
            aliases: Self::dump_aliases(),
            zoxide_paths: Self::dump_zoxide(),
        }
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let path_obj = std::path::Path::new(path);
        if let Some(parent) = path_obj.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &str) -> Option<Self> {
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    fn dump_aliases() -> HashMap<String, String> {
        let mut map = HashMap::new();
        // Use timeout to prevent hanging if .zshrc is interactive/broken
        let output = Command::new("timeout")
            .args(&["3s", "zsh", "-i", "-c", "alias"])
            .output();

        if let Ok(o) = output {
            let stdout = String::from_utf8_lossy(&o.stdout);
            for line in stdout.lines() {
                if let Some((key, val)) = line.split_once('=') {
                        map.insert(key.to_string(), val.trim_matches('\'').to_string());
                }
            }
        }
        map
    }

    fn dump_zoxide() -> Vec<String> {
        let output = Command::new("zoxide")
            .args(&["query", "-l"])
            .output();
            
        if let Ok(o) = output {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        }
    }
}
