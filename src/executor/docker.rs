use async_trait::async_trait;
use crate::executor::action::{Action, DangerLevel, ExecutionPlan};
use crate::executor::ExecuteResult;
use crate::executor::service::{ServiceProvider, ServiceStatus};
use std::sync::Arc;

pub struct DockerProvider;

#[async_trait]
impl ServiceProvider for DockerProvider {
    fn name(&self) -> String {
        "docker".to_string()
    }

    async fn detect(&self, snapshot: &crate::system::snapshot::HostSnapshot) -> ServiceStatus {
        if snapshot.capabilities.has_docker {
            ServiceStatus::Active
        } else {
            ServiceStatus::NotInstalled
        }
    }

    fn get_actions(&self) -> Vec<Arc<dyn Action>> {
        vec![]
    }

    async fn install_action(&self) -> Option<Arc<dyn Action>> {
        // Future: sudo apt install docker.io
        None
    }

    async fn uninstall_action(&self) -> Option<Arc<dyn Action>> {
        None
    }
}

pub struct DockerPull {
    pub id: String,
    pub image_name: String,
}

#[async_trait]
impl Action for DockerPull {
    fn id(&self) -> String { self.id.clone() }
    fn name(&self) -> String { format!("Docker Pull: {}", self.image_name) }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Moderate }

    async fn validate(&self, snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> {
        if !snapshot.capabilities.has_docker {
            return Err("Docker is not installed".to_string());
        }
        Ok(())
    }

    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec![format!("Execute 'docker pull {}'", self.image_name)],
            estimated_impact: format!("Downloads docker image '{}'", self.image_name),
            danger_level: self.danger_level(),
        })
    }

    fn build_command(&self) -> String {
        format!("sudo docker pull {}", self.image_name)
    }

    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult { success: true, stdout: String::new(), stderr: String::new(), exit_code: Some(0) })
    }

    async fn rollback(&self) -> Result<(), String> {
        Ok(())
    }
}
