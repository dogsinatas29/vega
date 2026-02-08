use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;
use serde::Serialize;

#[derive(Serialize)]
struct LogEntry<'a> {
    timestamp: String,
    command: &'a str,
    action_type: &'a str,
    success: bool,
}

pub struct ExecutionLogger {
    path: String,
}

impl ExecutionLogger {
    pub fn new(path: &str) -> Self {
        ExecutionLogger {
            path: path.to_string(),
        }
    }

    pub fn log(&self, command: &str, action_type: &str, success: bool) {
        use std::os::unix::fs::OpenOptionsExt;

        let entry = LogEntry {
            timestamp: Local::now().to_rfc3339(),
            command,
            action_type,
            success,
        };

        if let Ok(json) = serde_json::to_string(&entry) {
            let path_obj = std::path::Path::new(&self.path);
            if let Some(parent) = path_obj.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .mode(0o600) // Security: Read/Write for owner only
                .open(&self.path) 
            {
                let _ = writeln!(file, "{}", json);
            }
        }
    }
}
