use crate::executor::pipeline::{SimLog, VirtualExecutionEngine};
use crate::executor::ast::CommandAst;

pub struct BasicVee;

impl VirtualExecutionEngine for BasicVee {
    fn simulate(&self, ast: &CommandAst) -> anyhow::Result<SimLog> {
        let mut sim_log = SimLog {
            is_safe: true,
            predicted_impact: "Minimal".to_string(),
            risk_score: 0,
            suggestion: None,
        };

        let cmd_str = ast.to_shell_command();

        // 1. Destructive Command Detection
        if cmd_str.contains("rm -rf") {
            if cmd_str.contains("/") || cmd_str.contains("/home") || cmd_str.contains("/etc") {
                sim_log.is_safe = false;
                sim_log.risk_score = 100;
                sim_log.predicted_impact = "CRITICAL: Potential system-wide data loss.".to_string();
                sim_log.suggestion = Some("Specify a narrow subpath or use trash-cli.".to_string());
            } else {
                sim_log.risk_score = 60;
                sim_log.predicted_impact = "Likely recursive file deletion.".to_string();
            }
        }

        // 2. Disk Operations
        if cmd_str.contains("mkfs") || cmd_str.contains("fdisk") || cmd_str.contains("dd") {
            sim_log.risk_score = 90;
            sim_log.predicted_impact = "Disk partitioning or low-level formatting.".to_string();
        }

        // 3. Network/Sync
        if ast.tool == "rclone" {
            sim_log.risk_score = 20;
            sim_log.predicted_impact = "Cloud data synchronization.".to_string();
        }

        // 4. Package Management
        if ast.tool == "pkg" {
            sim_log.risk_score = 15;
            sim_log.predicted_impact = "System package modification.".to_string();
        }

        // 5. State-based Path Verification
        if let Some(src) = &ast.source {
            let path = std::path::Path::new(src);
            if !path.exists() && ast.tool != "ssh" { // ssh target is remote, don't check locally
                sim_log.is_safe = false;
                sim_log.risk_score = 40;
                sim_log.predicted_impact = format!("ERROR: Source path '{}' does not exist.", src);
                sim_log.suggestion = Some("Verify the path name or ensure the file/directory exists.".to_string());
            }
        }

        // 6. Rclone mask check (Safety check for masked names)
        if ast.tool == "rclone" {
            if let Some(dst) = &ast.destination {
                if dst.contains(":") && !dst.starts_with("/") {
                   // Looks like a remote target
                   sim_log.predicted_impact.push_str(" (Targeting remote cloud storage)");
                }
            }
        }

        Ok(sim_log)
    }
}
