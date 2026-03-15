# 🌌 Vega: The Sovereign SRE Agent

[English Documentation](README.md) | [개발 로드맵](ROADMAP_KR.md)

> **🚧 현재 상태**: QEMU에서 구동 중인 OS에 SSH로 접속하여 시스템 설정 작업을 테스트 중입니다.

> **"공돌이의 직관과 AI의 추론을 결합한 리눅스 자율 운영 시스템"**
>
> 데몬으로 상주하지 않고, 필요할 때만 호출되어 쉘 환경을 존중하는 경량 시스템 관리 에이전트입니다.

---

### 🛡️ 핵심 로직 업데이트: "탐색 우선 (Discovery First)"

> **"질문은 최후의 수단이다."**

VEGA는 네트워크(DHCP), 가상화 에이전트(QEMU Agent), ARP 테이블 등 가용한 모든 도구를 총동원해 스스로 정보를 확보합니다. 확보된 정보는 즉시 내부 상태 데이터베이스(State DB)에 기록되어 다음 작업의 맥락으로 활용됩니다.

- **동적 탐색 (Silent Discovery):** 불완전한 정보(예: IP 없음) 수신 시 즉시 백그라운드 탐색 수행.
- **상태 박제 (Resolve & Persist):** 찾아낸 시스템 정보는 즉시 기록하여 불필요한 재질의 원천 차단.
- **클라우드 동기화 (Cloud Sync):** `rclone` 기반의 무중단 프로젝트 백업 및 상태 동기화 기능.
- **하이브리드 실행 파이프라인 (v0.0.10):** 의도 파악부터 실행까지 7단계로 분리된 엔진(Intent -> AST -> AI Options -> Simulation -> Execution)으로 안전성 극대화.
- **사고 과정 기록 (Decision Lineage):** 명령이 왜 제안되었는지에 대한 모든 추론 과정을 DB에 영구 박제.
- **영구 메타데이터 (Persistent Metadata):** 시스템 고유 설정 및 장기 상태를 기억하는 전용 Metadata 테이블 운용.

### 📜 SRE 운영 3대 원칙
1. **Error Budgets**: "완벽한 시스템은 없다. 허용 가능한 장애 범위 내에서 최대한의 자동화를 추구한다."
2. **Toil Reduction**: "반복되는 수동 작업(Toil)은 죄악이다. 모든 관리 행위는 코드로 정의하고 VEGA가 집행한다."
3. **Blameless Postmortems**: "장애는 시스템의 문제다. VEGA는 비난 대신 로그를 남겨 미래의 당신을 지킨다."

---

## 🧠 핵심 아키텍처 (Hybrid Pipeline v0.0.10)

Vega는 AI의 유연성과 전통적인 시스템의 결정론적 제어를 결합한 **단계별 실행 파이프라인**을 통해 작동합니다.

1.  **의도 분석 (Intent Resolution)**: 자연어를 구조화된 작업(백업, 설치 등)으로 변환합니다. 복잡한 명령은 AI가 분석합니다.
2.  **템플릿 빌더 (Template Builder)**: AI에 의한 문법 오류를 방지하기 위해 결정론적인 **명령어 골격(AST)**을 생성합니다.
3.  **AI 옵션 생성 (Option Generator)**: AI는 골격에 주입될 최적의 옵션(예: `--checksum`, `--progress`)만 생성합니다.
4.  **가상 실행 엔진 (VEE)**: **실제 시스템 상태**를 확인(경로 존재 여부 등)하고 예상 파급력을 시뮬레이션합니다.
5.  **위험 평가 (Risk Evaluation)**: 위험 점수(0-100)를 산출합니다. 위험도가 높으면 명시적 승인을 요구합니다.
6.  **실행 프로바이더 (Execution)**: 로컬 쉘 또는 원격 SSH 환경에서 명령을 실제로 집행합니다.
7.  **리포팅 및 이력 관리 (Lineage)**: 모든 추론 근거를 기록하고 기술적인 SRE 보고서를 생성합니다.

---

## 📦 빌드 사전 요구사항

소스에서 빌드하기 전에 필요한 개발 패키지를 설치하세요:

```bash
# Fedora / RHEL / CentOS
sudo dnf install -y openssl-devel pkgconfig sqlite-devel

# Ubuntu / Debian
sudo apt install -y libssl-dev pkg-config libsqlite3-dev sqlite3

# Arch Linux
sudo pacman -S openssl pkg-config sqlite
```

---

## ⚡ 설치 방법

Vega는 단일 정적 바이너리로 빌드됩니다. 런타임 의존성은 필요하지 않습니다.

```bash
# 1. 릴리스 바이너리 빌드
cargo build --release

# 2. 로컬 bin 디렉토리 생성 (없는 경우)
mkdir -p ~/.local/bin

# 3. 바이너리 복사
cp target/release/vega ~/.local/bin/

# 4. PATH 추가 (필요한 경우)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# 5. 설치 확인
vega --help
```

---

## 🛠️ 사용 방법

### 1. 초기 설정 (Setup)
대화형 마법사를 실행하여 API 키와 설정을 구성합니다.
```bash
vega setup
```

### 2. 구글 로그인 및 할당량 관리
구글 계정으로 로그인하여 표준 API 키 제한을 우회하고 프로젝트 기반 할당량을 활용하세요.
```bash
vega login
```

- **높은 할당량**: 유료 계정은 무료 키보다 훨씬 높은 RPM을 제공받습니다.
- **자동 갱신**: OAuth2 Refresh Token을 사용하여 세션을 끊김 없이 관리합니다.
- **엔터프라이즈**: GCP "Application Default Credentials" (ADC) 탐색 기능을 지원합니다.

### 3. 히스토리 및 메모리 관리
`fzf` 인터페이스를 통해 과거의 작업 기록을 검색하고 명령어를 즉시 재실행하세요.
```bash
vega history
```

### 4. 자연어 명령 (Natural Language)
```bash
vega "현재 디렉토리에서 1GB 이상인 파일 찾아줘"
```

1.  **의도 (Intent)**: "무엇을" 할지 구조화된 데이터로 추출합니다.
2.  **시뮬레이션 (Simulation)**: VEE가 실제 경로 존재 여부를 로컬에서 검증합니다.
3.  **제안 (Proposal)**: AI가 최적화된 옵션(Flags)을 제안합니다.
4.  **기록 (Audit)**: 실행 전 모든 결정 근거(Lineage)를 로깅합니다.

---

## 📊 SRE 보고서 예시

VEGA는 세션 중 기록된 **Decision Lineage**를 바탕으로 고밀도 기술 보고서를 생성합니다.

```markdown
# 🌌 VEGA Maintenance Report
**Session ID:** SID-1042 | **Risk Level:** 🟡 MEDIUM

### 🧠 Decision Lineage
- **Request:** "현재 디렉토리를 serverA에 백업해줘"
- **Intent:** `Tool: rclone`, `Op: sync`, `Target: serverA`
- **VEE Simulation:** 원본 경로 존재 확인. (정상)
- **AI 제안 옵션:** `["--progress", "--checksum", "--fast-list"]`
- **Result:** ✅ SUCCESS (추론 이력 저장됨)
```

---

## ☁️ 클라우드 통합 (rclone)

VEGA는 `rclone`을 활용하여 무중단 프로젝트 백업 및 상태 동기화를 지원합니다.

### 1. 사전 준비
- 시스템에 `rclone`이 설치되어 있어야 합니다: `sudo dnf install rclone` (Fedora) 또는 `sudo apt install rclone` (Ubuntu).
- 클라우드 리모트를 설정합니다: `rclone config`.

### 2. 자율 탐색 (Autonomous Discovery)
VEGA의 탐색 엔진은 활성화된 `rclone` 리모트를 자동으로 식별하며, AI와 통신할 때 민감한 리모트 이름을 마스킹(예: `gdrive:` -> `REMOTE_01`)하여 보안을 유지합니다.

### 3. 기본 리모트 설정 (Primary Remote Setup)
특정 리모트를 기본 동기화 대상으로 "고정"할 수 있습니다. VEGA는 이 주소를 기억하고 프로젝트 단위 동기화 시 최우선적으로 사용합니다.
- `vega setup`을 실행합니다.
- **[2] Cloud Integration** 단계에서 원하는 클라우드 리모트를 선택합니다.
- 선택한 값은 `config.toml`의 `primary_remote` 필드에 저장됩니다.

### 3. 자연어 클라우드 작업
자연어로 클라우드 저장소와 상호작용할 수 있습니다. VEGA는 실행 직전에 마스킹된 이름을 원래의 리모트 이름으로 자동 복원합니다.
```bash
# 예시: 구글 드라이브에서 폴더 복사
vega "구글 드라이브의 'input' 폴더를 여기로 복사해줘"

# 예시: 현재 프로젝트를 클라우드와 동기화
vega sync
```

### 4. 안전 가드레일 (Safety Guardrails)
- **용량 제한**: 데이터 비용 및 오버헤드 방지를 위해, 동기화 크기가 **1GB**를 초과할 경우 자동으로 차단됩니다.
- **사용자 승인**: 모든 클라우드 작업은 실행 전 사용자의 명시적 확인을 거칩니다.

---

## 📋 내부 명령어

| 명령어 | 설명 |
| :--- | :--- |
| `setup` | 설정 마법사 실행 |
| `login` | 구글 OAuth2 인증 및 로그인 |
| `history` | fzf 기반 대화형 히스토리 UI |
| `install <pkg>` | 패키지 설치 (매니저 자동 감지) |
| `connect <host>` | 컨텍스트 메모리 기반 SSH 연결 |
| `status` | 시스템 상태 대시보드 |
| `health` | 로그 분석 및 해결책 제안 |
| `backup <src> <dst>` | 검증 기능을 포함한 스마트 백업 |
| `refresh <target>` | SSH 호스트 정보 갱신 |
| `update --all` | 시스템 패키지 일괄 업데이트 |
| `sync` | rclone 기반 클라우드 프로젝트 및 상태 동기화 |
| `config` | 쉘 환경 스냅샷 동기화 |

---

## 🛡️ 보안 기능

*   **명시적 승인**: 치명적인 명령어(`rm`, `dd`)는 "YES" 입력을 요구합니다.
*   **데이터 비식별화**: API 전송 전 IP, 키 등 민감 정보는 마스킹 처리됩니다.
*   **로컬 처리**: 단순 명령은 인터넷 연결 없이 로컬에서 즉시 안전하게 처리됩니다.

---

## 📂 프로젝트 구조 및 파일 역할

`src` 디렉토리의 핵심 컴포넌트와 그 기능은 다음과 같습니다:

### 🛠️ 핵심 인프라 (Core Infrastructure)
*   [`main.rs`](src/main.rs): 애플리케이션 진입점. CLI 인자 파싱 및 최상위 명령어 라우팅을 담당합니다.
*   [`context.rs`](src/context.rs): VEGA의 '자기 인식'의 핵심. OS, 하드웨어, 네트워크 메타데이터를 관리합니다.
*   [`init.rs`](src/init.rs): 부트스트랩 프로세스를 조율하며 DB와 설정 파일의 준비 상태를 보장합니다.
*   [`config.rs`](src/config.rs): `vega.toml` 설정 파일의 로드 및 검증을 처리합니다.

### 🧠 AI 및 추론 (AI & Reasoning) (`src/ai`)
*   [`router.rs`](src/ai/router.rs): 쿼리의 복잡도에 따라 어떤 AI 엔진을 사용할지 결정하는 로직입니다.
*   [`providers/`](src/ai/providers/): Gemini, Claude, 그리고 로컬 정규표현식 기반 엔진을 위한 전용 커넥터들입니다.
*   [`prompts.rs`](src/ai/prompts.rs): LLM 프롬프트를 위한 시스템 페르소나 및 컨텍스트 주입을 관리합니다.

### 🚀 실행 레이어 (Execution Layer) (`src/executor`)
*   [`orchestrator.rs`](src/executor/orchestrator.rs): 다단계 복구 작업을 포함한 작업 실행의 생명주기를 관리합니다.
*   [`pkg.rs`](src/executor/pkg.rs): 다양한 배포판(apt, dnf, pacman) 간 호환성을 위한 추상화된 패키지 매니저입니다.
*   [`healer.rs`](src/executor/healer.rs): 실패를 분석하고 자동화된 해결책을 제안하는 로직입니다.

### 🔍 시스템 인텔리전스 (System Intelligence) (`src/system`)
*   [`discovery.rs`](src/system/discovery.rs): 프로젝트별 메타데이터(예: Node/Rust 프로젝트)를 자율적으로 스캔합니다.
*   [`archivist.rs`](src/system/archivist.rs): 추론 기록 및 시스템 스냅샷의 장기 저장을 관리합니다.
*   [`env_scanner.rs`](src/system/env_scanner.rs): `.bashrc` 및 `.zshrc`를 깊이 분석하여 사용자의 커스텀 환경을 이해합니다.

### 🛡️ 안전 및 보안 (Safety & Security)
*   `src/safety/`: 위험한 패턴 목록에 대해 명령어를 검증하는 **Safety Registry**가 포함되어 있습니다.
*   `src/security/`: 민감 정보 비식별화 및 `keyring` 관리 핸들러입니다.

### 💾 저장소 및 지식 (Storage & Knowledge)
*   `src/storage/`: SQLite 백엔드와의 직접적인 상호작용을 담당합니다.
*   [`knowledge.rs`](src/knowledge.rs): 로컬 RAG 시스템 및 FTS5 검색 인덱스 관리자입니다.

---

## 📄 라이선스

GPL-3.0 라이선스.
