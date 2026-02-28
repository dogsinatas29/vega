use crate::connection::ssh::SshConnection;
use async_trait::async_trait;
use std::collections::HashMap;

pub mod rclone;

pub struct RemoteMasker {
    mapping: HashMap<String, String>,         // real -> masked
    reverse_mapping: HashMap<String, String>, // masked -> real
}

impl RemoteMasker {
    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
            reverse_mapping: HashMap::new(),
        }
    }

    pub fn mask(&mut self, real_name: &str) -> String {
        if let Some(masked) = self.mapping.get(real_name) {
            return masked.clone();
        }
        let count = self.mapping.len() + 1;
        let masked = format!("REMOTE_{:02}", count);
        self.mapping.insert(real_name.to_string(), masked.clone());
        self.reverse_mapping
            .insert(masked.clone(), real_name.to_string());
        masked
    }

    pub fn unmask(&self, masked_name: &str) -> Option<String> {
        self.reverse_mapping.get(masked_name).cloned()
    }

    pub fn resolve_command(&self, cmd: &str) -> String {
        let mut resolved = cmd.to_string();
        for (masked, real) in &self.reverse_mapping {
            // Replace masked with real (e.g. REMOTE_01 -> gdrive)
            resolved = resolved.replace(masked, real);
        }
        resolved
    }
}

#[async_trait]
pub trait RemoteProvider: Send + Sync {
    async fn list(&self, path: &str) -> Result<Vec<String>, String>;
    async fn get_path(&self, path: &str) -> Result<String, String>;
    async fn search(&self, query: &str) -> Result<Vec<String>, String>;
    async fn sync(&self, source: &str, destination: &str) -> Result<(), String>;
    async fn mount(&self, path: &str, mount_point: &str) -> Result<(), String>;
}

pub struct SshProvider {
    pub ip: String,
}

impl SshProvider {
    pub fn new(ip: String) -> Self {
        Self { ip }
    }
}

#[async_trait]
impl RemoteProvider for SshProvider {
    async fn list(&self, path: &str) -> Result<Vec<String>, String> {
        let cmd = format!("ls -m {}", path);
        let output = SshConnection::execute_remote_async(&self.ip, &cmd).await?;
        Ok(output.split(',').map(|s| s.trim().to_string()).collect())
    }

    async fn get_path(&self, path: &str) -> Result<String, String> {
        let cmd = format!("realpath {}", path);
        SshConnection::execute_remote_async(&self.ip, &cmd).await
    }

    async fn search(&self, query: &str) -> Result<Vec<String>, String> {
        let cmd = format!("find . -name '*{}*' -maxdepth 2", query);
        let output = SshConnection::execute_remote_async(&self.ip, &cmd).await?;
        Ok(output.lines().map(|s| s.to_string()).collect())
    }

    async fn sync(&self, source: &str, destination: &str) -> Result<(), String> {
        let output = std::process::Command::new("rsync")
            .args(&["-avz", source, &format!("{}:{}", self.ip, destination)])
            .output()
            .map_err(|e| format!("Failed to execute rsync: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn mount(&self, path: &str, mount_point: &str) -> Result<(), String> {
        let full_remote_path = format!("{}:{}", self.ip, path);
        let output = std::process::Command::new("sshfs")
            .args(&[&full_remote_path, mount_point])
            .output()
            .map_err(|e| format!("Failed to execute sshfs: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}
