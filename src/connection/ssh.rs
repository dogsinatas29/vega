use std::process::Command;
use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

pub struct SshConnection;

impl SshConnection {
    /// Verifies if the port is actually an SSH server by checking the protocol banner.
    pub fn verify_ssh_fingerprint(ip: &str, port: u16) -> bool {
        if let Ok(mut stream) = TcpStream::connect_timeout(
            &format!("{}:{}", ip, port).parse().unwrap_or("127.0.0.1:0".parse().unwrap()),
            Duration::from_millis(500)
        ) {
            let mut buffer = [0; 8];
            if stream.set_read_timeout(Some(Duration::from_millis(500))).is_ok() {
                if stream.read_exact(&mut buffer).is_ok() {
                    let banner = String::from_utf8_lossy(&buffer);
                    return banner.starts_with("SSH-2.0-");
                }
            }
        }
        false
    }
}

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
    pub fn check_connection(ip: &str, user: Option<&str>, port: Option<u16>) -> Result<(), (Option<i32>, String)> {
        let target = if let Some(u) = user {
            format!("{}@{}", u, ip)
        } else {
            ip.to_string()
        };

        let port_str = port.unwrap_or(22).to_string();

        let output = Command::new("ssh")
            .args(&[
                "-o", "BatchMode=yes",
                "-o", "ConnectTimeout=30",
                "-o", "StrictHostKeyChecking=no",
                "-p", &port_str,
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

    pub fn connect(ip: &str, user: Option<&str>, port: Option<u16>) {
        let target = if let Some(u) = user {
            format!("{}@{}", u, ip)
        } else {
            ip.to_string()
        };
        let port_str = port.unwrap_or(22).to_string();
        let _ = Command::new("ssh").arg("-p").arg(&port_str).arg(&target).status();
    }

    pub fn detect_os(ip: &str, user: Option<&str>, port: Option<u16>) -> Option<String> {
        let target = if let Some(u) = user {
            format!("{}@{}", u, ip)
        } else {
            ip.to_string()
        };
        let port_str = port.unwrap_or(22).to_string();
        let common_args = &["-o", "BatchMode=yes", "-o", "ConnectTimeout=5", "-p", &port_str, &target];

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

    pub fn execute_output(ip: &str, user: Option<&str>, port: Option<u16>, cmd: &str) -> Result<String, String> {
        let target = if let Some(u) = user {
            format!("{}@{}", u, ip)
        } else {
            ip.to_string()
        };
        let port_str = port.unwrap_or(22).to_string();

        let output = Command::new("ssh")
            .args(&[
                "-o", "BatchMode=yes",
                "-o", "ConnectTimeout=5",
                "-o", "StrictHostKeyChecking=no",
                "-p", &port_str,
                &target,
                cmd,
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
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
            || stderr.contains("Network is unreachable")
        {
            return DiagnosticResult {
                message: "🔌 Network Timeout / Unreachable".to_string(),
                recommendation: "Verify VM is running ('virsh list') and Network Bridge is active."
                    .to_string(),
            };
        }

        // If it's 255 but doesn't have a specific message, it's a generic connection/auth failure
        if status_code == Some(255) {
            return DiagnosticResult {
                message: "🔌 Connection Failed (SSH 255)".to_string(),
                recommendation: format!("Check if port 22 is open and credentials are correct. Raw: {}", stderr),
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

    pub async fn execute_remote_async(
        ip: &str,
        user: Option<&str>,
        port: Option<u16>,
        password: Option<&str>,
        cmd: &str,
    ) -> Result<String, String> {
        let port_val = port.unwrap_or(22);
        let user_val = user.unwrap_or("root").to_string();
        let cmd_val = cmd.to_string();
        let ip_val = ip.to_string();

        if let Some(pass) = password {
            // 🛡️ [Milestone v0.0.14.14] Pure Rust SSH Fallback (using ssh2)
            // ssh2 is synchronous, so we run it in a blocking task
            let pass_val = pass.to_string();
            return tokio::task::spawn_blocking(move || {
                use ssh2::Session;
                use std::io::Read;
                use std::net::TcpStream;

                let tcp = TcpStream::connect_timeout(
                    &format!("{}:{}", ip_val, port_val).parse().map_err(|e| format!("Invalid Address: {}", e))?,
                    Duration::from_secs(30)
                ).map_err(|e| format!("TCP Connect Timeout (30s): {}", e))?;
                
                let mut sess = Session::new().map_err(|e| format!("SSH Session Init Failed: {}", e))?;
                sess.set_timeout(30000); // 30 seconds timeout for all operations
                sess.set_blocking(true); // Ensure blocking mode for reliable data transfer
                sess.set_tcp_stream(tcp);
                
                eprintln!("   🔐 [Internal] Initiating Handshake...");
                sess.handshake().map_err(|e| format!("SSH Handshake Failed: {}", e))?;
                
                eprintln!("   🔐 [Internal] Authenticating as '{}'...", user_val);
                sess.userauth_password(&user_val, &pass_val)
                    .map_err(|e| format!("SSH Auth Failed (Password): {}", e))?;
                
                if !sess.authenticated() {
                    return Err("SSH Authentication Failed (Unknown Reason)".to_string());
                }
                
                eprintln!("   ✅ [Internal] SSH Authenticated.");
                let mut channel = sess.channel_session().map_err(|e| format!("Channel Open Failed: {}", e))?;
                
                // 🛡️ [Milestone v0.0.14.15] Automatic Sudo Password Injection
                let final_cmd = if cmd_val.contains("sudo ") && !cmd_val.contains("-S") {
                    cmd_val.replace("sudo ", "sudo -S -p '' ")
                } else {
                    cmd_val.clone()
                };

                channel.exec(&final_cmd).map_err(|e| format!("Command Exec Failed: {}", e))?;
                
                if final_cmd.contains("sudo -S") {
                    use std::io::Write;
                    let sudo_pass = format!("{}\n", pass_val);
                    channel.write_all(sudo_pass.as_bytes()).ok();
                    channel.flush().ok();
                }

                let mut s = String::new();
                channel.read_to_string(&mut s).map_err(|e| format!("Read Failed: {}", e))?;
                channel.wait_close().ok();
                
                let exit_status = channel.exit_status().unwrap_or(0);
                if exit_status != 0 {
                    let mut err_msg = String::new();
                    channel.stderr().read_to_string(&mut err_msg).ok();
                    return Err(format!("Command Failed (Exit Code: {}): {}", exit_status, err_msg));
                }

                Ok(s)
            }).await.map_err(|e| e.to_string())?;
        }

        // --- Standard CLI Path (for Key-based Auth) ---
        let target = if user.is_some() {
            format!("{}@{}", user.unwrap(), ip)
        } else {
            ip.to_string()
        };
        let port_str = port_val.to_string();

        let output = tokio::process::Command::new("ssh")
            .args(&[
                "-o", "BatchMode=yes",
                "-o", "ConnectTimeout=10",
                "-o", "StrictHostKeyChecking=no",
                "-p", &port_str,
                &target,
                cmd,
            ])
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(stderr)
        }
    }
    pub async fn get_system_info(ip: &str) -> Result<(String, String), String> {
        let cmd = "uname -r && awk '{print $1,$2,$3}' /proc/loadavg";
        let output = Self::execute_remote_async(ip, None, None, None, cmd).await?;
        let lines: Vec<&str> = output.lines().collect();
        if lines.len() >= 2 {
            Ok((lines[0].to_string(), lines[1].to_string()))
        } else {
            Err("Failed to parse system info".to_string())
        }
    }
}
