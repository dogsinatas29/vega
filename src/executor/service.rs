use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::executor::action::Action;
use crate::system::snapshot::HostSnapshot;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceStatus {
    Active,
    Inactive,
    NotInstalled,
    Error(String),
}

#[async_trait]
pub trait ServiceProvider: Send + Sync {
    /// Service name (e.g., "ollama", "docker")
    fn name(&self) -> String;

    /// Detect if the service is available on the host
    async fn detect(&self, snapshot: &HostSnapshot) -> ServiceStatus;

    /// Get list of supported actions for this service
    fn get_actions(&self) -> Vec<Arc<dyn Action>>;

    /// Action to install the service itself
    async fn install_action(&self) -> Option<Arc<dyn Action>>;

    /// Action to uninstall the service
    async fn uninstall_action(&self) -> Option<Arc<dyn Action>>;
}

pub struct ServiceRegistry {
    pub providers: Vec<Box<dyn ServiceProvider>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self { providers: Vec::new() }
    }

    pub fn register(&mut self, provider: Box<dyn ServiceProvider>) {
        self.providers.push(provider);
    }

    pub async fn find_service(&self, name: &str) -> Option<&Box<dyn ServiceProvider>> {
        self.providers.iter().find(|p| p.name() == name)
    }
}
