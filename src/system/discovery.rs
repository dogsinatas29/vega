use std::process::Command;

#[allow(dead_code)]
pub const DISCOVERY_SCRIPT: &str = "/home/dogsinatas/python_project2/src/utils/discovery.py";

#[allow(dead_code)]
pub struct Discovery;

#[allow(dead_code)]
impl Discovery {
    /// Runs the discovery script as a subprocess
    pub fn run() -> Result<(), String> {
        eprintln!("⚠️  IP Unknown. Running automatic discovery...");
        
        let output = Command::new("python3")
            .arg(DISCOVERY_SCRIPT)
            .output()
            .map_err(|e| format!("Failed to execute discovery script: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("❌ Discovery script failed: {}", stderr);
            return Err("Discovery failed".to_string());
        }
        
        // Print discovery output
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprint!("{}", stdout);
        
        eprintln!("✅ Discovery completed successfully.");
        Ok(())
    }
}
