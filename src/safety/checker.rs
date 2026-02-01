#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RiskLevel {
    Info,
    Warning,
    Critical,
}

pub fn check_risk_level(command: &str) -> RiskLevel {
    let cmd = command.trim();

    // Critical: Destructive Data Loss
    if cmd.contains("rm -rf") || cmd.contains("mkfs") || cmd.contains("dd if=") || cmd.contains("> /dev/sd") {
        return RiskLevel::Critical;
    }

    // Warning: System Modification or Process Killing
    if cmd.contains("chmod 777") || cmd.contains("kill -9") || cmd.contains("shutdown") || cmd.contains("reboot") || cmd.contains("systemctl stop") {
        return RiskLevel::Warning;
    }

    // Warning: Package Managers (can be messy)
    if cmd.starts_with("apt remove") || cmd.starts_with("dnf remove") {
        return RiskLevel::Warning;
    }

    RiskLevel::Info
}
