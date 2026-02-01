use rusqlite::{params, Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Database {
    conn: Connection,
    current_session_id: Option<i64>,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("vega.db")?;
        
        // Initialize Schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                total_weight INTEGER DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
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

        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_history (
                id INTEGER PRIMARY KEY,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER
            )",
            [],
        )?;

        let mut db = Database {
            conn,
            current_session_id: None,
        };

        db.start_session()?;
        
        Ok(db)
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
