# 🌌 Vega: The Sovereign SRE Agent

[한국어 문서 (Korean Documentation)](README_KR.md) | [Development Roadmap](ROADMAP.md)

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
- **Hybrid Execution Pipeline (v0.0.10):** A decoupled 7-stage engine (Intent -> AST -> AI Options -> Simulation -> Execution) for maximum safety.
- **Decision Lineage:** Every reasoning step (why a command was proposed) is permanently recorded in the State DB.
- **Persistent Metadata:** Dedicated SQLite storage for system-specific configurations and long-term state.

### 📜 SRE Operating Principles
1. **Error Budgets**: "No system is perfect. Automate as much as possible within acceptable failure margins."
2. **Toil Reduction**: "Manual repetition is a sin. All ops must be defined as code (IaC) and executed by VEGA."
3. **Blameless Postmortems**: "Failures are system issues, not human errors. VEGA logs heavily to protect your future self."

---

## 🧠 Core Architecture (Hybrid Pipeline v0.0.10)

Vega operates on a **Decoupled Execution Pipeline** that ensures absolute deterministic control with AI-assisted optimizations.

1.  **Intent Resolution**: Decodes natural language into structured operations (Backup, Install, etc.). Fallbacks to AI for complex inputs.
2.  **Template Building**: Constructs a deterministic **Command AST** (Skeleton) to prevent AI-induced syntax errors.
3.  **AI Option Generation**: AI provides optimal flags (e.g., `--checksum`, `--progress`) injected into the skeleton.
4.  **VEE (Virtual Execution Engine)**: Performs **State-based Simulation**. Checks path existence and predicts system impact.
5.  **Risk Evaluation**: Assigns a risk score (0-100). Critical ops require explicit manual authorization.
6.  **Execution Provider**: Dispatches commands to local or remote (SSH) environments.
7.  **Reporting & Lineage**: Persists the entire trace (Lineage) and generates technical SRE reports.

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
# 1. Build Release Binary
cargo build --release

# 2. Create local bin directory
mkdir -p ~/.local/bin

# 3. Install to local bin
cp target/release/vega ~/.local/bin/

# 4. Add to PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# 5. Verify installation
vega --help
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
> 3.  **Proposal**: AI suggests optimized `options` (flags).
> 4.  **Audit**: Logs the decision lineage before execution.

---

## 📊 SRE Report Example

VEGA generates high-density technical reports based on the **Decision Lineage** recorded during sessions.

```markdown
# 🌌 VEGA Maintenance Report
**Session ID:** SID-1042 | **Risk Level:** 🟡 MEDIUM

### 🧠 Decision Lineage
- **Request:** "backup current dir to serverA"
- **Intent:** `Tool: rclone`, `Op: sync`, `Target: serverA`
- **VEE Simulation:** Source path exists. (Safe)
- **AI Proposed Options:** `["--progress", "--checksum", "--fast-list"]`
- **Result:** ✅ SUCCESS (Lineage Stored)
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

## 📋 Internal Commands

Vega provides several built-in commands for direct control.

| Command | Description |
| :--- | :--- |
| `setup` | Run the configuration wizard |
| `login` | Authenticate via Google OAuth2 |
| `history` | Interactive history UI via fzf |
| `install <pkg>` | Install packages (detects apt/dnf/pacman) |
| `connect <host>` | SSH connection with context memory |
| `status` | Show system status dashboard |
| `health` | Analyze system logs and suggest fixes |
| `backup <src> <dst>` | Smart backup with validation |
| `refresh <target>` | Refresh SSH host context |
| `update --all` | Update system packages |
| `sync` | rclone-based cloud project & state synchronization |
| `config` | Sync shell environment snapshot |

---

## 🛡️ Safety Features

*   **Explicit Confirmation**: Critical commands (`rm`, `dd`) require typing "YES".
*   **Data Redaction**: Sensitive data (IPs, Keys) is redacted before sending to AI.
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

## 📄 License

GPL-3.0 License.
