use crate::ai::{AiProvider, QuotaStatus};
use crate::context::SystemContext;
use async_trait::async_trait;
use std::error::Error;

pub struct OfflineEngine;

impl OfflineEngine {
    pub fn new() -> Self {
        Self {}
    }

    fn map_query(&self, query: &str) -> Option<String> {
        let q = query.to_lowercase();
        
        // Simple Rule-based Mapping
        if q.contains("disk") || q.contains("space") {
            return Some("df -h".to_string());
        }
        if q.contains("ip") || q.contains("network") {
            return Some("ip a".to_string());
        }
        if q.contains("list") || q.contains("files") {
            return Some("ls -lah".to_string());
        }
        if q.contains("memory") || q.contains("ram") {
            return Some("free -h".to_string());
        }
        if q.contains("process") || q.contains("top") {
             return Some("ps aux | head -n 10".to_string());
        }

        None
    }
}

#[async_trait]
impl AiProvider for OfflineEngine {
    fn name(&self) -> &str {
        "Vega Offline (Rule-based)"
    }
    
    fn get_quota_status(&self) -> QuotaStatus {
        QuotaStatus::Unlimited
    }

    async fn generate_response(&self, _context: &SystemContext, prompt: &str) -> Result<String, Box<dyn Error>> {
        match self.map_query(prompt) {
            Some(cmd) => Ok(cmd),
            None => Err("No offline rule found for this query.".into())
        }
    }
}
