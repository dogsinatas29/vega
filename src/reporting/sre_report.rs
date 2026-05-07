use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SreReport {
    pub session_id: i64,
    pub timestamp: String,
    pub emotional_summary: String,
    pub issue: String,
    pub cause: String,
    pub solution: String,
    pub forecast: String,
    pub result: String,
    pub raw_data: Option<crate::system::diagnostic::DiagnosticData>,
}

impl SreReport {
    pub fn new(session_id: i64) -> Self {
        Self {
            session_id,
            timestamp: chrono::Local::now().to_rfc3339(),
            emotional_summary: String::from("No data analyzed yet."),
            issue: String::from("-"),
            cause: String::from("-"),
            solution: String::from("-"),
            forecast: String::from("-"),
            result: String::from("-"),
            raw_data: None,
        }
    }

    pub async fn generate_full_diagnostic(
        session_id: i64,
        data: crate::system::diagnostic::DiagnosticData
    ) -> anyhow::Result<Self> {
        let mut report = Self::new(session_id);
        report.raw_data = Some(data.clone());
        
        let prompt = format!(
            r#"As a 20-year veteran Senior SRE, generate a SOVEREIGN SYSTEM REPORT based on this data.
Language: Korean (Strictly)
Tone: Professional, Concise, Action-oriented.

DATASET:
- Host: {}
- Uptime: {}
- Load: {:?}
- Memory: {}
- Disk IO: {}
- Listening Ports: {}
- Remotes: {}

Output JSON Format:
{{
  "emotional_summary": "One-line emotional impact",
  "issue": "Summary of problems found",
  "cause": "Deep root cause analysis",
  "solution": "Immediate technical actions",
  "forecast": "Expected stability after fix",
  "result": "Final verdict on system health"
}}
"#, 
            data.context.hostname, 
            data.uptime, 
            data.context.load_avg,
            serde_json::to_string(&data.context.mem_info).unwrap_or_default(),
            data.disk_io,
            data.listening_ports.join("\n"),
            serde_json::to_string(&data.context.remotes).unwrap_or_default()
        );

        let dummy_ctx = crate::context::SystemContext::collect();
        match crate::ai::router::SmartRouter::generate_with_fallback(&dummy_ctx, &prompt, None).await {
            Ok(json_str) => {
                if let Ok(res) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    report.emotional_summary = res["emotional_summary"].as_str().unwrap_or("Done").to_string();
                    report.issue = res["issue"].as_str().unwrap_or("-").to_string();
                    report.cause = res["cause"].as_str().unwrap_or("-").to_string();
                    report.solution = res["solution"].as_str().unwrap_or("-").to_string();
                    report.forecast = res["forecast"].as_str().unwrap_or("-").to_string();
                    report.result = res["result"].as_str().unwrap_or("-").to_string();
                }
            },
            Err(_) => {
                report.emotional_summary = "AI 요약 실패 (기술 데이터를 직접 확인하십시오)".to_string();
            },
        }

        Ok(report)
    }

    pub async fn generate_from_lineage(
        session_id: i64, 
        lineage: &[crate::storage::db::DecisionRecord]
    ) -> anyhow::Result<Self> {
        if lineage.is_empty() {
            return Ok(Self::new(session_id));
        }

        let mut report = Self::new(session_id);
        
        // 1. Data Aggregation for AI
        let mut context_str = String::new();
        for (i, rec) in lineage.iter().enumerate() {
            context_str.push_str(&format!(
                "Event #{}:\nRequest: {}\nIntent: {}\nCommand: {}\nResult: {}\n\n",
                i + 1, rec.user_request, rec.intent, rec.generated_command, rec.execution_result
            ));
        }

        // 2. Call AI for Summarization (Using SmartRouter)
        let prompt = format!(
            r#"As a Senior SRE, analyze the following session data and generate a technical report in the 5-step SRE format.
Language: Korean
Data:
{}

Output JSON Format:
{{
  "emotional_summary": "One-line emotional impact",
  "issue": "Summary of problems",
  "cause": "Root cause analysis",
  "solution": "Actions taken",
  "forecast": "Long-term impact",
  "result": "Final outcome"
}}
"#, context_str);

        let dummy_ctx = crate::context::SystemContext::collect();
        match crate::ai::router::SmartRouter::generate_with_fallback(&dummy_ctx, &prompt, None).await {
            Ok(json_str) => {
                if let Ok(res) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    report.emotional_summary = res["emotional_summary"].as_str().unwrap_or("Done").to_string();
                    report.issue = res["issue"].as_str().unwrap_or("-").to_string();
                    report.cause = res["cause"].as_str().unwrap_or("-").to_string();
                    report.solution = res["solution"].as_str().unwrap_or("-").to_string();
                    report.forecast = res["forecast"].as_str().unwrap_or("-").to_string();
                    report.result = res["result"].as_str().unwrap_or("-").to_string();
                }
            },
            Err(e) => eprintln!("⚠️ AI Report Generation Failed: {}", e),
        }

        Ok(report)
    }

    pub fn render_markdown(&self) -> String {
        format!(
            r#"# 🚀 VEGA SRE Session Report - Session #{}

## 📊 Emotional Summary
"{}"

---

## 📄 SRE 5-Step Analysis

### 1. 현안 및 문제 (Issue)
{}

### 2. 근본 원인 분석 (Cause)
{}

### 3. 해결 방안 (Solution)
{}

### 4. 도입 결과 예측 (Forecast)
{}

### 5. 최종 수행 결과 (Result)
{}

---
**"Report generated by VEGA SRE Intelligence."**
"#,
            self.session_id,
            self.emotional_summary,
            self.issue,
            self.cause,
            self.solution,
            self.forecast,
            self.result
        )
    }
}
