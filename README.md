# 🌌 Vega: The Sovereign SRE Agent

[![Vega Demo](https://img.shields.io/badge/YouTube-Shorts-red?style=for-the-badge&logo=youtube)](https://youtube.com/shorts/6a-fscWTTVo?si=B4wqRKqZbGJbtn4Z)

[한국어 문서 (Korean Documentation)](README.ko.md) | [Development Roadmap](ROADMAP.md)

> **🚧 Current Status**: Testing system configuration tasks via SSH access to OS running on QEMU.

> **"The Pocket Knife Strategy"**
>
> A non-resident, lightweight system administration agent that respects your shell environment. Refuses to be a daemon.

---

### 🛡️ Core Logic Update: "Discovery First"

> **"Questions are a last resort."**

VEGA leverages every available tool (DHCP, QEMU Guest Agent, ARP tables, etc.) to gather information autonomously. Discovered context is immediately persisted in the State DB for future operations.

-   **Silent Discovery:** Background scanning triggered upon incomplete information (e.g., missing IP).
- **Resolve & Persist:** Found system info is cached to prevent redundant queries.
- **Cloud Sync Integration:** Non-disruptive project backup and state synchronization powered by `rclone`.
- **Hybrid Execution Pipeline (v0.0.14.9):** A decoupled engine with advanced context-awareness and safety interceptors.
- **Decision Lineage:** Every reasoning step (why a command was proposed) is permanently recorded in the State DB.
- **Persistent Metadata:** Dedicated SQLite storage for system-specific configurations and long-term state.
- **Full-Stack SRE Diagnostic:** Automated technical analysis with OS/HW/Network visibility and GB-normalized metrics.
- **Dynamic Localization:** System locale-aware prompt and report generation (Full Korean Support).
- **Intelligent Port Heuristics:** Automated safety guard to prevent AI port confusion (forcing SSH to port 22 over AI ports).
- **Llama 3.1 Hardening:** Specialized persona injection and instruction compliance for 8B-class local models.
- **Autonomous Remote SRE Control (v0.1.6):** Secure internal SSH engine (`ssh2`) with encrypted credential persistence and automatic `sudo` password injection.
- **One-Shot Remote Diagnostic:** 10x faster system metric collection via single-session multi-command payloads.
- **Infrastructure Cognition (v0.0.17.15):** Deep physical resource sensing (CPU/RAM/Disk) and proactive validation based on dynamic tool requirements.

### 📜 SRE Operating Principles
1. **Error Budgets**: "No system is perfect. Automate as much as possible within acceptable failure margins."
2. **Toil Reduction**: "Manual repetition is a sin. All ops must be defined as code (IaC) and executed by VEGA."
3. **Blameless Postmortems**: "Failures are system issues, not human errors. VEGA logs heavily to protect your future self."

---

## 🧠 Core Architecture (Hybrid Pipeline v0.0.17.15)

Vega operates on a **Decoupled Execution Pipeline** that ensures absolute deterministic control with AI-assisted optimizations.

1.  **Intent Resolution**: Decodes natural language into structured operations (Backup, Install, etc.). Fallbacks to AI for complex inputs.
2.  **Infrastructure Sensing & Validation (v0.0.14)**: Captures deep host snapshots (CPU/RAM/Disk) or performs **Thin Action Execution** (snapshot bypass) for simple control tasks.
3.  **Environment-Aware Cognition**: Semantically grounds abstract references (e.g., "ssh node", "remote server") to the managed inventory using **Heuristic Target Resolution**.
4.  **Template Building**: Constructs a deterministic **Command AST** (Skeleton) to prevent AI-induced syntax errors.
5.  **AI Option Generation**: AI provides optimal flags (e.g., `--checksum`, `--progress`) injected into the skeleton.
6.  **VEE (Virtual Execution Engine)**: Performs **State-based Simulation**. Checks path existence and predicts system impact.
7.  **Risk Evaluation**: Assigns a risk score (0-100). Critical ops require explicit manual authorization.
8.  **Execution & RAW Observability**: Dispatches commands to local/remote (SSH) environments with **Direct Stream Capture** (STDOUT/STDERR/EXIT_CODE).
9.  **Desired State Reconciliation**: Semantically evaluates outcomes to ensure the goal is satisfied even if raw commands return non-zero codes (e.g., "already absent" is success).
10. **Reporting & Lineage**: Persists the entire trace (Lineage) and generates **AI-Powered SRE 5-Step Reports**.

---

## 📦 Build Prerequisites

Before building from source, install the required development packages:

```bash
# Fedora / RHEL / CentOS
sudo dnf install -y openssl-devel pkg-config sqlite-devel

# Ubuntu / Debian
sudo apt install -y libssl-dev pkg-config libsqlite3-dev sqlite3

# Arch Linux
sudo pacman -S openssl pkg-config sqlite
```

**Common Build Dependencies (Rust Crates):**
- `colored` (Terminal colors)
- `reqwest` (HTTP Client for AI)
- `ssh2` (SSH Protocol - requires openssl headers)
- `serde` (JSON Parsing)
- `rusqlite` (SQLite storage)

### 🔧 Troubleshooting Build Issues

If you still get `openssl-sys` errors after installing the packages, try these steps:

**1. Verify OpenSSL installation:**
```bash
# Check if openssl.pc exists
pkg-config --modversion openssl

# If the above fails, find openssl.pc manually
find /usr -name "openssl.pc" 2>/dev/null
```

**2. Set PKG_CONFIG_PATH manually:**
```bash
# Common locations (adjust based on your system)
# Fedora/RHEL
export PKG_CONFIG_PATH=/usr/lib64/pkgconfig:$PKG_CONFIG_PATH

# Ubuntu/Debian
export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH

# Then retry build
cargo build --release
```

**3. Alternative: Use vendored OpenSSL:**
```bash
# This will compile OpenSSL from source (slower but more reliable)
cargo build --release --features vendored-openssl
```

> **Note:** If using vendored OpenSSL, you'll also need `perl` and `make` installed.

---

## ⚡ Installation

Vega is built as a single static binary. No runtime dependencies required.

```bash
# 1. Clone the repository
git clone https://github.com/dogsinatas29/vega
cd vega

# 2. Build Release Binary
cargo build --release

# 3. Create local bin directory
mkdir -p ~/.local/bin

# 4. Install to local bin
cp target/release/vega ~/.local/bin/

# 5. Add to PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# 6. Verify installation
vega --help

# 7. Configure API Keys & Preferences
# This will launch an interactive wizard to set your Gemini/OpenAI/Claude keys.
vega setup
```

---

## 🛠️ Usage

### 1. Setup
Launch the interactive wizard to configure API keys and preferences.

```bash
vega setup
```

### 2. Google Login & Managed Billing
Authenticate with your Google account to leverage project-based quotas.

```bash
vega login
```

-   **Higher Quotas**: Managed accounts enjoy significantly higher RPM than free keys.
-   **Auto-Renewal**: Uses OAuth2 Refresh Tokens for background session management.
-   **Fallback Logic**: Automatically cascades from OAuth tokens to API Keys or Web Sessions.

### 3. History & Memory Management
Browse past commands and re-run complex operations via the `fzf`-powered interface.

```bash
vega history
```

### 4. Natural Language Commands
```bash
# English
vega "Find all files larger than 1GB in /home"
```

> **AI Execution Flow (Pipeline v0.0.10):**
> 1.  **Intent**: Resolves structured "What" (e.g., `backup`).
> 2.  **Simulation**: VEE checks path existence locally.
> 3.  **Data De-identification**: Sensitive info like IPs and Keys is masked before API transmission.
> 4.  **Hardened SSH**: `-o BatchMode=yes` is forced to prevent hangs and zombie sessions.
> 5.  **Proposal**: AI suggests optimized `options` (flags).
> 6.  **Audit**: Logs the decision lineage before execution.

---

## 📊 SRE Report Example (Technical Detail)

VEGA generates high-density technical reports based on the **Decision Lineage** recorded during sessions. Here is an example of a generated session brief:

```markdown
# 🌌 VEGA Maintenance Session Report
**Session ID:** `SID-1042` | **Date:** 2026-03-15 | **Risk Level:** 🟡 MEDIUM

---

## 🧠 Decision Lineage (Execution Trace)

### 1. Request: "backup current dir to serverA"
- **[Intent]** `HybridResolver`: Identified `Tool: rclone`, `Op: sync`, `Target: REMOTE_01`
- **[Sim]** `VEE`: Local path `/home/user/project` exists. Path size: 450MB. (Safe)
- **[Risk]** `Evaluator`: Score 20 (Info). Automatic Approval.
- **[Final Command]** `rclone sync ./ serverA:backup/vega_sync --progress --checksum --fast-list`
- **[Result]** ✅ SUCCESS (Lineage Stored)

### 2. Request: "rm -rf /var/log"
- **[Intent]** `LocalResolver`: Identified `Tool: coreutils`, `Op: delete`, `Target: /var/log`
- **[Sim]** `VEE`: **CRITICAL**. Recursive deletion of system log directory detected.
- **[Risk]** `Evaluator`: **Score 100 (CRITICAL)**. 
- **[Status]** 🛑 **DENIED** (Safety Block)

---

## 📈 Impact Analysis (ASCII-Viz)
Risk Distribution across session:
```text
[CRITICAL]  | ██████████ (33%) -> BLOCKED
[WARNING]   | (0%)
[INFO]      | ████████████████████ (67%) -> EXECUTED

RISK SCORE HEATMAP:
0 [##########          ] 100
AVG: 40.0 (MEDIUM)
```

## 💡 SRE Insights
- **Safety Guard Effectiveness:** 100% (All high-risk operations were intercepted).
- **Automation Accuracy:** AI correctly resolved complex NL intents.
- **Toil Reduction:** Prevented 1 potential system-wide data loss.

---

## 📧 Supervisor Email Briefing (Template)

VEGA can format its lineage data into an executive summary ready to be sent to your team lead or SRE manager via `vega --report --email`.

```text
Subject: [SRE Briefing] Maintenance Session Summary - 2026-03-15 (SID-1042)

Hi Team,

I've completed the scheduled maintenance on the master node using VEGA SRE Agent. 
Summary of the session below:

1. System Status: Post-backup sync completed to primary cloud (serverA).
2. Risk Management: Intercepted 1 CRITICAL operation (unauthorized system log deletion).
3. Technical Trace:
   - Success Rate: 100% (2/2 attempted authorized tasks)
   - Safety Interventions: 1 blocked high-risk command.
   - Decision Lineage: Verified and stored in State DB.

Detailed technical report attached / archived in session lineage SID-1042.

Best regards,
[Your Name]
```
```

---

## ☁️ Cloud Integration (rclone)

VEGA uses `rclone` for seamless cloud project backup and state synchronization.

### 1. Pre-requisites
- Install `rclone` on your system: `sudo dnf install rclone` (Fedora) or `sudo apt install rclone` (Ubuntu).
- Configure your remotes: `rclone config`.

### 2. Autonomous Discovery
VEGA's discovery engine automatically identifies active `rclone` remotes and masks sensitive names (e.g., `gdrive:` becomes `REMOTE_01`) when communicating with the AI.

### 3. Primary Remote Setup
You can "pin" a specific remote as your default sync target. VEGA will remember this address and prioritize it for project-wide synchronization.
- Run `vega setup`.
- Select your preferred cloud remote in the **[2] Cloud Integration** step.
- The choice is saved in `config.toml` as `primary_remote`.

### 3. Natural Language Cloud Ops
You can use natural language to interact with your cloud storage. VEGA will automatically resolve the masked names back to your real remotes before execution.
```bash
# Example: Copy a folder from Google Drive
vega "Copy the 'input' folder from my Google Drive to here"

# Example: Sync the current project to cloud
vega sync
```

### 4. Safety Guardrails
- **Size Limit**: Sync operations are automatically blocked if the transfer size exceeds **1GB** to prevent accidental data costs or overhead.
- **Confirmation**: All cloud operations require explicit user confirmation.

---

## 🛰️ Fleet Management (v0.0.12)

VEGA now features a deterministic management system for your entire server fleet.

### 1. Registration
Registered nodes are stored in the local **Knowledge Base (KB)** and prioritized over temporary discovery results.
- **Auto-Registration**: During `vega setup`, select discovered hosts to add to your fleet.
- **Manual Addition**: `vega add-node 192.168.0.150` to explicitly manage a new server.

### 2. SSH Self-Healing (`sync-ssh`)
Automatically bridge the gap between your management agent and your shell.
- Run `vega sync-ssh` to generate/update entries in your `~/.ssh/config` based on your registered nodes.
- This enables you to use `ssh <name>` directly from any terminal window.

### 3. Real-time Monitoring
The `vega status` dashboard performs real-time TCP probes to verify server availability instantly, ensuring your "ONLINE" status is always accurate.

---

## 🔑 SSH Key Setup (Recommended)

VEGA uses `BatchMode=yes` for all remote operations to ensure deterministic, non-interactive execution. This means **Public Key Authentication** must be configured for all managed nodes (including `localhost`).

### 1. Check your Public Key
First, verify if you have an existing SSH key:
```bash
# Common paths for ed25519 or rsa keys
cat ~/.ssh/id_ed25519.pub || cat ~/.ssh/id_rsa.pub
```
*If you don't have one, generate it via `ssh-keygen -t ed25519`.*

### 2. Copy Key to Target Node
Use `ssh-copy-id` to authorize your key on the remote server. This eliminates the need for password prompts during VEGA operations.
```bash
# Syntax: ssh-copy-id <USER>@<HOST>
ssh-copy-id dogsinatas@192.168.0.150
```

### 3. Verify Passwordless Login
Ensure you can log in without being prompted for a password:
```bash
ssh dogsinatas@192.168.0.150
```
*Once successful, VEGA will be able to manage this node autonomously.*

### 🛠️ Troubleshooting `localhost` Access
If `ssh localhost` fails with `Permission denied`, ensure your local environment is configured for BatchMode:
1. **Setup authorized_keys**: `cat ~/.ssh/id_ed25519.pub >> ~/.ssh/authorized_keys`
2. **Start ssh-agent**: `eval $(ssh-agent -s) && ssh-add ~/.ssh/id_ed25519`
3. **Check SSH Server**: Ensure `sshd` is running locally if you want VEGA to manage the current host via SSH.

---

## 📋 Command Reference (SRE Playbook)

| Category | Command | Description |
| :--- | :--- | :--- |
| **Fleet** | `vega status` | Show real-time dashboard (Port, Load, Tags) |
| | `vega status <target>` | **Deep Scan**: Real-time OS, Kernel, Disk metrics via SSH |
| | `vega add-node <ip>` | Manually register a new SSH node to KB |
| | `vega sync-ssh` | Sync KB targets to `~/.ssh/config` (Port-aware) |
| | `vega update --fleet` | Fleet-wide maintenance (Kernel, Load sync) |
| **Intelligence** | `vega report --sre` | **5-Step Report**: AI analysis of session lineage |
| | `vega history` | Interactive history UI via `fzf` |
| | `vega health` | Analyze system logs and suggest automated fixes |
| **Project** | `vega refresh` | Global refresh (Discover nodes & Snapshot shell) |
| | `vega sync` | `rclone`-based project & state synchronization |
| | `vega backup <src> <dst>` | Smart backup with validation & risk check |
| **Setup** | `vega setup` | Interactive configuration wizard |
| | `vega login` | Authenticate via Google OAuth2 |
| | `vega config` | Sync shell environment snapshot |

---

## 🛡️ Safety Features

*   **Explicit Confirmation**: Critical commands (`rm`, `dd`) require typing "YES".
*   **Data Redaction**: Sensitive data (IPs, Keys) is redacted before sending to AI.
*   **Non-interactive SSH**: Mandatory `-o BatchMode=yes` to prevent hang-ups and ensure deterministic failures.
*   **Local Processing**: Simple commands match locally without API calls.

---

## 📂 Project Structure & File Roles

Below is an overview of the core components in the `src` directory:

### 🛠️ Core Infrastructure
*   [`main.rs`](src/main.rs): The application entry point. Handles CLI arguments and top-level command routing.
*   [`context.rs`](src/context.rs): The heart of VEGA's "Self-Awareness". Manages OS, hardware, and network metadata.
*   [`init.rs`](src/init.rs): Orchestrates the bootstrap process, ensuring DBs and configs are ready.
*   [`config.rs`](src/config.rs): Handles the loading and validation of `vega.toml`.

### 🧠 AI & Reasoning (`src/ai`)
*   [`router.rs`](src/ai/router.rs): The logic that decides which AI engine to use based on the complexity of the query.
*   [`providers/`](src/ai/providers/): Specialized connectors for Gemini (Flash/Pro), Claude, and local regex-based engines.
*   [`prompts.rs`](src/ai/prompts.rs): Manages system personas and context injection for LLM prompts.

### 🚀 Execution Layer (`src/executor`)
*   [`orchestrator.rs`](src/executor/orchestrator.rs): Manages the lifecycle of task execution, including multi-step recovery.
*   [`pkg.rs`](src/executor/pkg.rs): Abstracted package manager (apt, dnf, pacman) for cross-distro compatibility.
*   [`healer.rs`](src/executor/healer.rs): Logic for analyzing failures and suggesting automated fixes.

### 🔍 System Intelligence (`src/system`)
*   [`discovery.rs`](src/system/discovery.rs): Autonomous scanning for project-specific metadata (e.g., Node/Rust projects).
*   [`archivist.rs`](src/system/archivist.rs): Manages long-term storage of reasoning history and system snapshots.
*   [`env_scanner.rs`](src/system/env_scanner.rs): Deep-dives into `.bashrc` and `.zshrc` to understand your custom environment.

### 🛡️ Safety & Security
*   `src/safety/`: Contains the **Safety Registry** which validates commands against a list of dangerous patterns.
*   `src/security/`: Handlers for sensitive information redaction and `keyring` management.

### 💾 Storage & Knowledge
*   `src/storage/`: Direct interactions with the SQLite backend.
*   [`knowledge.rs`](src/knowledge.rs): Management of the local RAG system and FTS5 search index.

---

## 📊 VEGA SRE Reports

Vega provides two types of AI-powered SRE reports to ensure complete system visibility and reasoning transparency.

### 1. System Diagnostic Report (Active)
Provides a real-time "Pulse" of the target system (Local or Remote). 
*   **Trigger**: `vega status <target>` or via natural language ("Show system info for 192.168.0.150").

```markdown
# 🚀 VEGA SRE System Report
**Hardware Pulse**
- **OS**: Ubuntu 25.10
- **CPU**: Intel(R) Core(TM) i7-4790 CPU @ 3.60GHz
- **RAM**: 1.41 GB / 15.07 GB (Used/Total)
- **Uptime**: up 6 hours, 35 minutes

**Network Map (Listening Ports)**
- **22 (ssh)**: Open (Standard Management)
- **11434 (ollama)**: Active (AI/API Service)
```

### 2. Decision & Analysis Report (Planned / Experimental)
Focuses on "Why" and "How" by analyzing session lineage and providing deep SRE 5-step analysis.

```text
# 🌌 VEGA Maintenance Session Report
**SID-1042** | **Risk Level: 🟡 MEDIUM**

## 🧠 Decision Lineage (Execution Trace)
1. Request: "backup current dir to serverA"
- [Intent] Identified Tool: rclone, Op: sync
- [Risk] Score 20 (Info). Automatic Approval.
- [Final Command] rclone sync ./ serverA:backup/vega_sync

## 📄 SRE 5-Step Analysis
1. Issue: Port 11434 is in use by Ollama.
2. Cause: Active API service exposed.
3. Solution: Review security config or restrict access.
```

---

## 📄 License

GPL-3.0 License.
