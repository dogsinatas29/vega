use std::process::Command;
use crate::scan::vm::{VmScanner, VmInfo};

pub struct VmController;

impl VmController {
    pub fn start(name: &str) -> Result<String, String> {
        // 1. Check if running
        let vms = VmScanner::scan();
        if let Some(vm) = vms.iter().find(|v| v.name == name) {
            if vm.state == "running" {
                return Ok(format!("VM '{}' is already running.", name));
            }
        }

        // 2. Start
        let output = Command::new("virsh").arg("start").arg(name).output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
             Ok(format!("🚀 VM '{}' started successfully.", name))
        } else {
             Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn get_ip(name: &str) -> Option<String> {
        let vms = VmScanner::scan();
        if let Some(vm) = vms.iter().find(|v| v.name == name) {
            return vm.ip.clone();
        }
        None
    }
}
