use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Goal {
    /// 인프라 전체의 LLM 모델 정보를 취합하고 상태를 파악함
    EnumerateLlmModels,
    /// 시스템 건강 상태를 진단하고 임계값 기반 분석 수행
    DiagnoseFleet,
    /// 특정 서비스(Ollama, Docker 등)의 배포 및 최적화
    DeployService {
        service: String,
        target_state: String,
    },
    /// 알 수 없는 요청에 대한 시맨틱 추론 시도
    Unknown(String),
}

pub struct GoalPlanner;

impl GoalPlanner {
    pub fn plan(goal: &Goal) -> Vec<String> {
        match goal {
            Goal::EnumerateLlmModels => vec![
                "Discovery: 인프라 내 LLM 프로바이더 탐색".to_string(),
                "Inventory: 설치된 모델 리스트 취합".to_string(),
                "Status: 현재 가동 중인 모델 상태 확인".to_string(),
            ],
            Goal::DiagnoseFleet => vec![
                "Sensing: 시스템 리소스 메트릭 수집".to_string(),
                "Analysis: 결정론적 임계값 엔진 기반 분석".to_string(),
                "Insight: SRE 관점의 최적화 제안 생성".to_string(),
            ],
            _ => vec!["추가적인 세부 계획이 필요합니다.".to_string()],
        }
    }
}
