use crate::executor::pipeline::{Intent, TemplateBuilder};
use crate::executor::ast::CommandAst;

pub struct BasicTemplateBuilder;

impl TemplateBuilder for BasicTemplateBuilder {
    fn build(&self, intent: &Intent) -> anyhow::Result<CommandAst> {
        let mut ast = CommandAst::new(&intent.tool, &intent.operation);
        
        match intent.tool.as_str() {
            "rclone" => {
                // Assume operation is sync or copy
                if let Some(target) = &intent.target {
                    ast.source = Some(".".to_string()); // Default to current dir for NL
                    ast.destination = Some(format!("{}:backup", target));
                }
            },
            "ssh" => {
                if let Some(target) = &intent.target {
                    ast.target_server = Some(target.clone());
                }
            },
            "pkg" => {
                if let Some(target) = &intent.target {
                    ast.options.push(target.clone());
                }
            },
            _ => {
                // For unknown tools, just use the target as source if available
                if let Some(target) = &intent.target {
                    ast.source = Some(target.clone());
                }
            }
        }
        
        Ok(ast)
    }
}
