pub mod healer;
pub use healer::Healer;

use std::process::{Command, Stdio};
use std::io::Read;
use log::{info, error, debug};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    PermissionDenied,
    CommandNotFound,
    PathNotFound,
    ResourceLocked,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub error_type: Option<ErrorCategory>,
}

pub struct Executor;

impl Executor {
    pub fn execute_command(command_str: &str) -> ExecuteResult {
        info!("🐚 Executing Shell Command: {}", command_str);

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        
        match Command::new(&shell)
            .arg("-c")
            .arg(command_str)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn() 
        {
            Ok(mut child) => {
                let mut stdout_str = String::new();
                let mut stderr_str = String::new();
                
                if let Some(mut stdout) = child.stdout.take() {
                    let _ = stdout.read_to_string(&mut stdout_str);
                }
                if let Some(mut stderr) = child.stderr.take() {
                    let _ = stderr.read_to_string(&mut stderr_str);
                }

                let status = child.wait().unwrap_or_else(|_| std::process::ExitStatus::from_raw(1));
                let exit_code = status.code().unwrap_or(1);
                let success = status.success();
                
                let error_type = if !success {
                    Some(Self::classify_error(&stderr_str, &stdout_str, exit_code))
                } else {
                    None
                };

                if success {
                    debug!("   ✅ Command successful");
                } else {
                    error!("   ❌ Command failed (Code: {})", exit_code);
                }

                ExecuteResult {
                    success,
                    stdout: stdout_str,
                    stderr: stderr_str,
                    exit_code,
                    error_type,
                }
            },
            Err(e) => {
                ExecuteResult {
                    success: false,
                    stdout: String::new(),
                    stderr: e.to_string(),
                    exit_code: 1,
                    error_type: Some(ErrorCategory::Unknown),
                }
            }
        }
    }

    fn classify_error(stderr: &str, stdout: &str, code: i32) -> ErrorCategory {
        let combined = format!("{} {}", stderr, stdout).to_lowercase();
        
        if combined.contains("permission denied") || code == 126 {
            ErrorCategory::PermissionDenied
        } else if combined.contains("command not found") || combined.contains("not found") || code == 127 {
            ErrorCategory::CommandNotFound
        } else if combined.contains("no such file") || combined.contains("directory") {
            ErrorCategory::PathNotFound
        } else if combined.contains("lock") || combined.contains("temporarily unavailable") {
            ErrorCategory::ResourceLocked
        } else {
            ErrorCategory::Unknown
        }
    }
}

// Unix extension for creating ExitStatus from raw if needed, though simple wait() usually suffices
// We only use from_raw via a polyfill logic or just ignoring it for now as wait() should essentially work.
// Actually, std::process::ExitStatus doesn't expose a clean public constructor for integers easily without OsExt.
// But wait() returns Result<ExitStatus>, so we handle error case manually.
trait ExitStatusExt {
    fn from_raw(raw: i32) -> std::process::ExitStatus;
}
impl ExitStatusExt for std::process::ExitStatus {
    fn from_raw(raw: i32) -> std::process::ExitStatus {
        <std::process::ExitStatus as std::os::unix::process::ExitStatusExt>::from_raw(raw)
    }
}
