use crate::executor::pipeline::{RiskEvaluator, SimLog};
use crate::safety::{RiskLevel, confirm_action};

pub struct DefaultRiskEvaluator;

impl RiskEvaluator for DefaultRiskEvaluator {
    fn evaluate(&self, sim_log: &SimLog) -> bool {
        if !sim_log.is_safe && sim_log.risk_score >= 100 {
            println!("🛑 SAFETY ALERT: Command blocked strictly by simulation.");
            println!("   Reason: {}", sim_log.predicted_impact);
            return false;
        }

        let risk = if sim_log.risk_score >= 90 {
            RiskLevel::Critical
        } else if sim_log.risk_score >= 50 {
            RiskLevel::Warning
        } else {
            RiskLevel::Info
        };

        let description = format!("{} (Score: {})", sim_log.predicted_impact, sim_log.risk_score);
        confirm_action(risk, &description)
    }
}
