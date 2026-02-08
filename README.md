# 🌌 Vega: The Sovereign SRE Agent

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)]()
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)]()

> **"The Pocket Knife Strategy"**
>
> A non-resident, lightweight system administration agent that respects your shell environment. Refuses to be a daemon.
>
> **"공돌이의 직관과 AI의 추론을 결합한 리눅스 자율 운영 시스템"**
> 데몬으로 상주하지 않고, 필요할 때만 호출되어 쉘 환경을 존중하는 경량 시스템 관리 에이전트입니다.

---

### 🛡️ Core Logic Update: "Discovery First"

> **"질문은 최후의 수단이다."** (Questions are a last resort.)

VEGA는 네트워크(DHCP), 가상화 에이전트(QEMU Agent), ARP 테이블 등 가용한 모든 도구를 총동원해 스스로 정보를 확보합니다. 확보된 정보는 즉시 내부 상태 데이터베이스(State DB)에 기록되어 다음 작업의 맥락으로 활용됩니다.

- **동적 탐색 (Silent Discovery):** 불완전한 정보(예: IP 없음) 수신 시 즉시 백그라운드 탐색 수행.
- **상태 박제 (Resolve & Persist):** 찾아낸 시스템 정보는 즉시 기록하여 불필요한 재질의 원천 차단.
- **동적 데이터베이스 (State DB):** 사고 과정을 보여주는 대시보드를 넘어, 스스로 찾아낸 시스템 정보의 박제소 역할.

---

## 🧠 Core Architecture / 핵심 아키텍처

Vega operates on a 3-stage **Reasoning Engine** to ensure safety and accuracy.
Vega는 안전하고 정확한 실행을 위해 3단계 **추론 엔진**을 기반으로 작동합니다.

![Vega Logic Flow](assets/logic_flow.png)

1.  **Logical Scan (논리적 분석)**
    *   **Goal**: Understand user intent and identify target objects (files, processes, paths).
    *   **Korean**: 사용자의 의도를 파악하고 대상 객체(파일, 프로세스, 경로)를 식별합니다.
2.  **Physical Mapping (물리적 대조)**
    *   **Goal**: Map targets to physical resources (Partitions, SSH hosts) and verify existence.
    *   **Korean**: 대상이 실제 존재하는지, 어느 파티션이나 원격지에 위치하는지 확인합니다.
3.  **Privilege Enforcement (권한 및 보안)**
    *   **Goal**: Apply 'Least Privilege' principles and generate the safest possible command.
    *   **Korean**: '최소 권한' 원칙을 적용하여 가장 안전한 실행 명령어를 생성합니다.

---

## ⚡ Installation / 설치 방법

Vega is built as a single static binary. No dependencies required.
Vega는 단일 정적 바이너리로 빌드됩니다. 별도의 의존성이 필요하지 않습니다.

```bash
# 1. Build Release Binary
cargo build --release

# 2. Install to local bin
cp target/release/vega ~/.local/bin/
```

---

## 🛠️ Usage / 사용 방법

### 1. Setup / 초기 설정
Launch the interactive wizard to configure API keys and preferences.
대화형 마법사를 실행하여 API 키와 설정을 구성합니다.

```bash
vega setup
```

### 2. Natural Language Command / 자연어 명령
Ask Vega to perform tasks using plain English or Korean.
평범한 자연어로 작업을 요청하세요.

```bash
# English
vega "Find all files larger than 1GB in /home"

# Korean
vega "현재 디렉토리에서 1GB 이상인 파일 찾아줘"
```

### 3. System Monitor / 시스템 모니터
Visualize your system load in a DooM-style 3D interface.
둠(Doom) 스타일의 3D 인터페이스로 시스템 부하를 시각화합니다.

```bash
vega monitor
```

---

## 📋 Internal Commands / 내부 명령어

Vega provides several built-in commands for direct control.
Vega는 직접 제어를 위한 다양한 내장 명령어를 제공합니다.

| Command / 명령어 | Description (EN) | Description (KR) |
| :--- | :--- | :--- |
| `setup` | Run the configuration wizard | 설정 마법사 실행 |
| `install <pkg>` | Install packages (detects apt/dnf/pacman) | 패키지 설치 (패키지 매니저 자동 감지) |
| `connect <host>` | SSH connection with context memory | 컨텍스트 메모리를 활용한 SSH 연결 |
| `status` | Show system status dashboard | 시스템 상태 대시보드 표시 |
| `monitor` | Launch 3D System Monitor | 3D 시스템 모니터 실행 |
| `health` | Analyze system logs and suggest fixes | 시스템 로그 분석 및 해결책 제안 |
| `backup <src> <dst>` | Smart backup with validation | 검증 과정을 포함한 스마트 백업 |
| `refresh <target>` | Refresh SSH host context | SSH 호스트 컨텍스트 갱신 |
| `update --all` | Update system packages | 시스템 패키지 일괄 업데이트 |
| `config` | Sync shell environment snapshot | 쉘 환경 스냅샷 동기화 |

---

## 🛡️ Safety Features / 보안 기능

*   **Explicit Confirmation**: Critical commands (`rm`, `dd`) require typing "YES".
*   **Data Redaction**: Sensitive data (IPs, Keys) is redacted before sending to AI.
*   **Local Processing**: Simple commands match locally (Regex/Fuzzy) without API calls.

*   **명시적 승인**: 치명적인 명령어(`rm`, `dd`)는 "YES"를 입력해야 실행됩니다.
*   **데이터 비식별화**: 민감한 정보(IP, 키)는 AI 전송 전 마스킹 처리됩니다.
*   **로컬 처리**: 단순 명령어는 API 호출 없이 로컬에서 즉시 매칭됩니다.

---

## 📄 License

MIT License.
