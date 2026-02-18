use std::process::Command;

pub struct SshConnection;

pub struct DiagnosticResult {
    pub message: String,
    pub recommendation: String,
}

impl std::fmt::Display for DiagnosticResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n   Action: {}", self.message, self.recommendation)
    }
}

impl SshConnection {
    pub fn check_connection(ip: &str, user: Option<&str>) -> Result<(), (Option<i32>, String)> {
        let target = if let Some(u) = user {
            format!("{}@{}", u, ip)
        } else {
            ip.to_string()
        };

        println!("🔌 Testing connection to {}...", target);

        let output = Command::new("ssh")
            .args(&[
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=3",
                "-o",
                "StrictHostKeyChecking=no",
                &target,
                "echo 'ok'",
            ])
            .output()
            .map_err(|e| (None, e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err((output.status.code(), stderr))
        }
    }

    pub fn connect(ip: &str, user: Option<&str>) {
        let target = if let Some(u) = user {
            format!("{}@{}", u, ip)
        } else {
            ip.to_string()
        };

        let _ = Command::new("ssh").arg(&target).status();
    }

    pub fn detect_os(ip: &str, user: Option<&str>) -> Option<String> {
        let target = if let Some(u) = user {
            format!("{}@{}", u, ip)
        } else {
            ip.to_string()
        };

        let common_args = &["-o", "BatchMode=yes", "-o", "ConnectTimeout=5", &target];

        // 1. Try getting ID from os-release (Standard Linux)
        if let Ok(output) = Command::new("ssh")
            .args(common_args)
            .arg("source /etc/os-release && echo $ID")
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }

        // 2. Fallback: uname -s (BSD/Busybox/Legacy)
        if let Ok(output) = Command::new("ssh")
            .args(common_args)
            .arg("uname -s")
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_lowercase();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }

        None
    }

    pub fn diagnose(status_code: Option<i32>, stderr: &str) -> DiagnosticResult {
        // Exit Code 255 is general SSH error

        if stderr.contains("Connection refused") {
            return DiagnosticResult {
                message: "❌ Connection Refused (Port 22 unreachable)".to_string(),
                recommendation: "1. Service: 'systemctl status sshd'\n   2. Firewall: 'sudo firewall-cmd --add-port=22/tcp --permanent'".to_string(),
            };
        }
        if stderr.contains("Permission denied") {
            return DiagnosticResult {
                message: "🔑 Permission Denied (Authentication Failed)".to_string(),
                recommendation: "Check '~/.ssh/authorized_keys' on target or verify Username."
                    .to_string(),
            };
        }
        if stderr.contains("timed out")
            || stderr.contains("No route to host")
            || status_code == Some(255)
        {
            return DiagnosticResult {
                message: "🔌 Network Timeout / Unreachable".to_string(),
                recommendation: "Verify VM is running ('virsh list') and Network Bridge is active."
                    .to_string(),
            };
        }
        if stderr.contains("Host key verification failed") {
            return DiagnosticResult {
                message: "🛡️ Host Key Change Detected (MITM/IP Rotation)".to_string(),
                recommendation: "'ssh-keygen -f \"$HOME/.ssh/known_hosts\" -R <ip_address>'"
                    .to_string(),
            };
        }

        DiagnosticResult {
            message: format!("⚠️ Unknown SSH Error (Code: {:?})", status_code),
            recommendation: format!("Raw Error: {}", stderr),
        }
    }

    #[allow(dead_code)]
    pub async fn execute_remote_async(ip: &str, cmd: &str) -> Result<String, String> {
        let output = tokio::process::Command::new("ssh")
            .args(&[
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=10",
                "-o",
                "StrictHostKeyChecking=no",
                ip,
                cmd,
            ])
            .stderr(std::process::Stdio::null()) // Sovereign SRE: Suppress remote noise
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}
