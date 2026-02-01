
use std::env;
use crate::system::config::{AppConfig, Project};

pub struct SmartContext;

impl SmartContext {
    /// Checks if the current directory corresponds to a known project in Config.
    pub fn detect_project(config: &AppConfig) -> Option<Project> {
        let current_dir = env::current_dir().ok()?;
        
        for project in &config.projects {
            // Check if current_dir starts with project.path
            // e.g. /home/user/project/src is inside /home/user/project
            if current_dir.starts_with(&project.path) {
                return Some(project.clone());
            }
        }
        None
    }

    /// Checks git status in the current directory (Simple "modified" check)
    pub fn check_git_status() -> Option<String> {
        use std::process::Command;
        
        // git status --porcelain
        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .output()
            .ok()?;
            
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                let lines = stdout.lines().count();
                return Some(format!("{} files changed", lines));
            }
        }
        None
    }
}
