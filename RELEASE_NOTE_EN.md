# 🌌 VEGA Release Note: v0.0.17.14 (Environment-Aware Cognition)

## 🚀 Major Updates

### 1. Environment-Aware Target Cognition
VEGA now understands the context of your infrastructure. 
- **Heuristic Grounding**: Abstract references like "ssh node", "remote server", or "원격 머신" are now intelligently mapped to your managed hosts in the Knowledge Base.
- **Context Injection**: AI prompts now include detailed inventory metadata (User, IP, Port, Status) for better decision making.
- **Anti-Contamination**: Improved resolution prevents accidental `localhost` fallback for remote-intended tasks.

### 2. Enhanced SSH RAW Observability
Transformation from "Sanitized Reporting" to "System Transparency".
- **Direct Stream Capture**: Injected low-level debug blocks to capture and display raw STDOUT, STDERR, and EXIT CODE before parsing.
- **Diagnostic Precision**: See exactly why a remote command failed (e.g., "model not found") with system-level detail.

### 3. Thin Action Execution (Snapshot Bypass)
Optimized performance for atomic control tasks.
- **Bypass Logic**: Simple actions (like Ollama list/pull/remove) now bypass the heavy `HostSnapshot` capture phase.
- **Latency Reduction**: 3-5x faster execution for high-frequency infrastructure management tasks.

### 4. Isolation & Safety Hardening
- **Explicit OLLAMA_HOST**: Forced `OLLAMA_HOST=127.0.0.1:11434` for all Ollama actions to ensure deterministic execution on the target node.
- **Command Dispatch Normalization**: Standardized the `ActionExecutor` flow across local and remote transports.

---
**"Brevity is the soul of SRE. Determinism is its backbone."** 🫡🌌🚀
