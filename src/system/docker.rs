
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub id: String,
    pub image: String,
    pub name: String,
    pub status: String, // Up 2 hours
    pub state: String,  // running
}

pub struct DockerManager;

impl DockerManager {
    /// Lists all active containers
    pub fn list_containers() -> Vec<Container> {
        // docker ps --format '{{json .}}'
        let output = Command::new("docker")
            .args(&["ps", "--format", "{{json .}}"]) 
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut containers = Vec::new();

                for line in stdout.lines() {
                     match serde_json::from_str::<serde_json::Value>(line) {
                         Ok(json) => {
                             // Docker JSON keys are tricky (e.g. "ID", "Image", "Names", "Status", "State")
                             // Note: Check actual format. Standard Format is usually Title Case.
                             let id = json["ID"].as_str().unwrap_or("").to_string();
                             let image = json["Image"].as_str().unwrap_or("").to_string();
                             let name = json["Names"].as_str().unwrap_or("").to_string();
                             let status = json["Status"].as_str().unwrap_or("").to_string();
                             let state = json["State"].as_str().unwrap_or("").to_string();
                             
                             if !id.is_empty() {
                                 containers.push(Container { id, image, name, status, state });
                             }
                         },
                         Err(_) => {}
                     }
                }
                containers
            },
            Err(_) => Vec::new(), // Docker might not be installed
        }
    }
}
