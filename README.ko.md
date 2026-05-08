# 🌌 Vega: The Sovereign SRE Agent

[![Vega Demo](https://img.shields.io/badge/YouTube-Shorts-red?style=for-the-badge&logo=youtube)](https://youtube.com/shorts/6a-fscWTTVo?si=B4wqRKqZbGJbtn4Z)

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
- **하이브리드 실행 파이프라인 (v0.0.14.9):** 고도화된 컨텍스트 인지 및 보안 가드레일이 통합된 차세대 실행 엔진.
- **사고 과정 기록 (Decision Lineage):** 명령이 왜 제안되었는지에 대한 모든 추론 과정을 DB에 영구 박제.
- **영구 메타데이터 (Persistent Metadata):** 시스템 고유 설정 및 장기 상태를 기억하는 전용 Metadata 테이블 운용.
- **풀 스택 SRE 진단 (v0.0.14.8):** OS/하드웨어/네트워크를 망라하는 AI 기반 기술 분석 리포트 자동 생성.
- **동적 로컬라이제이션 (v0.0.14.9):** 시스템 언어(Locale)를 자동 감지하여 프롬프트 및 리포트 언어 동적 최적화 (한국어 완벽 지원).
- **지능형 포트 가드레일:** SSH 명령어 시 AI의 포트 혼선(11434 vs 22)을 자동으로 교정하는 SRE 보안 로직.
- **Llama 3.1 하드닝:** 8B급 로컬 모델의 지시 이행 능력 및 한국어 무결성을 위한 전용 페르소나 주입.
- **원격 SRE 자율 제어 (v0.1.6):** 보안 내장 SSH 엔진(`ssh2`)을 통한 암호 영구 저장 및 `sudo` 자동 암호 주입 제어.
- **원샷 원격 정밀 진단:** 단일 세션 멀티 커맨드 기술을 통한 10배 빠른 원격 지표 수집 및 분석.
- **인프라 인지 (v0.0.14):** 물리적 자원(CPU/RAM/Disk)의 심층 감지 및 도구별 요구사항에 기반한 선제적 유효성 검사 구현.

### 📜 SRE 운영 3대 원칙
1. **Error Budgets**: "완벽한 시스템은 없다. 허용 가능한 장애 범위 내에서 최대한의 자동화를 추구한다."
2. **Toil Reduction**: "반복되는 수동 작업(Toil)은 죄악이다. 모든 관리 행위는 코드로 정의하고 VEGA가 집행한다."
3. **Blameless Postmortems**: "장애는 시스템의 문제다. VEGA는 비난 대신 로그를 남겨 미래의 당신을 지킨다."

---

## 🧠 핵심 아키텍처 (Hybrid Pipeline v0.0.14)

Vega는 AI의 유연성과 전통적인 시스템의 결정론적 제어를 결합한 **단계별 실행 파이프라인**을 통해 작동합니다.

1.  **의도 분석 (Intent Resolution)**: 자연어를 구조화된 작업(백업, 설치 등)으로 변환합니다. 복잡한 명령은 AI가 분석합니다.
2.  **인프라 감지 및 검증 (v0.0.14)**: 대상 호스트의 물리적 자원(CPU/RAM/Disk) 정보를 정밀하게 감지하거나, 단순 제어 작업 시 **Thin Action Execution** (스냅샷 우회)을 수행합니다.
3.  **환경 인지 추론 (Environment-Aware Cognition)**: "ssh 노드", "원격 서버" 등 추상적 지시어를 관리 중인 인프라 인벤토리로 지능적으로 매핑(Semantic Grounding)합니다.
4.  **템플릿 빌더 (Template Builder)**: AI에 의한 문법 오류를 방지하기 위해 결정론적인 **명령어 골격(AST)**을 생성합니다.
5.  **AI 옵션 생성 (Option Generator)**: AI는 골격에 주입될 최적의 옵션(예: `--checksum`, `--progress`)만 생성합니다.
6.  **가상 실행 엔진 (VEE)**: **실제 시스템 상태**를 확인(경로 존재 여부 등)하고 예상 파급력을 시뮬레이션합니다.
7.  **위험 평가 (Risk Evaluation)**: 위험 점수(0-100)를 산출합니다. 위험도가 높으면 명시적 승인을 요구합니다.
8.  **실행 및 RAW 가시성 (v0.0.14)**: 로컬/원격(SSH) 환경에 명령을 집행하며, **직접 스트림 캡처**(STDOUT/STDERR/EXIT_CODE)를 통해 투명성을 확보합니다.
9.  **목표 상태 화해 (Desired State Reconciliation)**: 단순 Exit Code가 아닌 세만틱 평가를 통해 목표 상태 만족 여부(예: "이미 삭제됨"은 성공)를 판별합니다.
10. **리포팅 및 이력 관리 (Lineage)**: 모든 추론 근거를 기록하고 **AI 기반 SRE 5단계 리포트**를 생성합니다.

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

Vega은 단일 정적 바이너리로 빌드됩니다. 런타임 의존성은 필요하지 않습니다.

```bash
# 1. 저장소 복제 (Clone)
git clone https://github.com/dogsinatas29/vega
cd vega

# 2. 릴리스 바이너리 빌드
cargo build --release

# 3. 로컬 bin 디렉토리 생성 (없는 경우)
mkdir -p ~/.local/bin

# 4. 바이너리 복사
cp target/release/vega ~/.local/bin/

# 5. PATH 추가 (필요한 경우)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# 6. 설치 확인
vega --help

# 7. API 키 및 환경 설정
# 대화형 마법사를 실행하여 Gemini/OpenAI/Claude 키를 설정합니다.
vega setup
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

1.  **타겟 인지 (Cognition)**: 사용자의 요청을 로컬 또는 원격 노드 중 어디에서 처리할지 지능적으로 결정합니다.
2.  **의도 분석 및 시뮬레이션 (Intent & Simulation)**: "무엇을" 할지 정의하고, VEE가 경로 존재 여부 등 시스템 상태를 선제 검증합니다.
3.  **옵션 최적화 (Optimization)**: AI가 해당 환경에 가장 효율적인 명령어 옵션(Flags)을 제안합니다.
4.  **목표 상태 화해 (Reconciliation)**: 단순 Exit Code가 아닌, 목표한 상태(예: 파일 삭제됨)가 달성되었는지 세만틱하게 평가합니다.
5.  **기록 및 이력 관리 (Audit & Lineage)**: 모든 추론 과정과 실행 결과를 State DB에 영구 기록합니다.

---

## 📊 SRE 보고서 예시 (기술 명세)

VEGA는 세션 중 기록된 **Decision Lineage**를 바탕으로 고밀도 기술 보고서를 생성합니다. 다음은 생성된 세션 요약의 예시입니다:

```markdown
# 🌌 VEGA 유지보수 세션 보고서
**Session ID:** `SID-1042` | **날짜:** 2026-03-15 | **위험 등급:** 🟡 MEDIUM

---

## 🧠 의사결정 이력 (Decision Lineage)

### 1. 요청: "현재 디렉토리를 serverA에 백업해줘"
- **[Intent]** `HybridResolver`: `Tool: rclone`, `Op: sync`, `Target: REMOTE_01` 식별
- **[Sim]** `VEE`: 로컬 경로 `/home/user/project` 존재 확인. 크기: 450MB. (Safe)
- **[Risk]** `Evaluator`: 점수 20 (Info). 자동 승인.
- **[Final Command]** `rclone sync ./ serverA:backup/vega_sync --progress --checksum --fast-list`
- **[Result]** ✅ SUCCESS (추론 이력 저장됨)

### 2. 요청: "rm -rf /var/log"
- **[Intent]** `LocalResolver`: `Tool: coreutils`, `Op: delete`, `Target: /var/log` 식별
- **[Sim]** `VEE`: **CRITICAL**. 시스템 로그 디렉토리의 재귀적 삭제 감지.
- **[Risk]** `Evaluator`: **점수 100 (CRITICAL)**. 
- **[Status]** 🛑 **거부됨** (안전 가드레일에 의해 차단)

---

## 📈 영향 분석 (ASCII 시각화)
세션 위험 분포:
```text
[CRITICAL]  | ██████████ (33%) -> 차단됨
[WARNING]   | (0%)
[INFO]      | ████████████████████ (67%) -> 실행됨

위험 점수 히트맵:
0 [##########          ] 100
평균: 40.0 (MEDIUM)
```

## 💡 SRE 인사이트
- **안전 가드레일 효율:** 100% (모든 고위험 작업이 사전에 차단됨).
- **자동화 정확도:** AI가 복잡한 자연어 의도를 정확하게 분석함.
- **성과:** 1개의 시스템 치명적 데이터 손실 사고를 예방함.

---

## 📧 상급자 보고용 메일 브리핑 (템플릿)

VEGA는 `vega --report --email` 명령어를 통해 세션 이력을 기반으로 상급자나 SRE 팀장에게 즉시 발송 가능한 요약 보고서 형태를 제공합니다.

```text
제목: [SRE 주간 브리핑] 시스템 유지보수 세션 요약 보고 - 2026-03-15 (SID-1042)

팀장님,

VEGA SRE 에이전트를 사용하여 마스터 노드의 정기 유지보수 세션을 완료했습니다.
세션 요약 내용은 다음과 같습니다:

1. 시스템 상태: 로컬 프로젝트의 클라우드 백업(serverA) 및 동기화 완료.
2. 리스크 관리: 1건의 치명적(CRITICAL) 작업(비인가 시스템 로그 삭제 시도) 감지 및 차단.
3. 기술 요약:
   - 작업 성공률: 100% (허가된 2건의 작업 모두 성공)
   - 안전 개입: 1건의 고위험 명령어 사전 차단으로 시스템 안정성 확보.
   - 추론 이력: 모든 결정 근거(Lineage)가 State DB에 박제 및 검증됨.

상세 기술 리포트는 세션 이력(SID-1042)에 아카이브되어 있으며, 필요 시 첨부파일로 확인 가능합니다.

보고 드립니다.
[사용자 이름] 드림
```
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

## 📋 명령어 레퍼런스 (SRE Playbook)

| 카테고리 | 명령어 | 설명 |
| :--- | :--- | :--- |
| **플릿 관리** | `vega status` | 실시간 대시보드 출력 (포트, 로드, 태그 표시) |
| | `vega status <target>` | **심층 진단(Deep Scan)**: SSH를 통한 실시간 OS, 커널, 디스크 정보 스캔 |
| | `vega add-node <ip>` | 새로운 SSH 노드를 장부(KB)에 수동 등록 |
| | `vega sync-ssh` | 장부의 노드 정보를 `~/.ssh/config`에 동기화 (포트 인지형) |
| | `vega update --fleet` | 플릿 전체 유지보수 수행 (커널 업데이트, 로드 동기화) |
| **인텔리전스** | `vega report --sre` | **5단계 SRE 리포트**: AI 기반 세션 이력 분석 및 보고서 생성 |
| | `vega history` | `fzf` 기반의 대화형 히스토리 UI |
| | `vega health` | 시스템 로그 분석 및 자동화된 해결책 제안 |
| **프로젝트** | `vega refresh` | 글로벌 갱신 (노드 재탐색 및 쉘 환경 스냅샷) |
| | `vega sync` | `rclone` 기반의 프로젝트 및 상태 클라우드 동기화 |
| | `vega backup <src> <dst>` | 검증 및 위험 평가 기능이 포함된 스마트 백업 |
| **설정** | `vega setup` | 대화형 초기 설정 마법사 실행 |
| | `vega login` | 구글 OAuth2 기반 인증 및 로그인 |
| | `vega config` | 쉘 환경 스냅샷 수동 동기화 |

---

## 🔑 SSH 키 설정 (권장)

VEGA는 결정론적이고 비대화형 실행을 보장하기 위해 모든 원격 작업에 `BatchMode=yes`를 사용합니다. 따라서 `localhost`를 포함한 모든 관리 대상 노드에는 **공개키 인증(Public Key Authentication)**이 설정되어 있어야 합니다.

### 1. 로컬 공개키 확인
먼저 로컬에 SSH 키가 존재하는지 확인합니다:
```bash
# ed25519 또는 rsa 키 확인
cat ~/.ssh/id_ed25519.pub || cat ~/.ssh/id_rsa.pub
```
*키가 없다면 `ssh-keygen -t ed25519` 명령어로 생성하세요.*

### 2. 대상 노드에 키 복사
`ssh-copy-id`를 사용하여 원격 서버에 공개키를 등록합니다. 이를 통해 VEGA 작업 중 패스워드 입력 없이 자율적인 실행이 가능해집니다.
```bash
# 문법: ssh-copy-id <사용자>@<호스트>
ssh-copy-id dogsinatas@192.168.0.150
```

### 3. 비밀번호 없는 접속 확인
패스워드 입력 없이 로그인이 가능한지 최종 확인합니다:
```bash
ssh dogsinatas@192.168.0.150
```
*접속에 성공하면 VEGA가 해당 노드를 자율적으로 관리할 수 있는 준비가 된 것입니다.*

### 🛠️ `localhost` 접속 트러블슈팅
만약 `ssh localhost`가 `Permission denied`로 실패한다면, 로컬 환경이 BatchMode를 지원하도록 설정해야 합니다:
1. **authorized_keys 설정**: `cat ~/.ssh/id_ed25519.pub >> ~/.ssh/authorized_keys`
2. **ssh-agent 가동**: `eval $(ssh-agent -s) && ssh-add ~/.ssh/id_ed25519`
3. **SSH 서버 확인**: 로컬 시스템을 SSH를 통해 관리하려면 `sshd`가 실행 중이어야 합니다.

---

## 🛡️ 보안 기능

*   **명시적 승인**: 치명적인 명령어(`rm`, `dd`)는 "YES" 입력을 요구합니다.
*   **데이터 비식별화**: API 전송 전 IP, 키 등 민감 정보는 마스킹 처리됩니다.
*   **무중단 SSH**: `-o BatchMode=yes` 강제 적용으로 자동화 환경에서의 대기 및 좀비 세션 방지.
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

## 📊 VEGA SRE 리포트 (Reports)

Vega는 시스템 가시성과 추론 투명성을 보장하기 위해 두 가지 유형의 AI 기반 SRE 리포트를 제공합니다.

### 1. 시스템 진단 보고서 (System Diagnostic - 현재 지원)
대상 시스템(로컬 또는 원격)의 실시간 "맥박"을 제공합니다.
*   **실행**: `vega status <target>` 또는 자연어 명령 ("192.168.0.150 시스템 정보 알려줘")

```markdown
# 🚀 VEGA SRE System Report
**하드웨어 맥박 (Hardware Pulse)**
- **OS**: Ubuntu 25.10
- **CPU**: Intel(R) Core(TM) i7-4790 CPU @ 3.60GHz
- **RAM**: 1.41 GB / 15.07 GB (Used/Total)
- **가동 시간**: up 6 hours, 35 minutes

**네트워크 맵 (활성 포트)**
- **22 (ssh)**: 열림 (표준 관리 포트)
- **11434 (ollama)**: 활성 (AI/API 서비스)
```

### 2. 의도 및 문제 분석 보고서 (Decision & Analysis - 작업 예정)
세션의 추론 과정(Lineage)을 분석하고 SRE 5단계 분석을 통해 "왜"와 "어떻게"에 집중합니다.

```text
# 🌌 VEGA Maintenance Session Report
**SID-1042** | **위험 등급: 🟡 MEDIUM**

## 🧠 추론 이력 (Decision Lineage)
1. 요청: "현재 디렉토리를 serverA로 백업해줘"
- [의도] rclone sync 도구 식별
- [위험] 점수 20 (Info). 자동 승인.
- [최종 명령] rclone sync ./ serverA:backup/vega_sync

## 📄 SRE 5단계 분석
1. 현안: 11434 포트에서 Ollama 서비스가 작동 중입니다.
2. 원인: API 서비스 노출.
3. 해결: 보안 설정 검토 및 접근 제한 권장.
```

---

## 📄 라이선스 (License)

GPL-3.0 라이선스.
