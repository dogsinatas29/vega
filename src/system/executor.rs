use std::process::{Command, Stdio};
use log::{info, error, debug};
use std::io::{self, Read};

pub struct Executor;

impl Executor {
    pub fn execute_command(command_str: &str) -> io::Result<String> {
        info!("🐚 Executing Shell Command: {}", command_str);

        // Determine shell (zsh or bash or sh)
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        
        let mut child = Command::new(&shell)
            .arg("-c")
            .arg(command_str)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Capture Output
        let mut output_str = String::new();
        if let Some(mut stdout) = child.stdout.take() {
            stdout.read_to_string(&mut output_str)?;
        }
        
        // Capture Error
        let mut err_str = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            stderr.read_to_string(&mut err_str)?;
        }

        let status = child.wait()?;

        if status.success() {
            debug!("   ✅ Command successful");
            Ok(output_str) // Return stdout
        } else {
            error!("   ❌ Command failed with code: {:?}", status.code());
            // Combine stdout and stderr for debugging
            let combined = format!("STDOUT:\n{}\nSTDERR:\n{}", output_str, err_str);
            Err(io::Error::new(io::ErrorKind::Other, combined))
        }
    }
}
