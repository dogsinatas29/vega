// use log::info;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref MODEL_PATTERN: Regex = Regex::new(r"(?i)(phi[- ]?3|llama[- ]?[23]|mistral|qwen[0-9.]*|deepseek[- ]?[a-z0-9]*|gemma[0-9.]*|codellama|starcoder[0-9.]*|moe|stable[- ]?code)").unwrap();
    static ref IP_PATTERN: Regex = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();
}

pub struct ModelResolver;

impl ModelResolver {
    /// 원본 쿼리에서 모델명을 결정론적으로 추출합니다. (Slot Locking)
    pub fn resolve(original_query: &str, llm_hint: &str) -> String {
        // 1. Regex 우선 (가장 신뢰할 수 있는 소스)
        if let Some(mat) = MODEL_PATTERN.find(original_query) {
            let model = mat.as_str().to_lowercase().replace(" ", "").replace("-", "");
            return model;
        }

        // 2. LLM 힌트 검증 (ASCII 전용, IP 제외)
        let trimmed = llm_hint.trim();
        if !trimmed.is_empty() && trimmed.is_ascii() && !trimmed.contains('.') && !trimmed.contains(' ') {
            return trimmed.to_lowercase();
        }

        String::new()
    }
}

pub struct TargetResolver;

impl TargetResolver {
    /// 인벤토리 목록에서 포트 정보를 제거하고 고유한 호스트 목록을 반환합니다. (Normalization)
    fn normalize_hosts(hosts: &[String]) -> Vec<String> {
        let mut normalized: Vec<String> = hosts.iter()
            .map(|h| h.split(':').next().unwrap_or(h).to_string())
            .collect();
        normalized.sort();
        normalized.dedup();
        normalized
    }

    /// 원본 쿼리와 시스템 컨텍스트에서 타겟을 결정론적으로 추출합니다. (Symbolic Alias Resolution 강화)
    pub fn resolve(original_query: &str, llm_hint: &str, raw_hosts: &[String]) -> String {
        let lower_query = original_query.to_lowercase();
        let discovered_hosts = Self::normalize_hosts(raw_hosts);
        
        // 1. IP 패턴 우선 (가장 명시적 지표)
        if let Some(mat) = IP_PATTERN.find(original_query) {
            return mat.as_str().to_string();
        }

        // 2. Symbolic Target Alias Resolution (Authoritative)
        let symbolic_aliases = vec![
            "ssh로 접속한 시스템", "ssh에 연결된 시스템", "ssh 시스템", 
            "원격 서버", "원격 시스템", "원격지", "대상 노드", "접속된 서버",
            "remote host", "remote system", "target node", "connected system"
        ];

        let has_symbolic = symbolic_aliases.iter().any(|&alias| lower_query.contains(alias));
        if has_symbolic {
            if discovered_hosts.len() == 1 {
                crate::ui::console::SreConsole::logic(&format!("Symbolic Alias Resolved: Binding to authoritative host '{}'.", discovered_hosts[0]));
                return discovered_hosts[0].clone();
            } else if discovered_hosts.is_empty() {
                crate::ui::console::SreConsole::logic("Symbolic alias detected but no hosts discovered in inventory.");
            } else {
                crate::ui::console::SreConsole::logic(&format!("Symbolic alias detected but multiple hosts found: {:?}. Ambiguity remains.", discovered_hosts));
            }
        }

        // 3. SSH Keyword Detection (Safe Auto-Binding Promotion)
        let remote_keywords = vec!["ssh", "원격", "remote", "target"];
        if remote_keywords.iter().any(|k| lower_query.contains(k)) {
            if discovered_hosts.len() == 1 {
                crate::ui::console::SreConsole::logic(&format!("Auto-Binding: Remote keyword detected, promoting single host '{}'.", discovered_hosts[0]));
                return discovered_hosts[0].clone();
            }
        }

        // 4. 로컬 키워드 우선
        let local_keywords = vec!["내 시스템", "현재 시스템", "내 컴퓨터", "localhost", "my system", "this node"];
        if local_keywords.iter().any(|k| lower_query.contains(k)) && !lower_query.contains("ssh") {
            return "localhost".to_string();
        }

        // 5. LLM 힌트 사용 (조언 단계)
        let trimmed_hint = llm_hint.trim();
        if !trimmed_hint.is_empty() && trimmed_hint != "localhost" {
            // LLM 힌트도 정규화하여 비교
            let normalized_hint = trimmed_hint.split(':').next().unwrap_or(trimmed_hint).to_string();
            return normalized_hint;
        }

        // 6. Context-Aware Defaulting
        if lower_query.contains("localhost") || lower_query.contains("127.0.0.1") {
            "localhost".to_string()
        } else {
            // 발견된 호스트가 1개뿐일 때의 안전한 낙하 (UX 최적화)
            if discovered_hosts.len() == 1 && (lower_query.contains("system") || lower_query.contains("정보")) {
                 // 상징적 표현이 없어도 일반적인 정보 조회는 허용
                 if !lower_query.contains("셧다운") && !lower_query.contains("종료") {
                      crate::ui::console::SreConsole::logic("Passive Binding: Promoting single host for info query.");
                      return discovered_hosts[0].clone();
                 }
            }
            
            if !discovered_hosts.is_empty() && llm_hint.is_empty() {
                 crate::ui::console::SreConsole::logic(&format!("Context Check: Inventory has hosts {:?} but query is ambiguous.", discovered_hosts));
            }
            String::new() // Fail-Closed
        }
    }
}

pub struct VerbResolver;

impl VerbResolver {
    /// 원본 쿼리에서 액션을 결정론적으로 추출합니다.
    pub fn resolve_action(query: &str) -> Option<&'static str> {
        let q = query.to_lowercase();
        
        // 0. High Criticality: Stop/Shutdown/Remove (Destructive or Lifecycle Stop)
        // '꺼줘', '중지', '종료'는 어떤 서비스 키워드보다 우선함
        if q.contains("셧다운") || q.contains("종료") || q.contains("끄기") || q.contains("꺼줘") || q.contains("중지") || q.contains("stop") || q.contains("shutdown") || q.contains("poweroff") || q.contains("kill") {
            if q.contains("ollama") {
                return Some("OLLAMA_STOP");
            }
            return Some("SYSTEM_SHUTDOWN");
        }

        // 0.1 High Priority: Start/Launch (Lifecycle Start)
        if q.contains("켜줘") || q.contains("시작") || q.contains("실행") || q.contains("run") || q.contains("start") || q.contains("launch") {
            if q.contains("ollama") {
                return Some("OLLAMA_START");
            }
        }

        // 1. Ollama Actions
        if q.contains("설치") || q.contains("pull") || (q.contains("다운") && !q.contains("셧다운")) || q.contains("받아") || q.contains("install") {
            return Some("OLLAMA_PULL");
        }
        if q.contains("삭제") || q.contains("제거") || q.contains("rm") || q.contains("remove") || q.contains("delete") {
            return Some("OLLAMA_REMOVE");
        }
        if q.contains("목록") || q.contains("리스트") || q.contains("list") || q.contains("show") {
            if q.contains("실행") || q.contains("running") || q.contains("ps") {
                return Some("OLLAMA_LIST_RUNNING");
            }
            return Some("OLLAMA_LIST_INSTALLED");
        }
        if q.contains("버전") || q.contains("version") {
            return Some("OLLAMA_VERSION");
        }

        // 2. System Actions
        if q.contains("정보") || q.contains("상태") || q.contains("진단") || q.contains("status") || q.contains("diag") {
            return Some("SYSTEM_DIAGNOSTIC");
        }
        if q.contains("업데이트") || q.contains("update") || q.contains("upgrade") || q.contains("업글") {
            return Some("SYSTEM_UPDATE");
        }

        // 3. Infrastructure & Explorer Actions
        if q.contains("목록") || q.contains("리스트") || q.contains("찾아") || q.contains("보여") || q.contains("list") || q.contains("show") || q.contains("find") {
            if q.contains("ssh") || q.contains("연결된") || q.contains("접속") {
                return Some("SSH_LIST_TARGETS");
            }
            if q.contains("rclone") || q.contains("리모트") || q.contains("remote") || q.contains("클라우드") {
                return Some("INFRA_LIST_REMOTES");
            }
            if q.contains("프로세스") || q.contains("ps") || q.contains("작업") || q.contains("process") {
                return Some("SYSTEM_LIST_PROCESSES");
            }
        }

        None
    }
}

pub struct IntentValidator;

impl IntentValidator {
    /// 의도(Action)와 입력어 간의 모순을 탐지하고, 가능하다면 올바른 액션으로 교정합니다.
    pub fn validate_and_correct(action: &str, query: &str) -> String {
        let q = query.to_lowercase();
        let action_str = action.to_string();

        // 1. Deterministic Overrule: If VerbResolver finds a 100% match, trust it over LLM.
        if let Some(deterministic_action) = VerbResolver::resolve_action(&q) {
            if deterministic_action != action {
                crate::ui::console::SreConsole::logic(&format!("Correcting LLM intent drift: '{}' -> '{}'", action, deterministic_action));
                return deterministic_action.to_string();
            }
        }

        // 2. Fallback Contradiction Check
        if (q.contains("설치") || q.contains("pull")) && action == "OLLAMA_REMOVE" {
            crate::ui::console::SreConsole::error("Contradiction detected! Forcing OLLAMA_PULL.");
            return "OLLAMA_PULL".to_string();
        }
        
        action_str
    }
}

pub struct SovereignParser;

impl SovereignParser {
    /// LLM 호출 없이 원본 쿼리에서 모든 정보를 결정론적으로 추출합니다.
    pub fn parse_sovereign(query: &str, discovered_hosts: &[String]) -> crate::ai::AiResponse {
        let q = query.to_lowercase();
        
        // 1. Resolve Action (Deterministic)
        let action_opt = VerbResolver::resolve_action(&q);
        let action_str = action_opt.unwrap_or("UNKNOWN").to_string();
        
        // 2. Resolve Target (Deterministic Priority with Discovery Context)
        let target = TargetResolver::resolve(query, "", discovered_hosts); 
        
        // 3. Resolve Model (Deterministic Slot Locking)
        let mut params_map = serde_json::Map::new();
        let model = ModelResolver::resolve(query, ""); 
        if !model.is_empty() {
            params_map.insert("model".to_string(), serde_json::Value::String(model.clone()));
        }

        // 4. Determine Confidence (High if critical slots are filled)
        let has_action = action_opt.is_some();
        let has_target = !target.is_empty();
        
        // 탐색형 액션은 타겟이 없어도 자율적으로 수행 가능 (Target-less Autonomy)
        let is_discovery = action_str.contains("LIST") || action_str.contains("DISCOVER");
        let needs_model = action_str.contains("OLLAMA") && !action_str.contains("LIST");
        let has_model = !model.is_empty();

        let confidence = if has_action && (has_target || is_discovery) && (!needs_model || has_model) {
            1.0 // 👑 Fully Rehydrated sovereignly (Discovery is self-sufficient)
        } else if has_action || has_target {
            0.7 // Partially Rehydrated
        } else {
            0.0 // Still Ambiguous
        };

        let explanation = match action_str.to_uppercase().as_str() {
            "OLLAMA_STOP" => format!("원격 호스트 '{}'의 Ollama 서비스를 중단(Stop)합니다.", target),
            "OLLAMA_START" => format!("원격 호스트 '{}'의 Ollama 서비스를 시작(Start)합니다.", target),
            "OLLAMA_PULL" => format!("원격 호스트 '{}'에 모델을 다운로드(Pull)합니다.", target),
            "SYSTEM_SHUTDOWN" => format!("원격 호스트 '{}'을 종료(Shutdown)합니다.", target),
            "SSH_LIST_TARGETS" => "인벤토리에 등록된 SSH 타겟 목록을 조회합니다.".to_string(),
            "INFRA_LIST_REMOTES" => "원격 저장소 목록을 조회합니다.".to_string(),
            _ => {
                format!("결정론적 규칙 엔진이 의도('{}')를 감지하여 실행을 준비합니다.", action_str)
            }
        };

        let risk_level = if action_str.contains("SHUTDOWN") || action_str.contains("REMOVE") { "CRITICAL".to_string() } else { "INFO".to_string() };

        crate::ai::AiResponse {
            thought: format!("Sovereign Rehydration (Action: {}, Target: {}, Model: {}, Discovery: {})", action_str, target, model, is_discovery),
            action: action_str,
            target: if is_discovery && target.is_empty() { Some("inventory".to_string()) } else { Some(target) },
            params: serde_json::Value::Object(params_map),
            confidence,
            explanation,
            risk_level,
            needs_clarification: confidence < 1.0,
        }
    }

    /// LLM 응답과 Sovereign 분석 결과를 화해(Reconcile)시키고 최종 Typed Intent를 생성합니다.
    pub fn reconcile_to_intent(sovereign: crate::ai::AiResponse, mut llm: crate::ai::AiResponse, discovered_hosts: &[String]) -> crate::ai::AiResponse {
        // Sovereign 결과가 완벽(Confidence 1.0)하면 LLM 결과를 완전히 무시 (Fast-Path)
        if sovereign.confidence >= 1.0 {
            crate::ui::console::SreConsole::logic("Sovereign: Deterministic match found. Trusting Sovereign Parser.");
            return sovereign;
        }

        // 1. Action Ownership (Inversion)
        if sovereign.action != "UNKNOWN" {
             if sovereign.action != llm.action {
                 crate::ui::console::SreConsole::logic(&format!("Ownership: Overriding LLM intent '{}' with authoritative Sovereign intent '{}'", llm.action, sovereign.action));
                 llm.action = sovereign.action;
             }
        }

        // 2. Target Rehydration (Ownership + Discovery Binding)
        if let Some(s_target) = &sovereign.target {
            if !s_target.is_empty() {
                let llm_target = llm.target.as_deref().unwrap_or("");
                if llm_target.is_empty() || llm_target == "localhost" {
                    crate::ui::console::SreConsole::logic(&format!("Rehydration: Rehydrating target from raw query & discovery: '{}'", s_target));
                    llm.target = Some(s_target.clone());
                }
            }
        }

        // 2-1. Deep Rehydration: If LLM failed target but query mentions SSH and we have discovery
        if llm.target.as_deref().unwrap_or("").is_empty() || llm.target.as_deref().unwrap_or("") == "localhost" {
             let q = sovereign.thought.to_lowercase(); // Or query directly if available
             if q.contains("ssh") || q.contains("원격") {
                  if discovered_hosts.len() == 1 {
                       crate::ui::console::SreConsole::logic(&format!("Rehydration: Semantic match: Mapping SSH request to the only discovered host: {}", discovered_hosts[0]));
                       llm.target = Some(discovered_hosts[0].clone());
                  }
             }
        }

        // 3. Model Rehydration (Slot Locking)
        if let Some(s_model) = sovereign.params.get("model") {
             if let Some(obj) = llm.params.as_object_mut() {
                 if obj.get("model").map_or(true, |v| v.is_null()) {
                      crate::ui::console::SreConsole::logic(&format!("Rehydration: Rehydrating model slot: {}", s_model));
                      obj.insert("model".to_string(), s_model.clone());
                 }
             }
        }

        // 4. Final Integrity Check: Action must not be empty and must be semantically complete
        let final_action = llm.action.clone();
        let target_str = llm.target.as_deref().unwrap_or("");
        let model_str = llm.params.get("model").and_then(|v| v.as_str()).unwrap_or("");

        // 4-1. Mandatory Slot Validation (Semantic Firewall)
        if final_action == "OLLAMA_PULL" || final_action == "OLLAMA_REMOVE" {
            if model_str.is_empty() || model_str == "unknown" {
                crate::ui::console::SreConsole::error("Firewall: Blocking incomplete Ollama action: Model slot is empty.");
                llm.action = "UNKNOWN".to_string();
                llm.confidence = 0.0;
            }
        }

        if final_action == "SYSTEM_SHUTDOWN" {
            if target_str.is_empty() {
                crate::ui::console::SreConsole::error("Firewall: Blocking incomplete Shutdown action: Target slot is empty.");
                llm.action = "UNKNOWN".to_string();
                llm.confidence = 0.0;
            }
        }

        if llm.action.is_empty() || llm.action == "UNKNOWN" {
             crate::ui::console::SreConsole::error("Integrity: Action is empty or blocked! Aborting to prevent catastrophic failure.");
             llm.action = "UNKNOWN".to_string();
             llm.confidence = 0.0;
        }

        llm
    }
}
