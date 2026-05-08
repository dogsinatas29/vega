use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpec {
    pub name: String,
    pub min_version: Option<String>,
    pub description: String,
    pub required_disk_gb: f64,
    pub required_ram_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub tool_name: String,
    pub version: String,
    pub commands: HashMap<String, CommandSpec>,
}

pub struct CapabilityRegistry;

impl CapabilityRegistry {
    pub fn get_ollama_capabilities(version: &str) -> ToolCapabilities {
        let mut commands = HashMap::new();
        
        commands.insert("list".to_string(), CommandSpec {
            name: "list".to_string(),
            min_version: None,
            description: "List models".to_string(),
            required_disk_gb: 0.1,
            required_ram_gb: 0.5,
        });
        
        commands.insert("ps".to_string(), CommandSpec {
            name: "ps".to_string(),
            min_version: Some("0.1.33".to_string()),
            description: "List running models".to_string(),
            required_disk_gb: 0.1,
            required_ram_gb: 0.5,
        });
        
        commands.insert("pull".to_string(), CommandSpec {
            name: "pull".to_string(),
            min_version: None,
            description: "Pull a model".to_string(),
            required_disk_gb: 5.0,  // Base models need space
            required_ram_gb: 2.0,
        });

        ToolCapabilities {
            tool_name: "ollama".to_string(),
            version: version.to_string(),
            commands,
        }
    }

    pub fn validate_command(tool: &str, command: &str, version: &str) -> Result<(), String> {
        match tool {
            "ollama" => {
                let caps = Self::get_ollama_capabilities(version);
                if caps.commands.contains_key(command) {
                    Ok(())
                } else {
                    Err(format!("Hallucinated command detected: 'ollama {}' is not a valid command in the registry.", command))
                }
            },
            _ => Ok(()), // Default to pass for other tools for now
        }
    }

    pub fn check_resource_availability(tool: &str, command: &str, snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> {
        match tool {
            "ollama" => {
                let version = snapshot.capabilities.ollama_version.as_deref().unwrap_or("0.0.0");
                let caps = Self::get_ollama_capabilities(version);
                if let Some(spec) = caps.commands.get(command) {
                    // Check Disk
                    if snapshot.capabilities.disk_free_root_gb < spec.required_disk_gb {
                        return Err(format!(
                            "Insufficient disk space for 'ollama {}': {}GB available, {}GB required.",
                            command, snapshot.capabilities.disk_free_root_gb, spec.required_disk_gb
                        ));
                    }
                    // Check RAM
                    if snapshot.capabilities.ram_free_gb < spec.required_ram_gb {
                        return Err(format!(
                            "Insufficient RAM for 'ollama {}': {}GB available, {}GB required.",
                            command, snapshot.capabilities.ram_free_gb, spec.required_ram_gb
                        ));
                    }
                }
                Ok(())
            },
            _ => Ok(()),
        }
    }
}
