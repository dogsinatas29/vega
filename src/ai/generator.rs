use crate::executor::pipeline::OptionGenerator;
use crate::executor::ast::CommandAst;
use crate::ai::router::SmartRouter;
use crate::context::SystemContext;
use async_trait::async_trait;

pub struct AiOptionGenerator;

#[async_trait]
impl OptionGenerator for AiOptionGenerator {
    async fn generate_options(&self, ast: &mut CommandAst) -> anyhow::Result<()> {
        let ctx = SystemContext::collect();
        let query = format!(
            "CONTEXT: You are VEGA, a 20-year Senior SRE. 
            TASK: Provide optimal and safe CLI options for the following command skeleton.
            TOOL: {}
            OPERATION: {}
            SOURCE: {:?}
            DESTINATION: {:?}
            TARGET SERVER: {:?}
            
            RULES:
            1. Return ONLY a JSON array of strings, e.g., [\"--progress\", \"--checksum\"].
            2. Do not explain anything. 
            3. Prioritize safety and performance.",
            ast.tool, ast.operation, ast.source, ast.destination, ast.target_server
        );

        match SmartRouter::generate_with_fallback(&ctx, &query, None).await {
            Ok(res) => {
                // Attempt to parse result as JSON array
                let res_trimmed = res.trim();
                if let Ok(options) = serde_json::from_str::<Vec<String>>(res_trimmed) {
                    ast.options.extend(options);
                } else {
                    // Fallback: try to find something that looks like an array or just split whitespace
                    let cleaned = res_trimmed
                        .replace("[", "")
                        .replace("]", "")
                        .replace("\"", "")
                        .replace(",", " ");
                    for flag in cleaned.split_whitespace() {
                        if flag.starts_with('-') {
                            ast.options.push(flag.to_string());
                        }
                    }
                }
                Ok(())
            },
            Err(e) => anyhow::bail!("AI Option Generation failed: {}", e),
        }
    }
}
