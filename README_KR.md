# 🌌 Vega: The Sovereign SRE Agent

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)]()
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)]()

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
- **영구 메타데이터 (Persistent Metadata):** 시스템 고유 설정 및 장기 상태를 기억하는 전용 Metadata 테이블 운용.
- **동적 데이터베이스 (State DB):** 사고 과정을 보여주는 대시보드를 넘어, 스스로 찾아낸 시스템 정보의 박제소 역할.

### 📜 SRE 운영 3대 원칙
1. **Error Budgets**: "완벽한 시스템은 없다. 허용 가능한 장애 범위 내에서 최대한의 자동화를 추구한다."
2. **Toil Reduction**: "반복되는 수동 작업(Toil)은 죄악이다. 모든 관리 행위는 코드로 정의하고 VEGA가 집행한다."
3. **Blameless Postmortems**: "장애는 시스템의 문제다. VEGA는 비난 대신 로그를 남겨 미래의 당신을 지킨다."

---

## 🧠 핵심 아키텍처

Vega는 안전하고 정확한 실행을 위해 3단계 **추론 엔진**을 기반으로 작동합니다.

1.  **Logical Scan (논리적 분석)**: 사용자의 의도를 파악하고 대상 객체(파일, 프로세스, 경로)를 식별합니다.
2.  **Physical Mapping (물리적 대조)**: 대상이 실제 존재하는지, 어느 파티션이나 원격지에 위치하는지 확인합니다.
3.  **Privilege Enforcement (권한 및 보안)**: '최소 권한' 원칙을 적용하여 가장 안전한 실행 명령어를 생성합니다.

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

1.  **추론 (CoT)**: AI의 내부 사고 과정을 먼저 보여줍니다.
2.  **분석**: 자연어 요청을 분석하여 기술적인 계획을 세웁니다.
3.  **제안**: 위험 등급(`Risk Level`)과 함께 실행 명령어를 제안합니다.
4.  **승인**: 사용자가 실행 여부를 최종 결정합니다.

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

## 📄 라이선스

MIT License.
