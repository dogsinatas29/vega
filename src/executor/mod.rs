use serde::{Deserialize, Serialize};

pub mod goal;
pub mod orchestrator;
pub mod pkg;
pub mod status;
pub mod ast;
pub mod pipeline;
pub mod template;
pub mod virt;
pub mod action;
pub mod ollama;
pub mod capabilities;
pub mod service;
pub mod docker;
pub mod system;
pub mod cuda;
pub mod discovery;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Success,
    SuccessAlreadySatisfied, // Desired state already reached
    PartialSuccess,
    RetryableFailure,
    FatalFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResult {
    pub success: bool,
    pub status: ExecutionStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<OrchestrationError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insight: Option<String>, // SRE Insight for troubleshooting
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationError {
    Transport(String), // Permission denied, Timeout, etc.
    Protocol(String),  // Malformed JSON, Version mismatch
    Execution(String), // Command failed, Resource exhausted
}

impl Default for ExecuteResult {
    fn default() -> Self {
        Self {
            success: true,
            status: ExecutionStatus::Success,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: Some(0),
            error: None,
            insight: None,
        }
    }
}

impl ExecuteResult {
    pub fn new(success: bool, status: ExecutionStatus, stdout: String, stderr: String, exit_code: Option<i32>) -> Self {
        Self {
            success,
            status,
            stdout,
            stderr,
            exit_code,
            error: None,
            insight: None,
        }
    }
}
