use crate::executor::pipeline::{Intent, TemplateBuilder};
use crate::executor::ast::CommandAst;

pub struct BasicTemplateBuilder;

impl TemplateBuilder for BasicTemplateBuilder {
    fn build(&self, intent: &Intent) -> anyhow::Result<CommandAst> {
        match intent {
            Intent::OllamaListInstalled {} => Ok(CommandAst::new("ollama", "list")),
            Intent::OllamaListRunning {} => Ok(CommandAst::new("ollama", "ps")),
            Intent::OllamaPull { model } => Ok(CommandAst::new("ollama", &format!("pull {}", model))),
            Intent::OllamaRemove { model, .. } => Ok(CommandAst::new("ollama", &format!("rm {}", model))),
            Intent::OllamaVersion {} => Ok(CommandAst::new("ollama", "--version")),
            Intent::InstallApt { name } => {
                Ok(CommandAst::new("sudo apt", &format!("install -y {}", name)))
            },
            Intent::InstallDocker { name } => {
                Ok(CommandAst::new("docker", &format!("run -d {}", name)))
            },
            Intent::SshConnect { host } => {
                let mut ast = CommandAst::new("ssh", "connect");
                ast.target_server = Some(host.clone());
                Ok(ast)
            },
            Intent::SystemUpdate {} => {
                Ok(CommandAst::new("sudo apt", "update"))
            },
            Intent::BackupData { source, target } => {
                let mut ast = CommandAst::new("rclone", "sync");
                ast.source = Some(source.clone());
                ast.destination = Some(format!("{}:backup", target));
                Ok(ast)
            },
            Intent::Unknown => {
                Err(anyhow::anyhow!("Pure Semantic Mode: Cannot generate template for Unknown intent."))
            }
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
