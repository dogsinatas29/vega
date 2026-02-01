use ssh2::Session;
use std::net::TcpStream;
use std::io::Read;
use log::{info, debug};

pub struct RemoteManager {
    session: Option<Session>,
}

impl RemoteManager {
    pub fn new() -> Self {
        Self { session: None }
    }

    pub fn connect(&mut self, host: &str, user: &str, _key_path: Option<&str>) -> Result<(), String> {
        info!("🔌 Connecting to remote host: {}@{}", user, host);
        
        // 1. TCP Connection
        let tcp = TcpStream::connect(format!("{}:22", host))
            .map_err(|e| format!("TCP connection failed: {}", e))?;
        
        // 2. SSH Session
        let mut session = Session::new()
            .map_err(|e| format!("Session creation failed: {}", e))?;
        
        session.set_tcp_stream(tcp);
        session.handshake()
            .map_err(|e| format!("SSH handshake failed: {}", e))?;

        // 3. Authentication (Simplified: Agent or Password)
        // Ideally we'd use the key_path, but for now let's try agent first
        if let Err(_) = session.userauth_agent(user) {
            debug!("   Agent auth failed, falling back to none (might fail)");
             // Real implementation would handle keys/passwords here
        }

        if !session.authenticated() {
            return Err("Authentication failed".to_string());
        }

        info!("   ✅ SSH Connection established!");
        self.session = Some(session);
        Ok(())
    }

    pub fn exec_command(&self, command: &str) -> Result<String, String> {
        if let Some(session) = &self.session {
            debug!("   🚀 Executing remote command: {}", command);
            
            let mut channel = session.channel_session()
                .map_err(|e| format!("Channel open failed: {}", e))?;
            
            channel.exec(command)
                .map_err(|e| format!("Command exec failed: {}", e))?;
            
            let mut s = String::new();
            channel.read_to_string(&mut s)
                .map_err(|e| format!("Read failed: {}", e))?;
            
            channel.wait_close().ok();
            debug!("   ✅ Remote command finished");
            return Ok(s);
        }
        
        Err("No active session".to_string())
    }
}
