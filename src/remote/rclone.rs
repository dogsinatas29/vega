use crate::remote::RemoteProvider;
use async_trait::async_trait;
use log::info;
use serde_json::Value;
use std::process::Command;

pub struct RcloneProvider {
    pub remote_name: String,
}

impl RcloneProvider {
    pub fn new(remote_name: String) -> Self {
        Self { remote_name }
    }

    pub fn execute_rclone(&self, args: Vec<&str>) -> Result<String, String> {
        let output = Command::new("rclone")
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute rclone: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn list_remotes() -> Result<Vec<String>, String> {
        let output = Command::new("rclone")
            .arg("listremotes")
            .output()
            .map_err(|e| format!("Failed to list rclone remotes: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|l| l.trim_end_matches(':').to_string())
                .collect())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

#[async_trait]
impl RemoteProvider for RcloneProvider {
    async fn list(&self, path: &str) -> Result<Vec<String>, String> {
        let full_path = format!("{}:{}", self.remote_name, path);
        let output = self.execute_rclone(vec!["lsjson", &full_path])?;

        let json: Value = serde_json::from_str(&output)
            .map_err(|e| format!("Failed to parse rclone output: {}", e))?;

        let mut items = Vec::new();
        if let Some(array) = json.as_array() {
            for item in array {
                if let Some(name) = item["Name"].as_str() {
                    items.push(name.to_string());
                }
            }
        }
        Ok(items)
    }

    async fn get_path(&self, path: &str) -> Result<String, String> {
        Ok(format!("{}:{}", self.remote_name, path))
    }

    async fn search(&self, query: &str) -> Result<Vec<String>, String> {
        // Limited search to avoid excessive API calls
        let full_path = format!("{}:", self.remote_name);
        let output = self.execute_rclone(vec![
            "lsf",
            "-R",
            "--max-depth",
            "2",
            "--include",
            &format!("*{}*", query),
            &full_path,
        ])?;
        Ok(output.lines().map(|s| s.to_string()).collect())
    }

    async fn sync(&self, source: &str, destination: &str) -> Result<(), String> {
        info!("🔄 Syncing {} to {}", source, destination);
        self.execute_rclone(vec!["sync", source, destination])?;
        Ok(())
    }

    async fn mount(&self, path: &str, mount_point: &str) -> Result<(), String> {
        let full_path = format!("{}:{}", self.remote_name, path);
        info!("🔌 Mounting {} to {}", full_path, mount_point);

        // Use nohup or spawn in background to avoid blocking
        Command::new("rclone")
            .args(&[
                "mount",
                &full_path,
                mount_point,
                "--vfs-cache-mode",
                "full",
                "--daemon",
            ])
            .spawn()
            .map_err(|e| format!("Failed to mount rclone: {}", e))?;

        Ok(())
    }
}
