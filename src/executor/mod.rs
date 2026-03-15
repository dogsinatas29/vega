use serde::{Deserialize, Serialize};

pub mod orchestrator;
pub mod pkg;
pub mod status;
pub mod ast;
pub mod pipeline;
pub mod template;
pub mod virt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ExecuteResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}
