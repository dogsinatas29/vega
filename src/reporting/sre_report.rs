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
        
        let (system_lang, h_physical, h_req, hint_text) = if data.context.locale.starts_with("ko") {
            ("Korean (한국어)", "물리적 컨텍스트", "출력 요구사항", "한국어로 작성")
        } else {
            ("English", "Physical Context", "Output Requirements", "Write in English")
        };

        let prompt = format!(
            r#"당신은 20년 경력의 베테랑 Senior SRE로서, 다음 데이터를 기반으로 전문적인 시스템 진단 보고서를 생성하십시오.
언어: {} (반드시 지킬 것)
톤: 전문적이고 간결하며 행동 지향적.

## 🖥️ {}
- 호스트: {}
- 업타임: {}
- 로드 부하: {:?}
- 메모리 정보: {}
- 디스크 IO: {}
- 활성 포트: {}
- 원격 노드: {}

## 📄 {}
- 형식: 반드시 다음 스키마를 따르는 JSON:
{{
  "emotional_summary": "한 줄 감성 요약 (예: 시스템이 안정적입니다)",
  "issue": "{}",
  "cause": "{}",
  "solution": "{}",
  "forecast": "{}",
  "result": "{}"
}}

중요: 모든 필드 내용은 반드시 {}로만 작성하십시오.
"#,
            system_lang,
            h_physical,
            data.context.hostname, 
            data.uptime, 
            data.context.load_avg,
            serde_json::to_string(&data.context.mem_info).unwrap_or_default(),
            data.disk_io,
            data.listening_ports.join("\n"),
            serde_json::to_string(&data.context.remotes).unwrap_or_default(),
            h_req,
            hint_text,
            hint_text,
            hint_text,
            hint_text,
            hint_text,
            system_lang
        );

        let dummy_ctx = crate::context::SystemContext::collect();
        match crate::ai::router::SmartRouter::generate_with_fallback(&dummy_ctx, &prompt, None).await {
            Ok(json_str) => {
                // 🛡️ Milestone v0.0.14.9: Robust JSON Extraction
                let clean_json = if json_str.contains("```json") {
                    json_str.split("```json").nth(1).unwrap_or(&json_str)
                            .split("```").next().unwrap_or(&json_str)
                            .trim().to_string()
                } else if json_str.contains("```") {
                    json_str.split("```").nth(1).unwrap_or(&json_str)
                            .split("```").next().unwrap_or(&json_str)
                            .trim().to_string()
                } else {
                    json_str.trim().to_string()
                };

                if let Ok(res) = serde_json::from_str::<serde_json::Value>(&clean_json) {
                    report.emotional_summary = res["emotional_summary"].as_str().unwrap_or("진단 완료").to_string();
                    report.issue = res["issue"].as_str().unwrap_or("지표 정상").to_string();
                    report.cause = res["cause"].as_str().unwrap_or("해당 없음").to_string();
                    report.solution = res["solution"].as_str().unwrap_or("모니터링 유지").to_string();
                    report.forecast = res["forecast"].as_str().unwrap_or("안정적").to_string();
                    report.result = res["result"].as_str().unwrap_or("양호").to_string();
                } else {
                    eprintln!("⚠️ AI Parsing Failed. Raw Response: {}", json_str);
                    report.emotional_summary = "AI 분석 데이터 해석 실패".to_string();
                }
            },
            Err(e) => eprintln!("⚠️ AI Report Generation Failed: {}", e),
        }

        Ok(report)
    }

    pub async fn generate_from_lineage(
        session_id: i64, 
        lineage: &[crate::storage::db::DecisionRecord]
    ) -> anyhow::Result<Self> {
        let data = crate::system::diagnostic::DiagnosticScanner::scan();
        let mut report = Self::new(session_id);
        report.raw_data = Some(data.clone());
        
        // 1. Data Aggregation for AI
        let mut context_str = String::new();
        for (i, rec) in lineage.iter().enumerate() {
            context_str.push_str(&format!(
                "Event #{}:\nRequest: {}\nIntent: {}\nCommand: {}\nResult: {}\n\n",
                i + 1, rec.user_request, rec.intent, rec.generated_command, rec.execution_result
            ));
        }

        let total_kb = data.context.mem_info["MemTotal"].as_str().unwrap_or("0").replace(" kB", "").parse::<f64>().unwrap_or(0.0);
        let avail_kb = data.context.mem_info["MemAvailable"].as_str().unwrap_or("0").replace(" kB", "").parse::<f64>().unwrap_or(0.0);
        let used_gb = (total_kb - avail_kb) / 1024.0 / 1024.0;
        let total_gb = total_kb / 1024.0 / 1024.0;

        // 💡 Milestone v0.0.14.9: Dynamic Header Translation (Forcing Contextual Language)
        let (system_lang, h_physical, h_session, h_req, hint_text) = if data.context.locale.starts_with("ko") {
            ("Korean (한국어)", "물리적 컨텍스트", "세션 히스토리", "출력 요구사항", "한국어로 작성")
        } else if data.context.locale.starts_with("ja") {
            ("Japanese (日本語)", "物理的コン텍스트", "セッション履歴", "出力要件", "日本語で記述")
        } else if data.context.locale.starts_with("zh") {
            ("Chinese (中文)", "物理上下文", "会话历史", "输出要求", "用中文编写")
        } else {
            ("English", "Physical Context", "Session History", "Output Requirements", "Write in English")
        };

        // 2. Call AI for Summarization (Using SmartRouter)
        let prompt = format!(
            r#"Analyze the following SRE Diagnostic Data and generate a professional report in {}.
MANDATORY: You MUST respond in {} ONLY.

## 🖥️ {}
- OS: {}
- Desktop: {}
- CPU: {}
- RAM: {:.2} GB / {:.2} GB
- Uptime: {}

## 📊 {}
{}

## 📄 {}
- Language: {} (Strictly)
- Format: JSON strictly following this schema:
{{
  "emotional_summary": "{}",
  "issue": "{}",
  "cause": "{}",
  "solution": "{}",
  "forecast": "{}",
  "result": "{}"
}}

CRITICAL: Your entire response (thought and summary) MUST be in {} ONLY.
"#,
            system_lang,
            system_lang,
            h_physical,
            data.os_detailed,
            data.de_info,
            data.cpu_info,
            used_gb,
            total_gb,
            data.uptime,
            h_session,
            context_str,
            h_req,
            system_lang,
            hint_text,
            hint_text,
            hint_text,
            hint_text,
            hint_text,
            hint_text,
            system_lang
        );

        let dummy_ctx = crate::context::SystemContext::collect();
        match crate::ai::router::SmartRouter::generate_with_fallback(&dummy_ctx, &prompt, None).await {
            Ok(json_str) => {
                // 🛡️ Milestone v0.0.14.9: Robust JSON Extraction
                let clean_json = if json_str.contains("```json") {
                    json_str.split("```json").nth(1).unwrap_or(&json_str)
                            .split("```").next().unwrap_or(&json_str)
                            .trim().to_string()
                } else if json_str.contains("```") {
                    json_str.split("```").nth(1).unwrap_or(&json_str)
                            .split("```").next().unwrap_or(&json_str)
                            .trim().to_string()
                } else {
                    json_str.trim().to_string()
                };

                if let Ok(res) = serde_json::from_str::<serde_json::Value>(&clean_json) {
                    report.emotional_summary = res["emotional_summary"].as_str().unwrap_or("분석 완료").to_string();
                    report.issue = res["issue"].as_str().unwrap_or("지표 정상").to_string();
                    report.cause = res["cause"].as_str().unwrap_or("해당 없음").to_string();
                    report.solution = res["solution"].as_str().unwrap_or("모니터링 유지").to_string();
                    report.forecast = res["forecast"].as_str().unwrap_or("안정적").to_string();
                    report.result = res["result"].as_str().unwrap_or("양호").to_string();
                } else {
                    eprintln!("⚠️ AI Parsing Failed. Raw Response: {}", json_str);
                    report.emotional_summary = "AI 분석 데이터 해석 실패".to_string();
                }
            },
            Err(e) => eprintln!("⚠️ AI Report Generation Failed: {}", e),
        }

        Ok(report)
    }

    pub fn render_markdown(&self) -> String {
        let mut md = format!(
            r#"# 🚀 VEGA SRE System Report
**Timestamp**: {}
**Session**: #{}

## 📊 Emotional Summary
"{}"

---

"#,
            self.timestamp, self.session_id, self.emotional_summary
        );

        let locale = if let Some(data) = &self.raw_data {
            &data.context.locale
        } else {
            "en_US"
        };

        // 📉 Technical Metrics Section (Raw Data)
        let (h_metrics, h_pulse, h_net, h_fleet) = if locale.starts_with("ko") {
            ("📉 시스템 지표 (Raw Data)", "### 1. 하드웨어 맥박", "### 2. 네트워크 맵 (활성 포트)", "### 3. 플릿 및 저장소 현황")
        } else if locale.starts_with("ja") {
            ("📉 システム指標 (Raw Data)", "### 1. ハードウェアの鼓動", "### 2. ネットワークマップ (有効なポート)", "### 3. フリートとストレージのステータス")
        } else {
            ("📉 System Metrics (Raw Data)", "### 1. Hardware Pulse", "### 2. Network Map (Listening Ports)", "### 3. Fleet & Storage Status")
        };

        if let Some(data) = &self.raw_data {
            md.push_str(&format!("## {}\n\n", h_metrics));
            
            // 1. Hardware Pulse
            md.push_str(&format!("{}\n", h_pulse));
            md.push_str(&format!("- **OS**: {}\n", data.os_detailed));
            md.push_str(&format!("- **Desktop**: {}\n", data.de_info));
            md.push_str(&format!("- **CPU**: {}\n", data.cpu_info));
            md.push_str(&format!("- **Locale**: {}\n", data.context.locale));
            md.push_str(&format!("- **Uptime**: {}\n", data.uptime));
            md.push_str(&format!("- **Load Avg**: {:?}\n", data.context.load_avg));
            
            // 🐏 RAM Normalization (kB -> GB)
            if let Some(mem_total_str) = data.context.mem_info["MemTotal"].as_str() {
                let total_kb = mem_total_str.replace(" kB", "").parse::<f64>().unwrap_or(0.0);
                if let Some(mem_avail_str) = data.context.mem_info["MemAvailable"].as_str() {
                    let avail_kb = mem_avail_str.replace(" kB", "").parse::<f64>().unwrap_or(0.0);
                    let used_gb = (total_kb - avail_kb) / 1024.0 / 1024.0;
                    let total_gb = total_kb / 1024.0 / 1024.0;
                    md.push_str(&format!("- **RAM**: {:.2} GB / {:.2} GB (Used/Total)\n", used_gb, total_gb));
                }
            }
            md.push_str("\n");

            // 2. Network Map (Ports)
            md.push_str(&format!("{}\n", h_net));
            md.push_str("```text\n");
            for port in data.listening_ports.iter().take(10) {
                md.push_str(&format!("{}\n", port));
            }
            if data.listening_ports.len() > 10 {
                md.push_str("... (truncated)\n");
            }
            md.push_str("```\n\n");

            // 3. Fleet Status
            md.push_str(&format!("{}\n", h_fleet));
            md.push_str("| Target | Type | Provider | Status |\n");
            md.push_str("|--------|------|----------|--------|\n");
            for remote in &data.context.remotes {
                md.push_str(&format!("| {} | {:?} | {} | {} |\n", remote.name, remote.r#type, remote.provider, remote.status));
            }
            md.push_str("\n---\n\n");
        }

        // 📄 SRE 5-Step Analysis
        let (h_sre, step1, step2, step3, step4, step5) = if locale.starts_with("ko") {
            ("## 📄 SRE 5단계 분석", "### 1. 현안 및 문제 (Issue)", "### 2. 근본 원인 분석 (Cause)", "### 3. 해결 방안 (Solution)", "### 4. 도입 결과 예측 (Forecast)", "### 5. 최종 수행 결과 (Result)")
        } else {
            ("## 📄 SRE 5-Step Analysis", "### 1. Issue", "### 2. Cause", "### 3. Solution", "### 4. Forecast", "### 5. Result")
        };

        md.push_str(&format!(
            r#"{}

{}
{}

{}
{}

{}
{}

{}
{}

{}
{}

---
**"Report generated by VEGA SRE Intelligence."**
"#,
            h_sre,
            step1, self.issue,
            step2, self.cause,
            step3, self.solution,
            step4, self.forecast,
            step5, self.result
        ));

        md
    }
}
