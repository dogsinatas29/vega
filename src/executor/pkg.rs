
use crate::context::SystemContext;

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
        // Add more known aliases here
        _ => pkg.to_string(),
    }
}

#[allow(dead_code)]
pub struct Apt;
impl PackageManager for Apt {
    fn install(&self, package: &str) -> String { format!("sudo apt install -y {}", normalize(package, &self.kind())) }
    fn update(&self) -> String { "sudo apt update && sudo apt upgrade -y".to_string() }
    fn remove(&self, package: &str) -> String { format!("sudo apt remove -y {}", package) }
    fn search(&self, query: &str) -> String { format!("apt search {}", query) }
    fn name(&self) -> &str { "apt (Debian/Ubuntu)" }
    fn kind(&self) -> PackageSystem { PackageSystem::Apt }
}

#[allow(dead_code)]
pub struct Dnf;
impl PackageManager for Dnf {
    fn install(&self, package: &str) -> String { format!("sudo dnf install -y {}", normalize(package, &self.kind())) }
    fn update(&self) -> String { "sudo dnf update -y".to_string() }
    fn remove(&self, package: &str) -> String { format!("sudo dnf remove -y {}", package) }
    fn search(&self, query: &str) -> String { format!("dnf search {}", query) }
    fn name(&self) -> &str { "dnf (Fedora/RHEL)" }
    fn kind(&self) -> PackageSystem { PackageSystem::Dnf }
}

#[allow(dead_code)]
pub struct Pacman;
impl PackageManager for Pacman {
    fn install(&self, package: &str) -> String { format!("sudo pacman -S --noconfirm {}", normalize(package, &self.kind())) }
    fn update(&self) -> String { "sudo pacman -Syu --noconfirm".to_string() }
    fn remove(&self, package: &str) -> String { format!("sudo pacman -Rns --noconfirm {}", package) }
    fn search(&self, query: &str) -> String { format!("pacman -Ss {}", query) }
    fn name(&self) -> &str { "pacman (Arch)" }
    fn kind(&self) -> PackageSystem { PackageSystem::Pacman }
}

#[allow(dead_code)]
pub struct Flatpak;
impl PackageManager for Flatpak {
    fn install(&self, package: &str) -> String { format!("flatpak install -y flathub {}", package) }
    fn update(&self) -> String { "flatpak update -y".to_string() }
    fn remove(&self, package: &str) -> String { format!("flatpak uninstall -y {}", package) }
    fn search(&self, query: &str) -> String { format!("flatpak search {}", query) }
    fn name(&self) -> &str { "flatpak (Universal)" }
    fn kind(&self) -> PackageSystem { PackageSystem::Flatpak }
}

pub fn detect(ctx: &SystemContext) -> Box<dyn PackageManager> {
    match ctx.pkg_manager.as_str() {
        "apt" => Box::new(Apt),
        "dnf" => Box::new(Dnf),
        "pacman" => Box::new(Pacman),
        _ => Box::new(Apt), // Default fallback
    }
}
