🚀 [LLM 코딩 4대 원칙]
​1. 코딩 전 사고 (Think Before Coding)
 추측하지 마라. 혼란을 숨기지 말고 트레이드오프를 드러내라.
 ​가정을 명시적으로 밝힐 것. 불확실하다면 추측하지 말고 질문하라.
 ​모호함이 존재할 때 독단적으로 선택하지 말고 여러 해석을 제시하라.
 ​더 간단한 방법이 있다면 과감하게 제안하라.
 ​불명확한 부분이 있다면 작업을 멈추고 명확한 설명을 요구하라.

​2. 단순성 우선 (Simplicity First)
 문제를 해결하는 최소한의 코드만 작성하라. 추측에 기반한 코드는 금지한다.
 ​요청하지 않은 기능은 추가하지 마라.
 ​일회성 코드에 추상화를 적용하지 마라. 
 ​요청되지 않은 유연성이나 설정 기능을 배제하라.
 ​발생 불가능한 시나리오에 대한 예외 처리를 하지 마라.
 ​200줄의 코드를 50줄로 줄일 수 있다면 다시 써라.

​3. 최소한의 수정 (Surgical Changes)
 반드시 필요한 부분만 건드려라. 본인이 만든 문제만 정리하라.
 ​주변 코드, 주석, 포맷을 임의로 '개선'하지 마라.
 ​고장 나지 않은 것을 리팩토링하지 마라.
 ​본인의 스타일보다 기존 코드의 스타일을 우선하여 맞춰라.
 ​본인의 수정으로 인해 발생한 미사용 변수나 함수만 제거하라. 기존에 있던 데드 코드는 요청 없이는 건드리지 마라.

​4. 목표 중심 실행 (Goal-Driven Execution)
 성공 기준을 정의하라. 검증될 때까지 반복하라.
 ​'그냥 실행'하는 대신 검증 가능한 목표로 변환하라. (예: 버그 수정 시, 실패하는 테스트를 먼저 작성하고 통과시키기)
 ​다단계 작업의 경우 각 단계와 검증 방법을 명시한 계획을 세워라.
 ​'작동하게 만들기' 같은 모호한 기준 대신 강력하고 명확한 성공 기준을 설정하라.

🌌 Project VEGA: The Sovereign SRE Agent (Master Blueprint)

"엔지니어의 직관과 AI의 추론을 결합한 리눅스 자율 운영 시스템"

1. 프로젝트 개요 (Overview)

정의: 자연어 입력을 리눅스 실행 명령어로 변환하고, 시스템 컨텍스트를 파악하여 안전하게 실행 및 보고하는 '공돌이 전용 자발적 비서'.

핵심 가치: "복잡한 문법은 AI가, 최종 결정과 실행은 사용자가."

철학: KISS (Keep It Simple, Stupid). 단일 정적 바이너리로 의존성 없이 어디서나 실행될 것.

SRE 운영 3대 원칙 (SRE Operating Principles):
- **Error Budgets**: "완벽한 시스템은 없다. 하지만 허용 가능한 장애 범위 내에서 최대한의 자동화를 추구한다."
- **Toil Reduction**: "반복되는 수동 작업(Toil)은 죄악이다. 모든 관리 행위는 코드로 정의(IaC)하고 VEGA가 집행한다."
- **Blameless Postmortems**: "장애는 시스템의 문제다. VEGA는 사고 발생 시 비난 대신 SQLite에 가중치 높은 로그를 남겨 미래의 자네를 지킨다."


2. 초기화 및 설정 (Bootstrap & Setup)

OS 인벤토리 스캔: 설치 즉시 사용자의 OS(Ubuntu/Fedora), 커널, 파티션 구조(3분할: Root, User, Media), 설치된 도구(rclone, docker 등)를 자동 스캔하여 컨텍스트화함.

AI 엔진 설정: 초기 실행 시 Gemini/OpenAI/Claude 중 선택 및 API 키를 로컬 보안 저장소(Keyring)에 보관.


쉘 통합: 사용자의 에일리어스(Alias)와 환경 변수를 파싱하여 vega 내부 실행 로직에 반영.

2-1. 실행 모드 (Execution Modes)

Direct Command (즉시 실행):
$ vega "현재 디렉토리의 1GB 이상 파일 찾아줘"

Interactive Shell (대화형 모드):
$ vega


3. 핵심 아키텍처: 3단계 추론 엔진 (Reasoning Engine)

Logical Scan (논리적 분석): 사용자의 의도와 대상 객체(파일, 프로세스, 경로 등) 식별.

Physical Mapping (물리적 대조): 대상 경로가 위치한 파티션(루트, 사용자, 미디어 저장용) 및 원격지(SSH/FTP) 여부 확인.

Privilege Enforcement (권한 및 보안 설계): '최소 권한' 원칙에 따라 최적의 보안 옵션(예: mount --bind -o ro)이 반영된 명령어 세트 생성.

4. 보안 및 시각적 가독성 (Safety & UI)

A. 시각적 위험 등급제 (Visual Alert)

등급아이콘컬러 (ANSI)대상 명령어 예시CRITICAL🚨Bold Red 배경rm -rf /, mkfs, dd, fdiskWARNING⚠️Yellow 텍스트chmod 777, kill -9, shutdownINFO✅Green 텍스트ls, df -h, apt update

B. 방어 레이어 (Defense Layers)

Explicit Confirmation: CRITICAL 등급은 단순 y가 아닌 YES 직접 입력을 요구.

Data Sanitization: AI API 전송 전 IP, 패스워드, 키값 등 민감 정보는 [REDACTED] 처리.

Hidden Secret: 민감 정보는 로컬에서만 변수 처리하여 실행 시점에 결합.

5. 지능형 리포팅 시스템 (Reporting & Analytics)

가중치 로깅: 단순 명령은 낮게, 복잡한 장애 복구 및 보안 설정은 높은 가중치를 부여하여 SQLite에 기록.

감성 리포트: "이번 달 VEGA가 당신의 서버를 15번 지켰고, 총 4시간의 삽질을 줄였습니다."

시각화: 터미널 내 ASCII 막대그래프(█) 및 점 도표를 활용해 주간/월간 통계 제공.

인사이트: 빈번하게 발생하는 장애(Disk Full 등)를 분석하여 선제적 처방 제안.

문서화: 모든 작업은 Markdown 보고서로 변환 가능하며, 메일/슬랙 자동 발송 지원.

6. 극한 환경 대응 (Offline & Recovery)

오프라인 진단: 네트워크 단절 시 AI 대신 하드코딩된 '자가 진단 룰셋' 가동.

그럽(GRUB) 복구 모드: 네트워크 가동 여부를 묻고 수동 복구 가이드 제공.

맥락 유지: 명령어가 없어 설치(apt/dnf) 후, 이전 작업 맥락을 기억해 다시 실행 여부 확인.

📈 7. 인텔리전트 리포팅 시스템 (Reporting & Analytics)
VEGA는 단순한 실행에 그치지 않고, 모든 활동을 가치 있는 데이터로 변환합니다. 이는 사용자의 성취감을 고취하고 후원을 유도하는 핵심 메커니즘입니다.

A. 세션 로깅 및 가중치 시스템 (Weighted Logging)
성공/실패 가중치: 단순 조회는 낮은 가중치, 복잡한 장애 복구 및 보안 설정은 높은 가중치를 부여하여 기록합니다.

데이터 저장: SQLite 기반으로 YYYY-MM.json 또는 .db 형태로 파티셔닝하여 로컬에 저장합니다.

B. 감성 기반 자동 리포트 (Emotional Reports)
일/주/월간 브리핑: "이번 주 VEGA가 당신의 서버를 15번 지켰고, 총 4.2시간의 삽질을 줄여주었습니다."

차트 시각화: 터미널 내에서 ASCII 막대그래프(█)와 점 도표를 활용해 사용 빈도와 성공률을 시각화합니다.

장애 인사이트: 로그를 분석하여 "최근 30일간 가장 빈번했던 장애: Disk Out of Space(40%)"와 같은 통계를 제공합니다.

C. 문서화 및 전송 (Documentation & Export)
Markdown 보고서: vega --report 실행 시, 해당 기간의 모든 작업 맥락을 기술 보고서 양식으로 출력합니다.

자동 메일링: AI가 요약한 리포트를 PDF나 Markdown 포맷으로 팀장이나 본인에게 즉시 발송합니다.

11. Phase 3 Implementation Specs (Context & Intelligence) [IN-PROGRESS]
A. Intelligence Hardening
- **SRE Persona**: Injects "20-year Senior Embedded Linux SRE" context into every request. Enforces brevity and KISS principle.
- **Chain-of-Thought (CoT)**: Enforces mandatory reasoning via `thought` field in JSON response schema.

B. Local RAG (Pseudo-Semantic Retrieval)
- **FTS5 Backend**: Synchronizes `chat_history` and `commands` to a SQLite FTS5 virtual table (`search_index`).
- **Context Injection**: Uses FTS5 `MATCH` queries to find the most relevant past commands/chats to inject into current prompts.

C. High-Availability Routing (Quota Fallback 2.0)
- **Persistent State**: Saves quota exhaustion timestamps to `~/.cache/vega/quota_state.json`.
- **Atomic Operations**: Implements write-to-temp-then-rename to prevent cache corruption.
- **Context Sync**: Summarizes and injects history when falling back between providers.

7. 개발 로드맵 (Roadmap)
Phase 1 (Foundation) [COMPLETED]: Unified System Context (Context.rs), SQLite Logging, Basic Safety Modules.

Phase 2 (Intelligence) [COMPLETED]: 멀티 AI 라우팅(OpenAI/Claude/Gemini), 비식별화 로직, ASCII 차트 리포트 엔진.

Phase 3 (Optimization) [COMPLETED]: Persona Hardening, CoT 강화, SQLite FTS5 기반 Local RAG, fzf 히스토리 UI.

Phase 4 (Discovery & Security) [COMPLETED]: Rust-based Discovery (lazy-lock.json), SSH Agent Inheritance, Unified Context Architecture.

Phase 5 (Enterprise) [PLANNED]: rclone 기반 Cloud Sync, 영구 Metadata 저장소, PDF/이메일 리포트 발송.

12. Phase 4 Implementation Specs (Discovery & Security) [COMPLETED]
- **Discovery**: Pure Rust implementation in `src/system/discovery.rs` (checking `lazy-lock.json`).
- **Context**: Consolidated all metadata into `src/context.rs`.

13. Phase 5 Implementation Specs (Cloud & Persistence) [PLANNED]
A. Cloud Sync Integration
- **Mechanism**: Leverage `rclone` for seamless, non-disruptive project backups and state synchronization.
- **Trigger**: Automated sync upon session completion or manual `vega sync` command.

B. Persistent Metadata
- **Storage**: Maintain a dedicated `metadata` table in SQLite for user-specific configurations and persistent system state (beyond FTS5 memory).
- **Security**: Sensitive metadata is encrypted at rest using local keyring-derived keys.

10. Phase 2 Implementation Specs (Intelligence) [COMPLETED]
A. AI Architecture
- Trait: `LLMProvider` (async) accepting `SystemContext` for persona injection.
- Security: `keyring` crate for credential management, with environment variable fallback.

B. Context-Aware Prompting
- System Persona: "You are VEGA, running on {OS}..." injected into every request.
- Context Serialization: OS, Kernel, and Partition counts are sent to the LLM.

C. Smart Routing Strategy
- Router: `SmartRouter` determines the best engine based on query context.
  - Deep Analysis ("analyze", "debug") -> Claude (Simulated fallback to Gemini).
  - Long Context (>1000 chars) -> Gemini.
  - Speed/Default -> Gemini Flash.
  - Offline/Network Fail -> `OfflineEngine` (Regex/Rule-based).
- Env Discovery: `EnvScanner` parses `~/.bashrc`, `~/.zshrc` for `*_API_KEY` exports.

8. 안티그래비티(작업자)를 위한 지시사항
Context First: 모든 명령은 사용자의 3분할 파티션 구조를 최우선으로 고려할 것.

No Dependency: musl 정적 빌드를 지향하여 의존성 에러 없는 단일 바이너리 유지.

Step-by-Step: 복잡한 작업은 반드시 '가독성 높은 브리핑'을 선제 제공한 후 승인을 받을 것.

Sync: 작업 완료 시 rclone을 통해 구글 드라이브에 코드를 즉시 동기화하여 시니어의 검수를 받을 것.




