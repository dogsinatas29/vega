# 🌌 Vega: The Sovereign SRE Agent

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)]()
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)]()

[한국어 문서 (Korean Documentation)](README_KR.md)

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

---

## 🧠 Core Architecture

Vega operates on a 3-stage **Reasoning Engine** to ensure safety and accuracy.

![Vega Logic Flow](assets/logic_flow.png)

1.  **Logical Scan**: Understand user intent and identify target objects (files, processes, paths).
2.  **Physical Mapping**: Map targets to physical resources (Partitions, SSH hosts) and verify existence.
3.  **Privilege Enforcement**: Apply 'Least Privilege' principles and generate the safest possible command.

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

## 📄 License

MIT License.
