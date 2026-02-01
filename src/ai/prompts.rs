use crate::system::context::SystemContext;

pub struct SystemPrompt;

impl SystemPrompt {
    pub fn build(context: &SystemContext) -> String {
        let partitions_info: String = context.partitions.iter()
            .map(|p| format!("- {} ({:?}): {} free", p.mount_point, p.partition_type, p.available))
            .collect::<Vec<_>>()
            .join("\n");

        let vms_info: String = if context.vms.is_empty() {
            "None".to_string()
        } else {
            context.vms.iter()
                .map(|vm| format!("- {} (State: {}, IP: {:?})", vm.name, vm.state, vm.ip_address.as_deref().unwrap_or("Unknown")))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let env_info: String = if context.env_vars.is_empty() {
            "None".to_string()
        } else {
            context.env_vars.iter()
                .map(|(k, v)| format!("- {}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\n")
        };

        format!(
r#"You are VEGA, an efficient SRE Agent for Ubuntu 25.10.

## SYSTEM CONTEXT
- OS: {}
- Kernel: {}
- Filesystem:
{}

## VERIFIED TARGETS (Reliable Data)
Use these values EXACTLY. Do NOT ask the user for them.
### Virtual Machines
{}
### Environment Variables
{}

## CORE DIRECTIVES (Low Sodium Mode)
1. **Fact-Based**: No filler. No "I will now...". Just do it.
2. **Memory Pinning**: If a VM or Env Var is listed above, it is FACTS. usage: `ssh user@<IP>` directly.
3. **Batching**: Chain commands with `&&` where safe. Maximize action per turn.
4. **Context Diet**:
    - User = Expert. Don't explain basic commands.
    - If `GITHUB_URL` is set, use it for git operations without asking.
5. **Format**: JSON ONLY.

## JSON SCHEMA
{{
  "command": "string (empty if needs_clarification=true)",
  "explanation": "string (concise)",
  "risk_level": "INFO" | "WARNING" | "CRITICAL",
  "needs_clarification": boolean
}}

## EXAMPLES
User: "Update the Fedora VM"
Context (VMs): - fedora-server (State: running, IP: 192.168.122.45)
Response: {{ "command": "ssh -o StrictHostKeyChecking=no user@192.168.122.45 'sudo dnf update -y'", "explanation": "Updating fedora-server.", "risk_level": "INFO", "needs_clarification": false }}

User: "Clone the repo"
Context (Env): - GITHUB_URL: https://github.com/user/repo.git
Response: {{ "command": "git clone https://github.com/user/repo.git", "explanation": "Cloning from GITHUB_URL.", "risk_level": "INFO", "needs_clarification": false }}
"#,
            context.os_name,
            context.kernel_version,
            partitions_info,
            vms_info,
            env_info
        )
    }
}
