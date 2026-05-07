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

    pub fn mask(&mut self, real_name: &str, prefix: Option<&str>) -> String {
        if let Some(masked) = self.mapping.get(real_name) {
            return masked.clone();
        }
        
        // 🛡️ [Milestone v0.0.14.17] Deterministic Masking using MD5 hash
        let hash = format!("{:x}", md5::compute(real_name));
        let short_hash = &hash[0..4].to_uppercase();
        let base_masked = format!("REMOTE_{}", short_hash);
        
        let masked = if let Some(p) = prefix {
            format!("{}:{}", p, base_masked)
        } else {
            base_masked
        };
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
    pub user: Option<String>,
    pub port: Option<u16>,
    pub password: Option<String>,
}

impl SshProvider {
    pub fn new(ip: String, user: Option<String>, port: Option<u16>, password: Option<String>) -> Self {
        Self { ip, user, port, password }
    }
}

#[async_trait]
impl RemoteProvider for SshProvider {
    async fn list(&self, path: &str) -> Result<Vec<String>, String> {
        let cmd = format!("ls -m {}", path);
        let output = SshConnection::execute_remote_async(&self.ip, self.user.as_deref(), self.port, self.password.as_deref(), &cmd).await?;
        Ok(output.split(',').map(|s| s.trim().to_string()).collect())
    }

    async fn get_path(&self, path: &str) -> Result<String, String> {
        let cmd = format!("realpath {}", path);
        SshConnection::execute_remote_async(&self.ip, self.user.as_deref(), self.port, self.password.as_deref(), &cmd).await
    }

    async fn search(&self, query: &str) -> Result<Vec<String>, String> {
        let cmd = format!("find . -name '*{}*' -maxdepth 2", query);
        let output = SshConnection::execute_remote_async(&self.ip, self.user.as_deref(), self.port, self.password.as_deref(), &cmd).await?;
        Ok(output.lines().map(|s| s.to_string()).collect())
    }

    async fn sync(&self, source: &str, destination: &str) -> Result<(), String> {
        let mut args = vec!["-avz".to_string()];
        if let Some(p) = self.port {
            args.push("-e".to_string());
            args.push(format!("ssh -p {}", p));
        }
        
        let target = if let Some(u) = &self.user {
            format!("{}@{}:{}", u, self.ip, destination)
        } else {
            format!("{}:{}", self.ip, destination)
        };
        
        args.push(source.to_string());
        args.push(target);

        let output = std::process::Command::new("rsync")
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to execute rsync: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn mount(&self, path: &str, mount_point: &str) -> Result<(), String> {
        let mut args = Vec::new();
        if let Some(p) = self.port {
            args.push("-p".to_string());
            args.push(p.to_string());
        }
        
        let target = if let Some(u) = &self.user {
            format!("{}@{}:{}", u, self.ip, path)
        } else {
            format!("{}:{}", self.ip, path)
        };
        
        args.push(target);
        args.push(mount_point.to_string());

        let output = std::process::Command::new("sshfs")
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to execute sshfs: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}
