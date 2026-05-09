use crate::context::SystemContext;
use crate::ai::Domain;

pub const DOMAIN_CLASSIFIER_PROMPT: &str = r#"
You are the VEGA Domain Classifier.
Your role is to categorize the user's intent into one of the following DOMAINS.

### 🏗️ DOMAINS
- System: System status, diagnostics, power management (shutdown, reboot), OS updates.
- AiModels: Managing AI models (Ollama list, pull, remove, run).
- Infrastructure: SSH connections, remote node management.
- PackageManagement: Installing or removing software packages (apt, docker).

### WORLD STATE
{WORLD_STATE}

### ⚖️ RULES
1. Output ONLY a JSON object.
2. If the intent is related to "설치", "삭제", "pull" of AI models, use AiModels.
3. If the intent is related to "시스템 상태", "종료", "업데이트", use System.
4. IF THE INTENT IS AMBIGUOUS OR NOT CLEARLY MAPPED, USE Unknown.
5. NEVER fallback to System or System:Diagnostic if the user is asking about something else.
6. Confidence should be 0.0 to 1.0.

### 📝 OUTPUT SCHEMA
{
  "thought": "Brief reasoning in Korean",
  "domain": "System|AiModels|Infrastructure|PackageManagement|Unknown",
  "confidence": 0.0 to 1.0
}
"#;

pub const SYSTEM_DOMAIN_PROMPT: &str = r#"
You are the VEGA SYSTEM Action Parser.
Map the intent to a SYSTEM action.

### 🏗️ ACTIONS
- SYSTEM_DIAGNOSTIC: Get CPU, RAM, Disk, and Network status.
- SYSTEM_SHUTDOWN: Power off the system.
- SYSTEM_UPDATE: Update OS packages.

### ⚖️ ENTITY PRESERVATION RULES
1. HOSTNAMES AND IPs MUST BE ASCII PRESERVED.
2. DO NOT TRANSLATE target names.

### 📝 OUTPUT SCHEMA
{
  "thought": "Reasoning in Korean",
  "action": "SYSTEM_DIAGNOSTIC|SYSTEM_SHUTDOWN|SYSTEM_UPDATE",
  "target": "Selected Host",
  "params": {},
  "confidence": 1.0,
  "explanation": "Korean description",
  "risk_level": "INFO|WARNING|CRITICAL",
  "needs_clarification": false
}
"#;

pub const AI_MODELS_DOMAIN_PROMPT: &str = r#"
You are the VEGA AI_MODELS Action Parser.
Map the intent to an OLLAMA action.

### 🏗️ ACTIONS
- OLLAMA_LIST_INSTALLED: params: {}
- OLLAMA_PULL: params: { "model": "string" }
- OLLAMA_REMOVE: params: { "model": "string", "force": boolean }

### ⚖️ ENTITY PRESERVATION RULES
1. MODEL NAMES MUST BE ASCII PRESERVED EXACTLY AS THEY APPEAR IN THE QUERY.
2. DO NOT TRANSLATE model names (e.g., "phi 3" is NOT "문집", "총호-3", or "촁").
3. DO NOT PHONETICALLY CONVERT model names.
4. Normalize model names: "phi 3" -> "phi3", "llama 3" -> "llama3".
5. IF THE MODEL NAME IS IN THE QUERY, EXTRACT IT AS IS. NEVER GENERATE NEW KOREAN NAMES FOR MODELS.
6. TARGET MUST BE A STRING FROM THE WORLD STATE (e.g., "192.168.0.150").

### 📝 OUTPUT SCHEMA
{
  "thought": "Reasoning in Korean",
  "action": "OLLAMA_LIST_INSTALLED|OLLAMA_PULL|OLLAMA_REMOVE",
  "target": "Selected Host",
  "params": { ... },
  "confidence": 1.0,
  "explanation": "Korean description",
  "risk_level": "INFO|WARNING|CRITICAL",
  "needs_clarification": false
}
"#;

pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build_world_state(ctx: &SystemContext) -> String {
        let mut world_state = String::from("- localhost (VEGA Node)\n");
        for remote in &ctx.remotes {
            let remote_type = if remote.r#type == crate::context::RemoteType::Host { "Host" } else { "Storage" };
            world_state.push_str(&format!(
                "- {} [Type: {}, Real: {}]\n",
                remote.name, remote_type, remote.real_name
            ));
        }
        world_state
    }

    pub fn domain_prompt(ctx: &SystemContext) -> String {
        DOMAIN_CLASSIFIER_PROMPT.replace("{WORLD_STATE}", &Self::build_world_state(ctx))
    }

    pub fn action_prompt(domain: Domain) -> String {
        match domain {
            Domain::System => SYSTEM_DOMAIN_PROMPT.to_string(),
            Domain::AiModels => AI_MODELS_DOMAIN_PROMPT.to_string(),
            _ => "You are VEGA. Map natural language to actions.".to_string(),
        }
    }
}

pub struct SystemPrompt;
impl SystemPrompt {
    pub fn build(ctx: &SystemContext) -> String {
        PromptBuilder::domain_prompt(ctx)
    }
}
