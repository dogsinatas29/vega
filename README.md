# Vega: AI-Native Terminal Admin Engine

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)]()
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)]()

> **"The Pocket Knife Strategy"**
> 
> A non-resident, lightweight system administration agent that respects your shell environment.
> Built with Rust for safety, standard-compliance (XDG), and zero-cost abstraction.

## ⚡ Core Philosophy | 핵심 철학

1.  **Internalism (내부 관찰자)**:
    -   Does not run as a daemon. 
    -   Scans `/proc`, `/sys`, and `~/.config` only when invoked.
    -   Respects your shell aliases and history seamlessly.

2.  **Low Sodium (저염식 AI)**:
    -   **Zero-Token Path**: Prioritizes local matching (Regex, Fuzzy Search) before calling expensive LLMs.
    -   **Smart fzf**: Auto-triggers local search for known patterns.

3.  **Security First**:
    -   All logs and configs are stored with **600 (User-Only)** permissions.
    -   API Keys are masked during setup.
    -   Dangerous commands (`rm`, `dd`) require explicit override.

## 🛠️ Features

### [A] The Brain (Rust Core)
-   **Config Loader**: `toml` based configuration adhering to XDG standards.
-   **Context Collector**: Rapidly scans kernel state (`loadavg`, `meminfo`, `lsblk`).
-   **Shell Snapshot**: Captures `zsh` aliases and `zoxide` paths for seamless integration.
-   **Regex Scanner**: Robustly parses `export KEY="VALUE"` from shell configs (`.zshrc`, `.bashrc`) to auto-configure API keys.

### [B] The Interface
-   **Vega CLI**:
    -   `vega "command"`: Intelligent execution (Local -> fzf -> LLM).
    -   `vega setup`: Interactive wizard with **Regex-based API Key Discovery**.
    -   `vega setup --yes`: **Silent Mode** for automated deployments.
    -   `vega refresh-config`: Quickly syncs shell environment snapshot.
    -   `vega monitor`: **DooM-style 3D System Visualization**.

### [C] The Eye (DooM Monitor)
-   **Metaphor**: System Load = Wall Height.
-   **Tech**: Pure Rust ASCII Raycaster using `crossterm`.
-   **Performance**: runs at ~30 FPS on standard TTY.

## 🚀 Installation & Usage

### 1. Build
```bash
cargo build --release
cp target/release/vega ~/.local/bin/
```

### 2. Setup
```bash
vega setup
```

### 3. Usage examples
```bash
# Auto-match local alias or history
vega ssh

# Ask AI (Fallback)
vega "how to resize swap partition on lvm?"

# Visualize System
vega monitor
```

## 📂 Project Structure
```
src/
├── main.rs         # Entry point & Hybrid Routing
├── config.rs       # XDG Config & Validation
├── context.rs      # /proc & System Scanning
├── shell.rs        # zsh alias & zoxide snapshot
├── token_saver.rs  # Local Pattern Matching & History
└── doom/           # 3D Visualization Engine
    ├── engine.rs   # Raycasting Loop
    └── renderer.rs # ASCII Buffer
```

## 📄 License
MIT License.
