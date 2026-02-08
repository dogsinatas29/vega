use rusqlite::{params, Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use std::os::unix::fs::PermissionsExt;
use std::fs;

pub struct Database {
    conn: Connection,
    current_session_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct TaskEntry {
    pub project_name: Option<String>,
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub healer_used: bool,
    pub healer_log: Option<String>,
    pub token_usage: Option<i32>,
    pub timestamp: i64,
}

impl Database {
    pub fn get_current_session_id(&self) -> Option<i64> {
        self.current_session_id
    }

    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let db_path = config_dir.join("vega").join("vega.db");
        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&db_path)?;
        
        // Security: Enforce 600 permissions
        if let Ok(metadata) = fs::metadata(&db_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600);
            let _ = fs::set_permissions(&db_path, perms);
        }

        let mut db = Database {
            conn,
            current_session_id: None,
        };
        db.migrate()?;
        db.start_session()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute(
            "PRAGMA foreign_keys = ON;",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                total_weight INTEGER DEFAULT 0
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS commands (
                id INTEGER PRIMARY KEY,
                session_id INTEGER,
                command TEXT NOT NULL,
                ai_comment TEXT,
                weight INTEGER,
                timestamp INTEGER,
                success BOOLEAN,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_history (
                id INTEGER PRIMARY KEY,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS execution_logs (
                id INTEGER PRIMARY KEY,
                session_id INTEGER,
                command TEXT NOT NULL,
                success BOOLEAN,
                exit_code INTEGER,
                stdout TEXT,
                stderr TEXT,
                healer_intervention TEXT,
                timestamp INTEGER,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            )",
            [],
        )?;

        // Advanced Schema (Phase 3-4-1)
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS task_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER,
                project_name TEXT,
                command TEXT NOT NULL,
                exit_code INTEGER,
                stdout TEXT,
                stderr TEXT,
                healer_used BOOLEAN,
                healer_log TEXT,
                token_usage INTEGER,
                timestamp INTEGER,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS error_solutions (
                error_pattern TEXT PRIMARY KEY,
                solution_cmd TEXT,
                success_count INTEGER DEFAULT 1
            )",
            [],
        )?;

        // Phase 4: Advanced Metrics
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS system_stats (
                date TEXT PRIMARY KEY,
                total_input_tokens INTEGER DEFAULT 0,
                total_output_tokens INTEGER DEFAULT 0,
                error_count INTEGER DEFAULT 0
            )",
            [],
        )?;

    // Migration logic moved to migrate()
    Ok(()) 
    }

    pub fn start_session(&mut self) -> Result<()> {
        let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        
        self.conn.execute(
            "INSERT INTO sessions (start_time) VALUES (?)",
            params![start_time],
        )?;

        self.current_session_id = Some(self.conn.last_insert_rowid());
        Ok(())
    }

    pub fn log_command(&self, command: &str, ai_comment: &str, success: bool) -> Result<()> {
        if let Some(session_id) = self.current_session_id {
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
            let weight = calculate_weight(command);

            if let Err(e) = self.conn.execute(
                "INSERT INTO commands (session_id, command, ai_comment, weight, timestamp, success)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![session_id, command, ai_comment, weight, timestamp, success],
            ) {
                eprintln!("⚠️ Failed to log command: {}", e);
            } else {
                 // Update session total weight only if insert succeeded
                if let Err(e) = self.conn.execute(
                    "UPDATE sessions SET total_weight = total_weight + ? WHERE id = ?",
                    params![weight, session_id],
                ) {
                    eprintln!("⚠️ Failed to update session weight: {}", e);
                }
            }
        }
        Ok(())
    }

    pub fn save_chat_message(&self, role: &str, content: &str) -> Result<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        self.conn.execute(
            "INSERT INTO chat_history (role, content, timestamp) VALUES (?, ?, ?)",
            params![role, content, timestamp],
        )?;
        Ok(())
    }

    pub fn add_execution_log(&self, session_id: i64, command: &str, success: bool, exit_code: i32, stdout: &str, stderr: &str, healer_intervention: Option<&str>) -> Result<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        self.conn.execute(
            "INSERT INTO execution_logs (session_id, command, success, exit_code, stdout, stderr, healer_intervention, timestamp)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![session_id, command, success, exit_code, stdout, stderr, healer_intervention, timestamp],
        )?;
        Ok(())
    }

    pub fn get_recent_history(&self, limit: usize) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT role, content FROM chat_history ORDER BY id DESC LIMIT ?"
        )?;
        
        let history_iter = stmt.query_map(params![limit], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let mut history = Vec::new();
        for msg in history_iter {
            history.push(msg?);
        }
        history.reverse(); // Return in chronological order
        Ok(history)
    }

    pub fn get_failure_count(&self, command: &str) -> Result<i32> {
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM execution_logs WHERE command = ? AND success = 0"
        )?;
        let count: i32 = stmt.query_row(params![command], |row| row.get(0))?;
        Ok(count)
    }

    // --- Phase 3-4-1 Methods ---

    pub fn log_task(
        &self, 
        session_id: i64, 
        project_name: Option<&str>, 
        command: &str, 
        exit_code: i32, 
        stdout: &str, 
        stderr: &str, 
        healer_used: bool, 
        healer_log: Option<&str>, 
        token_usage: Option<i32>
    ) -> Result<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        self.conn.execute(
            "INSERT INTO task_history (session_id, project_name, command, exit_code, stdout, stderr, healer_used, healer_log, token_usage, timestamp)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                session_id, 
                project_name, 
                command, 
                exit_code, 
                stdout, 
                stderr, 
                healer_used, 
                healer_log, 
                token_usage, 
                timestamp
            ],
        )?;
        Ok(())
    }

    pub fn learn_solution(&self, error_pattern: &str, solution_cmd: &str) -> Result<()> {
        // Upsert logic: if exists, increment success_count
        let _rows_affected = self.conn.execute(
            "INSERT INTO error_solutions (error_pattern, solution_cmd, success_count)
             VALUES (?, ?, 1)
             ON CONFLICT(error_pattern) DO UPDATE SET success_count = success_count + 1",
             params![error_pattern, solution_cmd]
        )?;
        Ok(())
    }

    pub fn get_solution(&self, error_pattern: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT solution_cmd FROM error_solutions WHERE error_pattern = ?"
        )?;
        
        // Handling strict exact match for now. Regex matching in SQLite is harder without extensions.
        // We'll assume the 'error_pattern' passed here is a known key.
        // Or in the future, we iterate/search. 
        let mut rows = stmt.query(params![error_pattern])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn record_task(
        &self,
        project: Option<&str>,
        cmd: &str,
        res: &crate::executor::ExecuteResult,
        tokens: Option<i32>
    ) -> Result<()> {
        if let Some(sid) = self.current_session_id {
            self.log_task(
                 sid,
                 project,
                 cmd,
                 res.exit_code.unwrap_or(-1),
                 &res.stdout,
                 &res.stderr,
                 false,
                 None,
                 tokens
            )?;
        }
        Ok(())
    }

    pub fn get_session_tasks(&self, session_id: i64) -> Result<Vec<TaskEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT project_name, command, exit_code, stdout, stderr, healer_used, healer_log, token_usage, timestamp 
             FROM task_history 
             WHERE session_id = ? 
             ORDER BY timestamp ASC"
        )?;
        
        let task_iter = stmt.query_map(params![session_id], |row| {
            Ok(TaskEntry {
                project_name: row.get(0)?,
                command: row.get(1)?,
                exit_code: row.get(2)?,
                stdout: row.get(3)?,
                stderr: row.get(4)?,
                healer_used: row.get(5)?,
                healer_log: row.get(6)?,
                token_usage: row.get(7)?,
                timestamp: row.get(8)?,
            })
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    // --- Metadata Helper Methods ---

    pub fn set_metadata(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO metadata (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_metadata(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM metadata WHERE key = ?")?;
        let mut rows = stmt.query(params![key])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }
}

fn calculate_weight(command: &str) -> i32 {
    let cmd = command.trim();
    
    if cmd.starts_with("apt") || cmd.starts_with("dnf") || cmd.starts_with("pacman") {
        return 5;
    }
    
    if cmd.starts_with("systemctl") || cmd.starts_with("service") {
        return 7;
    }

    if cmd.contains("rm -rf") || cmd.contains("mkfs") || cmd.contains("dd") {
        return 20; // High risk/impact
    }

    if cmd.starts_with("ls") || cmd.starts_with("cd") || cmd.starts_with("pwd") || cmd.starts_with("echo") {
        return 1;
    }

    // Default for other commands
    3
}
