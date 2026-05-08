use std::process::Command;
use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

pub struct SshConnection;

impl SshConnection {
    /// Verifies if the port is actually an SSH server by checking the protocol banner.
    pub fn verify_ssh_fingerprint(ip: &str, port: u16) -> bool {
        let addr = crate::connection::NetworkAddress::new(ip, port);
        if let Ok(socket_addr) = addr.to_socket_addr() {
            if let Ok(mut stream) = TcpStream::connect_timeout(&socket_addr, Duration::from_millis(500)) {
                let mut buffer = [0; 8];
                if stream.set_read_timeout(Some(Duration::from_millis(500))).is_ok() {
                    if stream.read_exact(&mut buffer).is_ok() {
                        let banner = String::from_utf8_lossy(&buffer);
                        return banner.starts_with("SSH-2.0-");
                    }
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
        use tokio_stream::StreamExt;
        let mut stream = Self::execute_remote_streaming(ip, user, port, password, cmd).await?;
        let mut full_output = String::new();
        while let Some(chunk) = stream.next().await {
            full_output.push_str(&chunk);
        }
        Ok(full_output)
    }

    pub async fn execute_remote_streaming(
        ip: &str,
        user: Option<&str>,
        port: Option<u16>,
        password: Option<&str>,
        cmd: &str,
    ) -> Result<tokio_stream::wrappers::ReceiverStream<String>, String> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let port_val = port.unwrap_or(22);
        let user_val = user.unwrap_or("root").to_string();
        let cmd_val = cmd.to_string();
        let ip_val = ip.to_string();

        if let Some(pass) = password {
            let pass_val = pass.to_string();
            tokio::task::spawn_blocking(move || {
                use ssh2::Session;
                use std::io::Read;
                use std::net::TcpStream;

                let addr = crate::connection::NetworkAddress::new(&ip_val, port_val);
                let socket_addr = match addr.to_socket_addr() {
                    Ok(a) => a,
                    Err(e) => { let _ = tx.blocking_send(format!("Error: {}", e)); return; }
                };

                let tcp = match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(30)) {
                    Ok(t) => t,
                    Err(e) => { let _ = tx.blocking_send(format!("Error: {}", e)); return; }
                };
                
                let mut sess = Session::new().unwrap();
                sess.set_tcp_stream(tcp);
                if let Err(e) = sess.handshake() { let _ = tx.blocking_send(format!("Error: {}", e)); return; }
                if let Err(e) = sess.userauth_password(&user_val, &pass_val) { let _ = tx.blocking_send(format!("Error: {}", e)); return; }
                
                let mut channel = sess.channel_session().unwrap();
                if let Err(e) = channel.exec(&cmd_val) { let _ = tx.blocking_send(format!("Error: {}", e)); return; }
                
                let mut buffer = [0; 1024];
                loop {
                    match channel.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            let s = String::from_utf8_lossy(&buffer[..n]).to_string();
                            if tx.blocking_send(s).is_err() { break; }
                        }
                        Err(_) => break,
                    }
                    // Handle stderr separately if possible in libssh2, 
                    // but for now we focus on standardizing CLI path
                }
                let _ = channel.wait_close();
            });
            return Ok(tokio_stream::wrappers::ReceiverStream::new(rx));
        }

        // CLI Path
        let addr = crate::connection::NetworkAddress::new(ip, port_val);
        let target = if let Some(u) = user { format!("{}@{}", u, addr.host) } else { addr.host.clone() };
        let port_str = addr.port.to_string();

        println!("📡 [SSH] CMD: ssh -o BatchMode=yes -o ConnectTimeout=10 -o StrictHostKeyChecking=no -p {} {} '...' ", port_str, target);

        let mut child = tokio::process::Command::new("ssh")
            .args(&["-o", "BatchMode=yes", "-o", "ConnectTimeout=10", "-o", "StrictHostKeyChecking=no", "-p", &port_str, &target, cmd])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut out_buffer = [0; 1024];
            let mut err_buffer = [0; 1024];
            let mut out = stdout;
            let mut err = stderr;
            
            loop {
                tokio::select! {
                    res = out.read(&mut out_buffer) => {
                        match res {
                            Ok(0) => break,
                            Ok(n) => {
                                let s = String::from_utf8_lossy(&out_buffer[..n]).to_string();
                                if tx.send(s).await.is_err() { break; }
                            }
                            Err(_) => break,
                        }
                    }
                    res = err.read(&mut err_buffer) => {
                        match res {
                            Ok(0) => {}, // keep reading stdout
                            Ok(n) => {
                                let s = String::from_utf8_lossy(&err_buffer[..n]).to_string();
                                eprint!("{}", s); // Visible to user, but doesn't pollute data stream
                            }
                            Err(_) => break,
                        }
                    }
                }
            }
            
            // 🚨 Check Exit Status to distinguish Transport Failure
            match child.wait().await {
                Ok(status) if !status.success() => {
                    let code = status.code().unwrap_or(-1);
                    eprintln!("❌ [SSH] Transport Failure (Exit Code: {})", code);
                    // We don't send to tx here, snapshot.rs will see empty/malformed data and report Protocol Error
                    // But we've already printed the specific SSH error to stderr.
                }
                _ => {}
            }
        });

        Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
    }

    pub async fn execute_remote_full(
        ip: &str,
        user: Option<&str>,
        port: Option<u16>,
        password: Option<&str>,
        cmd: &str,
    ) -> Result<crate::executor::ExecuteResult, String> {
        let port_val = port.unwrap_or(22);
        let user_val = user.unwrap_or("root").to_string();
        let target = format!("{}@{}", user_val, ip);
        let port_str = port_val.to_string();

        println!("📡 [SSH] CMD: ssh -o BatchMode=yes -o ConnectTimeout=10 -o StrictHostKeyChecking=no -p {} {} '{}' ", port_str, target, cmd);

        if let Some(pass) = password {
            // Password based path (libssh2)
            use ssh2::Session;
            use std::io::Read;
            use std::net::TcpStream;

            let ip_val = ip.to_string();
            let user_val = user_val.clone();
            let pass_val = pass.to_string();
            let cmd_val = cmd.to_string();

            tokio::task::spawn_blocking(move || {
                let addr = crate::connection::NetworkAddress::new(&ip_val, port_val);
                let socket_addr = addr.to_socket_addr().map_err(|e| e.to_string())?;
                let tcp = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(30)).map_err(|e| e.to_string())?;
                
                let mut sess = Session::new().unwrap();
                sess.set_tcp_stream(tcp);
                sess.handshake().map_err(|e| e.to_string())?;
                sess.userauth_password(&user_val, &pass_val).map_err(|e| e.to_string())?;
                
                let mut channel = sess.channel_session().map_err(|e| e.to_string())?;
                
                // [Stable Code Protection 적용: 오직 디버깅 스트림만 확보]
                println!("\n⚡ [RAW SSH TRANSPORT DEBUG] ------------------");
                println!("   - Host: {}", ip_val);
                println!("   - Command: '{}'", cmd_val);
                println!("---------------------------------------------");

                channel.exec(&cmd_val).map_err(|e| e.to_string())?;
                
                let mut stdout_buf = String::new();
                match channel.read_to_string(&mut stdout_buf) {
                    Ok(_) => println!("🟢 [STDOUT]\n{}", stdout_buf),
                    Err(e) => println!("🔴 [STDOUT_READ_ERR] {:?}", e),
                }
                
                let mut stderr_buf = String::new();
                match channel.stderr().read_to_string(&mut stderr_buf) {
                    Ok(_) => println!("🔴 [STDERR]\n{}", stderr_buf),
                    Err(e) => println!("🔴 [STDERR_READ_ERR] {:?}", e),
                }
                
                let _ = channel.wait_close();
                let exit_status = channel.exit_status().unwrap_or(-1);
                println!("🟡 [EXIT CODE] {}", exit_status);
                println!("---------------------------------------------\n");
                
                Ok(crate::executor::ExecuteResult {
                    success: exit_status == 0,
                    stdout: stdout_buf,
                    stderr: stderr_buf,
                    exit_code: Some(exit_status),
                })
            }).await.map_err(|e| e.to_string())?
        } else {
            // CLI based path
            // [Stable Code Protection 적용: 오직 디버깅 스트림만 확보]
            println!("\n⚡ [RAW SSH TRANSPORT DEBUG (CLI)] ------------");
            println!("   - Host: {}", target);
            println!("   - Command: '{}'", cmd);
            println!("---------------------------------------------");

            let output = tokio::process::Command::new("ssh")
                .args(&["-o", "BatchMode=yes", "-o", "ConnectTimeout=10", "-o", "StrictHostKeyChecking=no", "-p", &port_str, &target, cmd])
                .output()
                .await
                .map_err(|e| e.to_string())?;

            let stdout_buf = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr_buf = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_status = output.status.code().unwrap_or(-1);

            println!("🟢 [STDOUT]\n{}", stdout_buf);
            println!("🔴 [STDERR]\n{}", stderr_buf);
            println!("🟡 [EXIT CODE] {}", exit_status);
            println!("---------------------------------------------\n");

            Ok(crate::executor::ExecuteResult {
                success: output.status.success(),
                stdout: stdout_buf,
                stderr: stderr_buf,
                exit_code: Some(exit_status),
            })
        }
    }

    pub async fn get_system_info(ip: &str) -> Result<(String, String), String> {
        let cmd = "uname -r && awk '{print $1,$2,$3}' /proc/loadavg";
        let res = Self::execute_remote_full(ip, None, None, None, cmd).await?;
        let lines: Vec<&str> = res.stdout.lines().collect();
        if lines.len() >= 2 {
            Ok((lines[0].to_string(), lines[1].to_string()))
        } else {
            Err(format!("Failed to parse system info. Stderr: {}", res.stderr))
        }
    }
}
