# VEGA Next Architecture Tasks

# Semantic Infrastructure Cognition Transition

현재 VEGA는:

- Typed Intent Parsing
- Remote SSH Execution
- Validation Pipeline
- Safety System
- Dependency-aware Execution
- Semantic Orchestration

까지 확보했다.

다음 단계는:

```text
Environment-aware Infrastructure Cognition
```

이다.

---

# Core Problem

현재 구조:

```text
Natural Language
 → Intent
 → Action
```

이 구조는 아직 서비스 명시형이다.

예:

```text
"ollama에 mistral 설치해줘"
```

하지만 목표는:

```text
"설치된 LLM 모델 정보를 알려줘"
```

같은 상위 Semantic Goal을 이해하는 것이다.

즉:

- Ollama
- vLLM
- llama.cpp
- HuggingFace Cache
- Docker Runtime

등을 자동 추론해야 한다.

---

# Required Architectural Transition

현재:

```text
Intent → Execution
```

목표:

```text
Goal
 → Capability Discovery
 → Planner
 → Action Graph
 → Deterministic Execution
```

---

# Phase 1 — Immediate Required Work

# 1. Semantic Goal Layer

추가 필요:

```rust
enum Goal {
    EnumerateLlmModels,
    InstallLlmModel,
    RemoveLlmModel,
    CheckLlmHealth,
    DeployInferenceStack,
}
```

LLM 역할:

- Goal 추론
- Semantic abstraction 해석

LLM은 command 생성 금지.

---

# 2. Capability Registry

```rust
enum Capability {
    Ollama,
    Vllm,
    LlamaCpp,
    Docker,
    CUDA,
    PythonVenv,
}
```

---

# 3. Capability Detector System

```rust
trait CapabilityDetector {
    async fn detect(host: &Host) -> Vec<Capability>;
}
```

---

# 4. Ollama Detector

검사 대상:

```bash
which ollama
systemctl status ollama
~/.ollama/models
curl localhost:11434/api/tags
```

---

# 5. vLLM Detector

검사 대상:

```bash
docker ps
ps aux | grep vllm
pip show vllm
```

---

# 6. llama.cpp Detector

검사 대상:

```bash
llama-server
llama.cpp binary
GGUF files
```

---

# 7. Host Snapshot System

필요 구조:

```rust
struct HostSnapshot {
    capabilities,
    services,
    models,
    gpu,
    memory,
    containers,
}
```

복합 추론은 결국:

```text
환경 상태 기반 reasoning
```

이다.

---

# 8. Planner Layer

현재:

```text
Intent == Execution
```

목표:

```text
Goal → Execution Strategy
```

예:

사용자:

```text
설치된 LLM 모델 알려줘
```

↓

Planner:

```text
Need:
- detect providers
- enumerate models
```

↓

Executor:

```text
Ollama detected
vLLM absent
llama.cpp absent
```

↓

Action Graph:

```text
ollama list
```

---

# 9. Deterministic Execution Boundary

절대 유지해야 하는 원칙:

```text
LLM → bash 생성
```

금지.

반드시:

```text
LLM → Goal
Goal → Typed Action Graph
Rust → Deterministic Execution
```

구조 유지.

---

# Phase 2 — Runtime Evolution

필요 기능:

- stdout streaming
- stderr streaming
- cancellation
- timeout
- retry
- rollback
- task lifecycle

---

# Task Lifecycle

```rust
enum TaskState {
    Pending,
    Running,
    Streaming,
    Validating,
    Completed,
    Failed,
    RolledBack,
}
```

---

# Phase 3 — Distributed Orchestration

장기 목표:

- dependency graph
- cross-provider reasoning
- distributed orchestration
- semantic infrastructure cognition

---

# Future Direction

VEGA는 단순 AI Shell Wrapper가 아니다.

목표는:

```text
Semantic Infrastructure Runtime
```

그리고 최종적으로:

```text
Safe Autonomous Infrastructure Orchestrator
```

를 구축하는 것이다.
