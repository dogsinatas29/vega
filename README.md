# 🌌 Vega: The Sovereign SRE Agent

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)]()
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)]()

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
- **Persistent Metadata:** Dedicated SQLite storage for system-specific configurations and long-term state.
- **Dynamic State DB:** Acts as a persistent vault for discovered system metadata and reasoning history.

### 📜 SRE Operating Principles
1. **Error Budgets**: "No system is perfect. Automate as much as possible within acceptable failure margins."
2. **Toil Reduction**: "Manual repetition is a sin. All ops must be defined as code (IaC) and executed by VEGA."
3. **Blameless Postmortems**: "Failures are system issues, not human errors. VEGA logs heavily to protect your future self."

---

## 🧠 Core Architecture

Vega operates on an evolved **3-stage Reasoning Engine** that synthesizes system context with LLM intelligence to ensure SRE-grade safety.

1.  **Logical Scan & Context Synthesis**:
    *   **Intent Analysis**: Uses **SmartRouter** to determine if the task requires CoT reasoning via LLMs or a local fallback.
    *   **Context Gathering**: Gathers "Self-Awareness" metadata (OS, Kernel, Partitions) and scans shell environments (`.bashrc`, `.zshrc`) for aliases and environment variables.
2.  **Physical Mapping & Discovery**:
    *   **Resource Discovery**: Autonomously identifies system objects (IPs, project files like `lazy-lock.json`) through the **Discovery** module.
    *   **Mapping**: Bridges the gap between your logical intent and physical system resources (SSH hosts, disk partitions).
3.  **Privilege Enforcement & Execution**:
    *   **Safety Guardrails**: Validates commands against the **Safety Registry** and redacts sensitive data via the **Deidentifier**.
    *   **Orchestration & Learning**: Executes commands via the **Orchestrator**. Outcomes are stored in the **State DB** to enable **Local RAG** (Retrieval-Augmented Generation) for future learning.

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

> **AI Execution Flow:**
> 1.  **Reasoning (CoT)**: Peek into the AI's logical process before the final suggestion.
> 2.  **Proposal**: Presents a plan with a `Risk Level` (INFO/WARNING/CRITICAL).
> 3.  **Confirmation**: Explicit prompt to execute (`[y/N]`).

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

MIT License.
