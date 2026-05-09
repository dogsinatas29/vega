use crate::context::SystemContext;
use crate::executor::action::{Action, DangerLevel, ExecutionPlan};
use crate::executor::ExecuteResult;
use async_trait::async_trait;

pub enum PackageSystem {
    Apt,
    Dnf,
    Pacman,
    #[allow(dead_code)]
    Flatpak,
    #[allow(dead_code)]
    Unknown,
}

pub trait PackageManager {
    fn install(&self, package: &str) -> String;
    #[allow(dead_code)]
    fn update(&self) -> String;
    #[allow(dead_code)]
    fn remove(&self, package: &str) -> String;
    #[allow(dead_code)]
    fn search(&self, query: &str) -> String;
    fn name(&self) -> &str;
    fn kind(&self) -> PackageSystem;
}

fn normalize(pkg: &str, kind: &PackageSystem) -> String {
    match (pkg, kind) {
        ("obs", PackageSystem::Apt) => "obs-studio".to_string(),
        ("docker", PackageSystem::Apt) => "docker.io".to_string(), // Debian legacy
        ("docker", PackageSystem::Dnf) => "docker-ce".to_string(),
        _ => pkg.to_string(),
    }
}

pub struct Apt;
impl PackageManager for Apt {
    fn install(&self, package: &str) -> String {
        prepend_auth_sock(format!(
            "sudo apt install -y {}",
            normalize(package, &self.kind())
        ))
    }
    fn update(&self) -> String {
        prepend_auth_sock("sudo apt update && sudo apt upgrade -y".to_string())
    }
    fn remove(&self, package: &str) -> String {
        prepend_auth_sock(format!("sudo apt remove -y {}", package))
    }
    fn search(&self, query: &str) -> String {
        format!("apt search {}", query)
    }
    fn name(&self) -> &str {
        "apt (Debian/Ubuntu)"
    }
    fn kind(&self) -> PackageSystem {
        PackageSystem::Apt
    }
}

pub struct Dnf;
impl PackageManager for Dnf {
    fn install(&self, package: &str) -> String {
        prepend_auth_sock(format!(
            "sudo dnf install -y {}",
            normalize(package, &self.kind())
        ))
    }
    fn update(&self) -> String {
        prepend_auth_sock("sudo dnf update -y".to_string())
    }
    fn remove(&self, package: &str) -> String {
        prepend_auth_sock(format!("sudo dnf remove -y {}", package))
    }
    fn search(&self, query: &str) -> String {
        format!("dnf search {}", query)
    }
    fn name(&self) -> &str {
        "dnf (Fedora/RHEL)"
    }
    fn kind(&self) -> PackageSystem {
        PackageSystem::Dnf
    }
}

pub struct Pacman;
impl PackageManager for Pacman {
    fn install(&self, package: &str) -> String {
        prepend_auth_sock(format!(
            "sudo pacman -S --noconfirm {}",
            normalize(package, &self.kind())
        ))
    }
    fn update(&self) -> String {
        prepend_auth_sock("sudo pacman -Syu --noconfirm".to_string())
    }
    fn remove(&self, package: &str) -> String {
        prepend_auth_sock(format!("sudo pacman -Rns --noconfirm {}", package))
    }
    fn search(&self, query: &str) -> String {
        format!("pacman -Ss {}", query)
    }
    fn name(&self) -> &str {
        "pacman (Arch)"
    }
    fn kind(&self) -> PackageSystem {
        PackageSystem::Pacman
    }
}

pub struct AptInstall {
    pub package_name: String,
}

#[async_trait]
impl Action for AptInstall {
    fn id(&self) -> String { format!("apt-install-{}", self.package_name) }
    fn name(&self) -> String { format!("Apt: Install {}", self.package_name) }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Moderate }
    fn required_capabilities(&self) -> Vec<crate::executor::action::CapabilityRequirement> {
        vec![
            crate::executor::action::CapabilityRequirement::Disk,
            crate::executor::action::CapabilityRequirement::Ram,
        ]
    }

    async fn validate(&self, snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> {
        match snapshot.capabilities.distro {
            crate::system::snapshot::LinuxDistro::Ubuntu | crate::system::snapshot::LinuxDistro::Debian => Ok(()),
            _ => Err(format!("Target host distro {:?} is not supported by AptInstall", snapshot.capabilities.distro)),
        }
    }

    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec![format!("sudo apt update"), format!("sudo apt install -y {}", self.package_name)],
            estimated_impact: format!("Installs package '{}' via apt.", self.package_name),
            danger_level: self.danger_level(),
        })
    }

    fn build_command(&self) -> String {
        format!("sudo apt update && sudo apt install -y {}", self.package_name)
    }

    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult { success: true, status: crate::executor::ExecutionStatus::Success, stdout: String::new(), stderr: String::new(), exit_code: Some(0), error: None, insight: None })
    }

    async fn rollback(&self) -> Result<(), String> { 
        Ok(())
    }
}

pub struct DockerRun {
    pub image_name: String,
}

#[async_trait]
impl Action for DockerRun {
    fn id(&self) -> String { format!("docker-run-{}", self.image_name) }
    fn name(&self) -> String { format!("Docker: Run {}", self.image_name) }
    fn danger_level(&self) -> DangerLevel { DangerLevel::Moderate }
    fn required_capabilities(&self) -> Vec<crate::executor::action::CapabilityRequirement> {
        vec![
            crate::executor::action::CapabilityRequirement::Docker,
            crate::executor::action::CapabilityRequirement::Ram,
        ]
    }

    async fn validate(&self, snapshot: &crate::system::snapshot::HostSnapshot) -> Result<(), String> {
        if !snapshot.capabilities.has_docker { return Err("Docker not installed on target".to_string()); }
        Ok(())
    }

    async fn plan(&self) -> Result<ExecutionPlan, String> {
        Ok(ExecutionPlan {
            steps: vec![format!("docker pull {}", self.image_name), format!("docker run -d {}", self.image_name)],
            estimated_impact: format!("Pulls and runs docker image '{}'.", self.image_name),
            danger_level: self.danger_level(),
        })
    }

    fn build_command(&self) -> String {
        format!("docker pull {} && docker run -d {}", self.image_name, self.image_name)
    }

    async fn execute(&self) -> Result<ExecuteResult, String> {
        Ok(ExecuteResult { success: true, status: crate::executor::ExecutionStatus::Success, stdout: String::new(), stderr: String::new(), exit_code: Some(0), error: None, insight: None })
    }

    async fn rollback(&self) -> Result<(), String> { Ok(()) }
}

pub fn detect(ctx: &SystemContext) -> Box<dyn PackageManager> {
    match ctx.pkg_manager.as_str() {
        "apt" => Box::new(Apt),
        "dnf" => Box::new(Dnf),
        "pacman" => Box::new(Pacman),
        _ => Box::new(Apt), // Default fallback
    }
}

fn prepend_auth_sock(cmd: String) -> String {
    if let Ok(sock) = std::env::var("SSH_AUTH_SOCK") {
        return format!("SSH_AUTH_SOCK={} {}", sock, cmd);
    }
    cmd
}
