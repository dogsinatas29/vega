use crate::storage::db::Database;
use crate::executor::ExecuteResult;
use crate::system::config::AppConfig;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;
use serde::{Serialize, Deserialize};

use serde_json::json;

use crate::safety::sanitizer::sanitize_input as sanitize_string;

#[derive(Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub timestamp: String,
    pub command: String,
    pub success: bool,
    pub exit_code: i32,
    pub result: ExecuteResult,
    pub healer_action: Option<String>,
}

pub struct Archivist;

impl Archivist {
    pub fn log_execution(
        db: &Option<Database>, 
        session_id: &Option<i64>, 
        project_name: Option<&str>,
        command: &str, 
        result: &ExecuteResult, 
        healer_action: Option<String>,
        token_usage: Option<i32>
    ) {
        let timestamp = Local::now().to_rfc3339();
        let sanitized_cmd = sanitize_string(command);

        // 1. Hot Data: SQLite
        if let (Some(database), Some(sid)) = (db, session_id) {
            // Use advanced task logging
            let _ = database.log_task(
                *sid,
                project_name,
                &sanitized_cmd,
                result.exit_code.unwrap_or(-1),
                &result.stdout,
                &result.stderr,
                healer_action.is_some(),
                healer_action.as_deref(),
                token_usage
            );
        }

        // 2. Cold Data: File Archive
        let entry = ArchiveEntry {
            timestamp: timestamp.clone(),
            command: sanitized_cmd.clone(),
            success: result.success,
            exit_code: result.exit_code.unwrap_or(-1),
            result: result.clone(),
            healer_action: healer_action.clone(),
        };

        // Save JSON
        Self::append_to_json_history(&entry);

        // Save Markdown Report
        Self::append_to_daily_report(&entry);
    }

    fn get_history_dir(year: &str, month: &str) -> PathBuf {
        let config_path = AppConfig::get_config_path();
        let history_dir = config_path.parent().unwrap().join("history").join(year).join(month);
        if !history_dir.exists() {
            fs::create_dir_all(&history_dir).ok();
        }
        history_dir
    }

    fn append_to_json_history(entry: &ArchiveEntry) {
        let now = Local::now();
        let year = now.format("%Y").to_string();
        let month = now.format("%m").to_string();
        
        let path = Self::get_history_dir(&year, &month).join("history.json");
        // For simplicity, we are appending line-delimited JSON (NDJSON) or just appending to a list?
        // True "history.json" usually implies a valid JSON array. But appending to an array is expensive (read-parse-write).
        // Let's use NDJSON (Newlines) for efficiency and robustness.
        
        if let Ok(json_str) = serde_json::to_string(entry) {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                if let Err(e) = writeln!(file, "{}", json_str) {
                    eprintln!("Failed to write history.json: {}", e);
                }
            }
        }
    }

    fn append_to_daily_report(entry: &ArchiveEntry) {
        let now = Local::now();
        let current_date = now.format("%Y-%m-%d").to_string();
        let year = now.format("%Y").to_string();
        let month = now.format("%m").to_string();
        
        let filename = format!("report_{}.md", current_date);
        let path = Self::get_history_dir(&year, &month).join(filename);

        let status_icon = if entry.success { "✅" } else { "❌" };
        let fail_tag = if !entry.success { " #FAILED" } else { "" };
        let healer_note = if let Some(action) = &entry.healer_action {
             format!("\n- **🚑 Healer:** {}", action)
        } else {
             String::new()
        };

        // Construct Markdown Entry
        let md_entry = format!(
            "\n## {} Execution: `{}` {}\n- **Time:** {}\n- **Exit Code:** {}\n- **Result:** {}{}{}\n",
            status_icon, 
            entry.command, 
            fail_tag,
            now.format("%H:%M:%S"), 
            entry.exit_code, 
            status_icon,
            healer_note,
            if !entry.success { format!("\n- **Error:**\n```\n{}\n```", entry.result.stderr.trim()) } else { String::new() }
        );

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            // Write header if new file
            if file.metadata().map(|m| m.len() == 0).unwrap_or(false) {
                let _ = writeln!(file, "# 🛡️ Daily Report: {}\n", current_date);
            }
            let _ = write!(file, "{}", md_entry);
        }
    }

    // internal sanitize_command removed in favor of crate::safety::sanitizer


    /// JSON & Markdown Report Generator (End of Session)
    pub fn archive_session(db: &Database, session_id: i64) {
        if let Ok(history) = db.get_session_tasks(session_id) {
            if history.is_empty() { return; }

            // Determine project name from usage (first non-none) or default "vega"
            let project_name = history.iter()
                .find_map(|t| t.project_name.clone())
                .unwrap_or_else(|| "vega".to_string());
                
            let now = Local::now();
            let timestamp_str = now.format("%Y%m%d_%H%M%S").to_string();
            let year = now.format("%Y").to_string();
            let month = now.format("%m").to_string();
            
            let report_dir = Self::get_history_dir(&year, &month);
            
            // Determine Status
            let has_failures = history.iter().any(|t| t.exit_code != 0);
            let has_recovery = history.iter().any(|t| t.healer_used);
            
            let status = if has_failures && !has_recovery {
                "FAILED"
            } else if has_recovery {
                "RECOVERED"
            } else {
                "SUCCESS"
            };

            // 1. JSON
            let json_filename = format!("{}_{}_{}.json", timestamp_str, project_name, status);
            let json_path = report_dir.join(json_filename);
            
            let json_data = json!({
                "project": project_name,
                "timestamp": timestamp_str,
                "status": status,
                "tasks": history.iter().map(|t| json!({
                    "command": sanitize_string(&t.command),
                    "exit_code": t.exit_code,
                    "success": t.exit_code == 0,
                    "token_usage": t.token_usage,
                    "healer_used": t.healer_used,
                    "healer_log": t.healer_log
                })).collect::<Vec<_>>()
            });
            fs::write(&json_path, json_data.to_string()).ok();

            // 2. Markdown
            let md_filename = format!("{}_{}_{}.md", timestamp_str, project_name, status);
            let md_path = report_dir.join(md_filename);
            
            let mut md_content = format!("# 🛡️ VEGA Session Report: {} ({})\n**Status:** `{}`\n\n", project_name, timestamp_str, status);
            
            for task in &history {
                let clean_cmd = sanitize_string(&task.command);
                let status_icon = if task.exit_code == 0 { "✅" } else { "❌" };
                md_content.push_str(&format!(
                    "### ❯ `{}`\n- **Result:** {} (Exit: {})\n- **Tokens:** {}\n",
                    clean_cmd,
                    status_icon,
                    task.exit_code,
                    task.token_usage.unwrap_or(0)
                ));
                 if task.healer_used {
                    md_content.push_str(&format!("- **🚑 Healer:** {}\n", task.healer_log.as_deref().unwrap_or("Action taken")));
                }
                md_content.push_str("\n---\n");
            }
            
            // Append Cynical Summary placeholder
            md_content.push_str("\n\n> *\"I've logged everything. Try to be more efficient next time.\"* - VEGA\n");
            
            fs::write(md_path, md_content).ok();
        }
    }
}
