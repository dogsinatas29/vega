🌌 Project VEGA: The Sovereign SRE Agent (Master Blueprint)

"엔지니어의 직관과 AI의 추론을 결합한 리눅스 자율 운영 시스템"

1. 프로젝트 개요 (Overview)

정의: 자연어 입력을 리눅스 실행 명령어로 변환하고, 시스템 컨텍스트를 파악하여 안전하게 실행 및 보고하는 '공돌이 전용 자발적 비서'.

핵심 가치: "복잡한 문법은 AI가, 최종 결정과 실행은 사용자가."

철학: KISS (Keep It Simple, Stupid). 단일 정적 바이너리로 의존성 없이 어디서나 실행될 것.

9. Phase 1 Implementation Specs (Technical Detail)
A. System Discovery Module (Context Awareness)
- Logic:
  - OS: Parses PRETTY_NAME from /etc/os-release. Fallback to "Unknown Linux" if file missing.
  - Partitions: Executes df -h.
    - Root: /
    - User: /home or mount points containing "User"/"Home"
    - Media: /media, /mnt, /run/media
- Error Handling: Failures in df or file reading return "Safe Defaults" (empty lists or "Unknown") instead of panicking.

B. Data Storage (SQLite Schema)
- File: vega.db (Created in CWD)
- Schema:
  - sessions(id PK, start_time, end_time, total_weight)
  - commands(id PK, session_id FK, command, ai_comment, weight, timestamp, success)
  - metadata(key PK, value)
- Weighting Logic:
  - Critical (20): rm -rf, mkfs, dd
  - Warning (7): systemctl, service
  - Install (5): apt, dnf, pacman
  - Info (1): ls, cd, echo

C. Safety Interceptor (Sanitizer & Barrier)
- Sanitizer (Regex Redaction):
  - IPv4: \d{1,3}\.\d{1,3}... -> [REDACTED_IP]
  - Email: ...@... -> [REDACTED_EMAIL]
  - Secrets: sk-..., Bearer ... -> [REDACTED_SECRET]
- Safety Barrier (UI):
  - CRITICAL: Requires case-sensitive "YES" input.
  - WARNING: Requires "y" input.
  - INFO: Auto-proceed.

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

7. 개발 로드맵 (Roadmap)
Phase 1 (Foundation) [COMPLETED]: OS 스캔 엔진, 로컬 파일 제어, SQLite 가중치 로깅, Safety Interceptor 구축.

Phase 2 (Intelligence): 멀티 AI 라우팅(OpenAI/Claude/Gemini), 비식별화 로직, ASCII 차트 리포트 엔진.

Phase 3 (Enterprise): SSH/FTP 원격 관리, PDF/이메일 리포트 발송, 하이브리드 클라우드 동기화.

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


## Current Architecture (Phase 1)
- **System Discovery**: `SystemContext` global singleton via `df -h` and `/etc/os-release`.
- **Storage**: SQLite (`vega.db`) with `rusqlite` (bundled). Tracks Sessions and Commands.
- **Safety**: `Sanitizer` (Regex) -> `Checker` (Risk Level) -> `SafetyUI` (Confirmation).

## Current Architecture (Phase 2)
- **AI**: `LLMProvider` trait + `GeminiProvider`.
- **Security**: `keyring` crate + `SystemContext` injection.
