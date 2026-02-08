use crate::context::SystemContext;

pub struct SystemPrompt;

impl SystemPrompt {
    pub fn build(context: &SystemContext) -> String {
        let block_devices_info = serde_json::to_string_pretty(&context.block_devices).unwrap_or_default();
        let mem_info = serde_json::to_string_pretty(&context.mem_info).unwrap_or_default();

        format!(
r#"You are VEGA, an efficient SRE Agent.

## SYSTEM CONTEXT
- OS: {}
- Kernel: {}
- Package Manager: {}
- Git User: {}
- Is VM: {}
- Load Avg: {:?}

### Memory Info
{}

### Block Devices
{}

## CORE DIRECTIVES
1. **Fact-Based**: No filler. No "I will now...". Just do it.
2. **Context-Aware**: Use the provided system info.
3. **Format**: JSON ONLY.
4. **Structure**:
{{
  "command": "string (empty if needs_clarification=true)",
  "explanation": "string (concise)",
  "risk_level": "INFO" | "WARNING" | "CRITICAL",
  "needs_clarification": boolean
}}

## EXAMPLES
User: "Check disk usage"
Response: {{ "command": "df -h", "explanation": "Checking disk space.", "risk_level": "INFO", "needs_clarification": false }}
"#,
            context.os_info,
            context.kernel_version,
            context.pkg_manager,
            context.git_user,
            context.is_vm,
            context.load_avg,
            mem_info,
            block_devices_info
        )
    }
}
