pub mod pkg;
pub mod status;
pub mod orchestrator;

#[derive(Debug, Clone)]
pub struct ExecuteResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

