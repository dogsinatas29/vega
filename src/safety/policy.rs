use serde::{Deserialize, Serialize};
use crate::executor::action::DangerLevel;
use crate::system::snapshot::HostSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub name: String,
    pub target_host_pattern: String, // Regex or "*"
    pub blocked_danger_levels: Vec<DangerLevel>,
    pub reason: String,
}

pub struct PolicyEngine {
    policies: Vec<Policy>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        // For Phase 2, we can hardcode some default policies
        // Future: Load from vega.toml or DB
        let default_policies = vec![
            Policy {
                name: "Production Safety".to_string(),
                target_host_pattern: "192.168.0.1$".to_string(), // Example specific IP
                blocked_danger_levels: vec![DangerLevel::Critical],
                reason: "Critical actions are blocked on production hosts".to_string(),
            },
            Policy {
                name: "Resource Protection".to_string(),
                target_host_pattern: "*".to_string(),
                blocked_danger_levels: vec![], // No blocks by default
                reason: "General policy".to_string(),
            },
        ];

        Self { policies: default_policies }
    }

    pub fn check(&self, host: &HostSnapshot, danger_level: DangerLevel) -> Result<(), String> {
        for policy in &self.policies {
            // Simple match for now: if pattern is "*" it matches all. 
            // Otherwise, check if IP ends with the pattern
            let host_matches = policy.target_host_pattern == "*" || host.ip.ends_with(&policy.target_host_pattern);
            
            if host_matches && policy.blocked_danger_levels.contains(&danger_level) {
                return Err(format!("Policy Violation: {} - {}", policy.name, policy.reason));
            }
        }
        Ok(())
    }
}
