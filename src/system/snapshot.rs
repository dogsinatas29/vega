use serde::{Deserialize, Serialize};
use crate::connection::ssh::SshConnection;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinuxDistro {
    Ubuntu,
    Debian,
    Fedora,
    Arch,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CpuArch {
    X86_64,
    Aarch64,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostCapabilities {
    pub distro: LinuxDistro,
    pub arch: CpuArch,
    pub cpu_cores: u32,
    pub ram_total_gb: f64,
    pub ram_free_gb: f64,
    pub vram_total_gb: Option<f64>,
    pub has_nvidia: bool,
    pub has_ollama: bool,
    pub ollama_version: Option<String>,
    pub has_docker: bool,
    pub disk_free_root_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostSnapshot {
    pub hostname: String,
    pub ip: String,
    pub capabilities: HostCapabilities,
    pub timestamp: i64,
}

impl HostSnapshot {
    pub fn dummy() -> Self {
        Self {
            hostname: "dummy".to_string(),
            ip: "0.0.0.0".to_string(),
            capabilities: HostCapabilities {
                distro: LinuxDistro::Unknown("dummy".to_string()),
                arch: CpuArch::Unknown("dummy".to_string()),
                cpu_cores: 0,
                ram_total_gb: 0.0,
                ram_free_gb: 0.0,
                vram_total_gb: None,
                has_nvidia: false,
                has_ollama: true, // Optimistic for skip_snapshot actions
                ollama_version: None,
                has_docker: false,
                disk_free_root_gb: 0.0,
            },
            timestamp: 0,
        }
    }
    pub fn collect_local() -> Self {
        let ctx = crate::context::SystemContext::collect(true);
        let hostname = ctx.hostname.clone();
        
        // Extract RAM in GB
        let ram_total = ctx.mem_info.get("MemTotal")
            .and_then(|s| s.as_str())
            .and_then(|s| s.replace("kB", "").trim().parse::<f64>().ok())
            .map(|k| k / 1024.0 / 1024.0)
            .unwrap_or(0.0);

        let ram_free = ctx.mem_info.get("MemAvailable")
            .and_then(|s| s.as_str())
            .and_then(|s| s.replace("kB", "").trim().parse::<f64>().ok())
            .map(|k| k / 1024.0 / 1024.0)
            .unwrap_or(0.0);

        // Extract Root Disk in GB
        let disk_gb = ctx.partitions.iter()
            .find(|p| p.partition_type == crate::context::PartitionType::Root)
            .and_then(|p| p.available.replace("G", "").parse::<f64>().ok())
            .unwrap_or(0.0);

        HostSnapshot {
            hostname,
            ip: "127.0.0.1".to_string(),
            capabilities: HostCapabilities {
                distro: match ctx.pkg_manager.as_str() {
                    "apt" => LinuxDistro::Ubuntu,
                    "dnf" => LinuxDistro::Fedora,
                    "pacman" => LinuxDistro::Arch,
                    _ => LinuxDistro::Unknown(ctx.os_name),
                },
                arch: CpuArch::X86_64, // Simplified for now
                cpu_cores: std::thread::available_parallelism().map(|n| n.get() as u32).unwrap_or(1),
                ram_total_gb: ram_total,
                ram_free_gb: ram_free,
                vram_total_gb: None, // Logic for local GPU can be added later
                has_nvidia: false,
                has_ollama: ctx.remotes.iter().any(|r| r.name.contains("ollama")), // Rough check
                ollama_version: std::process::Command::new("ollama")
                    .arg("--version")
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.split_whitespace().last().unwrap_or("").to_string()),
                has_docker: false,
                disk_free_root_gb: disk_gb,
            },
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub async fn collect_remote(
        target: &str, 
        user: Option<&str>, 
        port: Option<u16>, 
        pass: Option<&str>,
        requirements: &[crate::executor::action::CapabilityRequirement]
    ) -> Result<Self, String> {
        use crate::executor::action::CapabilityRequirement;

        // JSON Framing: Use sentinel tokens to isolate data from terminal noise (MOTD, shell messages)
        let mut script = String::from("printf '---VEGA_JSON_BEGIN---\\n{");
        script.push_str("\"hostname\":\"$(hostname)\",");
        script.push_str("\"arch\":\"$(uname -m)\",");
        script.push_str("\"distro\":\"$(cat /etc/os-release | grep ^ID= | cut -d= -f2 | tr -d '\"')\"");

        for req in requirements {
            match req {
                CapabilityRequirement::Cpu => script.push_str(",\"cores\":$(nproc)"),
                CapabilityRequirement::Ram => {
                    script.push_str(",\"ram_total\":$(grep MemTotal /proc/meminfo | awk '{print $2/1024/1024}')");
                    script.push_str(",\"ram_free\":$(grep MemAvailable /proc/meminfo | awk '{print $2/1024/1024}')");
                },
                CapabilityRequirement::Disk => script.push_str(",\"disk_free\":$(df -BG / | tail -1 | awk '{print $4}' | tr -d 'G')"),
                CapabilityRequirement::Gpu => script.push_str(",\"vram\":$( (command -v nvidia-smi >/dev/null && nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | awk '{sum+=$1} END {print sum/1024}') || echo \"0\" )"),
                CapabilityRequirement::Ollama => {
                    script.push_str(",\"ollama\":$( (command -v ollama >/dev/null && echo \"true\") || echo \"false\" )");
                    script.push_str(",\"ollama_ver\":\"$( (command -v ollama >/dev/null && ollama --version | awk '{print $NF}') || echo \"none\" )\"");
                },
                CapabilityRequirement::Docker => script.push_str(",\"docker\":$( (command -v docker >/dev/null && echo \"true\") || echo \"false\" )"),
            }
        }
        script.push_str("}\\n---VEGA_JSON_END---\\n'");

        let output = match SshConnection::execute_remote_async(target, user, port, pass, &script).await {
            Ok(out) => {
                if out.is_empty() {
                    // This could be a silent failure or auth error not captured by Result::Err
                    return Err(format!("🔌 Transport Error: Empty response from {}. Check if SSH connection was successful (BatchMode might have failed silently).", target));
                }
                out
            },
            Err(e) => {
                // Transport Layer Error (Auth, Timeout, Unreachable)
                if e.contains("Permission denied") {
                    return Err(format!(
                        "🔑 Transport Error: SSH Authentication Failed for {}.\n   💡 Recommendation: Run 'ssh-copy-id {}' to authorize your public key.",
                        target,
                        if let Some(u) = user { format!("{}@{}", u, target) } else { target.to_string() }
                    ));
                } else if e.contains("Connection refused") || e.contains("timeout") {
                    return Err(format!("🔌 Transport Error: Host {} unreachable or connection refused.", target));
                } else {
                    return Err(format!("🔌 Transport Error: SSH execution failed: {}", e));
                }
            }
        };
        
        // Protocol Layer: Only attempt extraction if transport was successful
        let json_start_marker = "---VEGA_JSON_BEGIN---\n";
        let json_end_marker = "\n---VEGA_JSON_END---";
        
        let start = output.find(json_start_marker).map(|idx| idx + json_start_marker.len());
        let end = output.find(json_end_marker);

        let json_str = match (start, end) {
            (Some(s), Some(e)) if s < e => &output[s..e],
            _ => {
                // If transport was OK but framing is missing, it's a true Protocol Error (e.g. bashrc contamination)
                return Err(format!("📜 Protocol Error: Sentinel framing missing in response from {}. The remote host might be outputting non-JSON data on stdout.", target));
            }
        };

        #[derive(Deserialize)]
        struct RemoteData {
            hostname: String,
            arch: String,
            distro: String,
            cores: Option<u32>,
            ram_total: Option<f64>,
            ram_free: Option<f64>,
            disk_free: Option<f64>,
            vram: Option<f64>,
            ollama: Option<bool>,
            ollama_ver: Option<String>,
            docker: Option<bool>,
        }

        let data: RemoteData = serde_json::from_str(json_str).map_err(|e| format!("JSON Parse Error: {}. Raw: {}", e, json_str))?;

        let arch = match data.arch.as_str() {
            "x86_64" => CpuArch::X86_64,
            "aarch64" => CpuArch::Aarch64,
            _ => CpuArch::Unknown(data.arch),
        };

        let distro = match data.distro.as_str() {
            "ubuntu" => LinuxDistro::Ubuntu,
            "debian" => LinuxDistro::Debian,
            "fedora" => LinuxDistro::Fedora,
            "arch" => LinuxDistro::Arch,
            _ => LinuxDistro::Unknown(data.distro),
        };

        Ok(HostSnapshot {
            hostname: data.hostname,
            ip: target.to_string(),
            capabilities: HostCapabilities {
                distro,
                arch,
                cpu_cores: data.cores.unwrap_or(0),
                ram_total_gb: data.ram_total.unwrap_or(0.0),
                ram_free_gb: data.ram_free.unwrap_or(0.0),
                vram_total_gb: if data.vram.unwrap_or(0.0) > 0.0 { data.vram } else { None },
                has_nvidia: data.vram.unwrap_or(0.0) > 0.0,
                has_ollama: data.ollama.unwrap_or(false),
                ollama_version: if data.ollama_ver.as_deref() == Some("none") { None } else { data.ollama_ver },
                has_docker: data.docker.unwrap_or(false),
                disk_free_root_gb: data.disk_free.unwrap_or(0.0),
            },
            timestamp: chrono::Utc::now().timestamp(),
        })
    }
}
