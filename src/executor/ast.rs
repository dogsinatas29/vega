use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAst {
    pub tool: String,
    pub operation: String,
    pub source: Option<String>,
    pub destination: Option<String>,
    pub options: Vec<String>,
    pub target_server: Option<String>,
}

impl CommandAst {
    pub fn new(tool: &str, operation: &str) -> Self {
        Self {
            tool: tool.to_string(),
            operation: operation.to_string(),
            source: None,
            destination: None,
            options: Vec::new(),
            target_server: None,
        }
    }

    pub fn to_shell_command(&self) -> String {
        let mut cmd = format!("{} {}", self.tool, self.operation);
        
        if let Some(src) = &self.source {
            cmd.push_str(&format!(" {}", src));
        }
        
        if let Some(dst) = &self.destination {
            cmd.push_str(&format!(" {}", dst));
        }
        
        if !self.options.is_empty() {
            cmd.push_str(&format!(" {}", self.options.join(" ")));
        }
        
        cmd
    }
}
