# 🚀 Vega Next Phase Roadmap

## 📌 현재 상태

Vega는 단순 SSH 원격 접속 도구 단계를 넘어 다음 수준까지 도달했다.

```text
SSH Remote Terminal
    ↓
Remote AI Operations Runtime
```

현재 이미 가능한 작업:

- SSH 원격 명령 실행
- sudo 기반 권한 상승
- 서비스 상태 확인
- Ollama 모델 설치/삭제
- 활성 모델 감지
- 서버 Shutdown
- 실시간 Progress Streaming

이제부터 Vega는 실제 인프라를 제어하는 시스템이 된다.

따라서 다음 단계의 핵심은:

- 기능 추가
- UI 추가

가 아니라:

- 실행 안전성
- 상태 정합성
- 작업 추적
- Rollback 가능성
- 오케스트레이션 구조화

이다.

---

# 🎯 최우선 작업

# 1. Action Execution Framework 구축

현재 구조:

```text
Button → SSH Command
```

다음 목표 구조:

```text
Action
 → Validate
 → Plan
 → Execute
 → Observe
 → Rollback
```

---

## 추천 Trait 구조

```rust
trait Action {
    fn validate(&self) -> Result<()>;
    fn plan(&self) -> ExecutionPlan;
    fn execute(&self) -> Result<ActionResult>;
    fn rollback(&self) -> Result<()>;
}
```

---

## 모든 작업을 Action으로 통합

예시:

```text
pull model
delete model
install package
restart service
reboot
shutdown
docker pull
nginx reload
```

이 구조를 먼저 고정하지 않으면 이후 확장에서 구조 붕괴가 발생한다.

---

# 2. Host Capability Scanner

현재 Vega는 서버에 명령을 보내는 수준이다.

다음 단계에서는 서버 상태를 이해해야 한다.

---

## 수집 대상

```text
OS
Kernel
CPU Architecture
RAM
VRAM
GPU
Disk
Installed Services
Open Ports
Running Models
```

---

## 추천 구조

```rust
struct HostSnapshot {
    system: SystemInfo,
    hardware: HardwareInfo,
    services: Vec<ServiceInfo>,
    ai_runtime: AIRuntimeInfo,
}
```

---

## 목적

- 모델 설치 가능 여부 판단
- CUDA 가능 여부 판단
- ARM/x86 구분
- 자동 설치 전략 선택
- 위험 작업 차단

---

# 3. Persistent Task Engine

현재 설치 작업은 세션 기반이다.

하지만 실제 오케스트레이션은 작업 기반이어야 한다.

---

## 해결해야 할 문제

```text
SSH 끊김
UI 종료
앱 크래시
네트워크 단절
```

이 발생해도 작업 상태가 유지되어야 한다.

---

## 추천 구조

```rust
struct TaskRecord {
    task_id: String,
    state: TaskState,
    started_at: Timestamp,
    updated_at: Timestamp,
    host: String,
    progress: f32,
    logs: Vec<String>,
}
```

---

## 목적

- 장시간 작업 안정화
- 대형 모델 다운로드 복구
- 다중 서버 작업 지원
- 작업 Resume 기능

---

# 4. Unified Service Registry

현재는 Ollama 중심 구조다.

하지만 앞으로는 다양한 서비스가 추가된다.

---

## 확장 대상

```text
Docker
Nginx
Redis
PostgreSQL
CUDA Toolkit
systemd
Kubernetes
```

---

## 추천 Trait 구조

```rust
trait ServiceProvider {
    fn detect(&self);
    fn install(&self);
    fn uninstall(&self);
    fn status(&self);
    fn update(&self);
}
```

---

## 목적

- 서비스 구조 통합
- UI 공통화
- LLM Action 일반화
- 안전 계층 통합

---

# 5. Safety Layer 강화

Vega는 이제 위험한 작업을 수행할 수 있다.

따라서 반드시 안전 계층이 필요하다.

---

# A. Confirmation Barrier

위험 작업:

```text
shutdown
reboot
rm -rf
purge
```

에는 다음이 필요하다:

- 2단계 확인
- active session 체크
- 현재 사용자 검증
- cooldown 적용

---

# B. Dry-Run / Simulation Mode

실제 실행 없이 계획만 생성한다.

```text
Generate Plan Only
```

---

## 목적

- LLM 오판 방지
- 위험 작업 사전 검토
- 자동화 안전성 확보

---

# C. Policy Engine

예시:

```text
production 서버 shutdown 금지
GPU 서버 model delete 금지
특정 호스트 package remove 차단
```

---

## 목적

- 조직 정책 강제
- 실수 방지
- AI 행동 제한

---

# 6. Event Stream / Observability

이제부터는 단순 로그가 아니라 이벤트 기반 구조가 필요하다.

---

## 예시 이벤트

```text
MODEL_PULL_STARTED
MODEL_PULL_PROGRESS
MODEL_PULL_COMPLETED
MODEL_DELETE_BLOCKED
SERVICE_INSTALL_STARTED
SERVER_SHUTDOWN_INITIATED
```

---

## 목적

- Timeline UI
- Audit
- Replay
- Automation Trigger
- Debugging

---

# 7. Multi-host Orchestration

현재:

```text
Single Host
```

미래:

```text
Multiple Hosts
```

---

## 예시 구조

```text
Host A → Inference
Host B → Embedding
Host C → Vector Database
Host D → Monitoring
```

---

## 최종 방향

```text
Local AI Datacenter Manager
```

---

# 🏗️ 추천 개발 순서

# Phase 1 — 기반 안정화

1. Action Trait
2. Install State Machine
3. Persistent Task Engine
4. Host Snapshot
5. Event Bus

---

# Phase 2 — 안전성

6. Permission Tier
7. Dry-Run
8. Rollback
9. Policy Engine

---

# Phase 3 — 확장

10. Unified Service Registry
11. Docker Support
12. CUDA Installer
13. Multi-host Orchestration

---

# Phase 4 — AI Integration

14. LLM Planner
15. Natural Language Operations
16. Autonomous Remediation
17. Fleet Optimization

---

# 📌 핵심 결론

지금 시점에서 가장 중요한 것은:

```text
새 기능 추가
```

가 아니라:

```text
작업 실행 모델 표준화
```

이다.

이 단계가 안정화되면:

- 모든 서비스가 trait 기반으로 통합되고
- 안전 계층이 공통 적용되며
- Vega 전체가 orchestration runtime으로 진화할 수 있다.
