use std::process::{Command, Stdio};
use std::io::Write;

pub struct Interactor;

impl Interactor {
    pub fn select_with_fzf(prompt: &str, items: Vec<String>, query: Option<&str>) -> Option<String> {
        // Spawn fzf process
        let mut cmd = Command::new("fzf");
        cmd.arg("--header").arg(prompt).arg("--reverse");
        
        if let Some(q) = query {
            // Pre-fill query
            cmd.arg("--query").arg(q);
             // Auto-select if there is exactly one match? Optional.
        }

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .ok()?;

        // Pipe items to fzf stdin
        if let Some(stdin) = child.stdin.as_mut() {
            let input = items.join("\n");
            stdin.write_all(input.as_bytes()).ok()?;
        }

        // Wait for selection
        let output = child.wait_with_output().ok()?;
        
        if output.status.success() {
            let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if selection.is_empty() {
                None
            } else {
                Some(selection)
            }
        } else {
            None
        }
    }
}
