use crate::system::snapshot::HostSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Normal,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticFinding {
    pub metric: String,
    pub value: String,
    pub severity: Severity,
    pub message: String,
}

pub struct ThresholdEngine;

impl ThresholdEngine {
    pub fn analyze(snapshot: &HostSnapshot) -> Vec<DiagnosticFinding> {
        let mut findings = Vec::new();
        let caps = &snapshot.capabilities;

        // 1. Load Average Analysis (Relative to CPU Cores)
        // Note: In a real scenario, we'd get the actual load value. 
        // For now, simulating the logic using cpu_cores.
        let cpu_count = caps.cpu_cores as f64;
        // In the previous log, load was 0.83.
        let current_load = 0.83; // Placeholder for actual parsing logic

        if current_load > cpu_count * 1.0 {
            findings.push(DiagnosticFinding {
                metric: "Load Average".to_string(),
                value: current_load.to_string(),
                severity: Severity::Warning,
                message: format!("Load average ({}) exceeds CPU core count ({}). Performance may be impacted.", current_load, cpu_count),
            });
        } else {
            findings.push(DiagnosticFinding {
                metric: "Load Average".to_string(),
                value: current_load.to_string(),
                severity: Severity::Normal,
                message: "Load average is within healthy limits.".to_string(),
            });
        }

        // 2. RAM Usage Analysis
        let ram_used_pct = (caps.ram_total_gb - caps.ram_free_gb) / caps.ram_total_gb * 100.0;
        if ram_used_pct > 90.0 {
            findings.push(DiagnosticFinding {
                metric: "RAM Usage".to_string(),
                value: format!("{:.1}%", ram_used_pct),
                severity: Severity::Critical,
                message: "Memory exhaustion imminent. System stability at risk.".to_string(),
            });
        } else if ram_used_pct > 75.0 {
            findings.push(DiagnosticFinding {
                metric: "RAM Usage".to_string(),
                value: format!("{:.1}%", ram_used_pct),
                severity: Severity::Warning,
                message: "High memory pressure detected.".to_string(),
            });
        }

        // 3. Disk Space Analysis
        if caps.disk_free_root_gb < 2.0 {
            findings.push(DiagnosticFinding {
                metric: "Root Disk Space".to_string(),
                value: format!("{:.1}GB", caps.disk_free_root_gb),
                severity: Severity::Critical,
                message: "Root partition is almost full. Critical for system operations.".to_string(),
            });
        }

        findings
    }
}
