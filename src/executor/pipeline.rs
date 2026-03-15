use crate::executor::ast::CommandAst;
use crate::executor::ExecuteResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub tool: String,
    pub operation: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimLog {
    pub is_safe: bool,
    pub predicted_impact: String,
    pub risk_score: i32,
    pub suggestion: Option<String>,
}

#[async_trait]
pub trait IntentResolver {
    async fn resolve(&self, input: &str) -> anyhow::Result<Intent>;
}

pub trait TemplateBuilder {
    fn build(&self, intent: &Intent) -> anyhow::Result<CommandAst>;
}

#[async_trait]
pub trait OptionGenerator {
    async fn generate_options(&self, ast: &mut CommandAst) -> anyhow::Result<()>;
}

pub trait VirtualExecutionEngine {
    fn simulate(&self, ast: &CommandAst) -> anyhow::Result<SimLog>;
}

pub trait RiskEvaluator {
    fn evaluate(&self, sim_log: &SimLog) -> bool; // returns true if safe to execute
}

#[async_trait]
pub trait ExecutionProvider {
    async fn execute(&self, ast: &CommandAst) -> anyhow::Result<ExecuteResult>;
}

pub struct PipelineOrchestrator {
    pub intent_resolver: Box<dyn IntentResolver + Send + Sync>,
    pub template_builder: Box<dyn TemplateBuilder + Send + Sync>,
    pub option_generator: Box<dyn OptionGenerator + Send + Sync>,
    pub vee: Box<dyn VirtualExecutionEngine + Send + Sync>,
    pub risk_evaluator: Box<dyn RiskEvaluator + Send + Sync>,
    pub execution_provider: Box<dyn ExecutionProvider + Send + Sync>,
}

impl PipelineOrchestrator {
    pub async fn run_pipeline(&self, input: &str) -> anyhow::Result<ExecuteResult> {
        // 1. Intent Resolution
        let intent = self.intent_resolver.resolve(input).await?;
        let intent_str = format!("{:?}", intent);
        
        // 2. Template Building
        let mut ast = self.template_builder.build(&intent)?;
        
        // 3. AI Option Generation
        self.option_generator.generate_options(&mut ast).await?;
        let final_cmd = ast.to_shell_command();
        
        // 4. Virtual Execution (Simulation)
        let sim_log = self.vee.simulate(&ast)?;
        let sim_log_str = format!("{:?}", sim_log);
        let risk_score = sim_log.risk_score;
        
        // 5. Risk Evaluation
        if !self.risk_evaluator.evaluate(&sim_log) {
            // Log rejection
            if let Ok(db) = crate::storage::db::Database::new() {
                let _ = db.log_decision_lineage(input, &intent_str, &final_cmd, &sim_log_str, risk_score, "DENIED");
            }
            anyhow::bail!("Execution denied by Risk Evaluation Engine.");
        }
        
        // 6. Execution
        let result = self.execution_provider.execute(&ast).await?;
        
        // 7. Decision Lineage Persistence
        if let Ok(db) = crate::storage::db::Database::new() {
            let res_str = format!("Success: {}, ExitCode: {:?}", result.success, result.exit_code);
            let _ = db.log_decision_lineage(input, &intent_str, &final_cmd, &sim_log_str, risk_score, &res_str);
        }
        
        Ok(result)
    }
}

pub struct LocalExecutionProvider;

#[async_trait]
impl ExecutionProvider for LocalExecutionProvider {
    async fn execute(&self, ast: &CommandAst) -> anyhow::Result<ExecuteResult> {
        let cmd_str = ast.to_shell_command();
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd_str)
            .output()?;
        
        Ok(ExecuteResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        })
    }
}

pub struct RemoteExecutionProvider {
    pub ip: String,
    pub user: Option<String>,
}

#[async_trait]
impl ExecutionProvider for RemoteExecutionProvider {
    async fn execute(&self, ast: &CommandAst) -> anyhow::Result<ExecuteResult> {
        let cmd_str = ast.to_shell_command();
        let output = crate::connection::ssh::SshConnection::execute_remote_async(&self.ip, &cmd_str).await
            .map_err(|e| anyhow::anyhow!("SSH Execution Failed: {}", e))?;
        
        Ok(ExecuteResult {
            success: true, 
            stdout: output,
            stderr: "".to_string(),
            exit_code: Some(0),
        })
    }
}
