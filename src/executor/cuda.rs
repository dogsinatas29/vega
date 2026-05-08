use async_trait::async_trait;
use crate::executor::action::Action;
use crate::executor::service::{ServiceProvider, ServiceStatus};
use std::sync::Arc;

pub struct CudaProvider;

#[async_trait]
impl ServiceProvider for CudaProvider {
    fn name(&self) -> String {
        "cuda".to_string()
    }

    async fn detect(&self, snapshot: &crate::system::snapshot::HostSnapshot) -> ServiceStatus {
        if !snapshot.capabilities.has_nvidia {
            return ServiceStatus::NotInstalled; // No GPU, no CUDA needed
        }
        
        // Check if nvcc exists (Future implementation)
        ServiceStatus::Inactive 
    }

    fn get_actions(&self) -> Vec<Arc<dyn Action>> {
        vec![]
    }

    async fn install_action(&self) -> Option<Arc<dyn Action>> {
        None
    }

    async fn uninstall_action(&self) -> Option<Arc<dyn Action>> {
        None
    }
}
