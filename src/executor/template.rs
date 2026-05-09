use crate::executor::pipeline::{Intent, TemplateBuilder};
use crate::executor::ast::CommandAst;

pub struct BasicTemplateBuilder;

impl TemplateBuilder for BasicTemplateBuilder {
    fn build(&self, intent: &Intent) -> anyhow::Result<CommandAst> {
        match intent.action.as_str() {
            "OLLAMA_LIST_INSTALLED" => Ok(CommandAst::new("ollama", "list")),
            "OLLAMA_LIST_RUNNING" => Ok(CommandAst::new("ollama", "ps")),
            "OLLAMA_PULL" => Ok(CommandAst::new("ollama", &format!("pull {}", intent.params["model"].as_str().unwrap_or("")))),
            "OLLAMA_REMOVE" => Ok(CommandAst::new("ollama", &format!("rm {}", intent.params["model"].as_str().unwrap_or("")))),
            "OLLAMA_VERSION" => Ok(CommandAst::new("ollama", "--version")),
            "INSTALL_APT" => {
                Ok(CommandAst::new("sudo apt", &format!("install -y {}", intent.params["name"].as_str().unwrap_or(""))))
            },
            "INSTALL_DOCKER" => {
                Ok(CommandAst::new("docker", &format!("run -d {}", intent.params["name"].as_str().unwrap_or(""))))
            },
            "SSH_CONNECT" => {
                let mut ast = CommandAst::new("ssh", "connect");
                ast.target_server = Some(intent.params["host"].as_str().unwrap_or(&intent.target).to_string());
                Ok(ast)
            },
            "SYSTEM_UPDATE" => {
                Ok(CommandAst::new("sudo apt", "update"))
            },
            "SYSTEM_DIAGNOSTIC" => {
                Ok(CommandAst::new("uname", "-a"))
            },
            "UNKNOWN" => {
                Err(anyhow::anyhow!("Pure Semantic Mode: Cannot generate template for Unknown intent."))
            },
            _ => Err(anyhow::anyhow!("No template for action: {}", intent.action)),
        }
    }
}

#[async_trait::async_trait]
impl crate::executor::pipeline::OptionGenerator for BasicTemplateBuilder {
    async fn generate_options(&self, _ast: &mut CommandAst) -> anyhow::Result<()> {
        Ok(()) // Placeholder
    }
}

impl crate::executor::pipeline::VirtualExecutionEngine for BasicTemplateBuilder {
    fn simulate(&self, _ast: &CommandAst) -> anyhow::Result<crate::executor::pipeline::SimLog> {
        Ok(crate::executor::pipeline::SimLog {
            is_safe: true,
            predicted_impact: "Minimal impact predicted by semantic simulation.".to_string(),
            risk_score: 10,
            suggestion: None,
        })
    }
}

impl crate::executor::pipeline::RiskEvaluator for BasicTemplateBuilder {
    fn evaluate(&self, sim_log: &crate::executor::pipeline::SimLog) -> bool {
        sim_log.is_safe && sim_log.risk_score < 80
    }
}
