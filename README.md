# 🌌 Project VEGA: The Sovereign SRE Agent

> **"Engineered Intuition. Sovereign Execution."**

**Vega** is a next-generation AI-powered SRE agent designed for Linux environments. It translates natural language into precise, safe, and context-aware system commands. Unlike standard CLI tools, Vega possesses **"System Intuition"**—it understands your OS, learns from mistakes, and engages in dialogue when ambiguous.

## 🚀 Key Capabilities

*   **🧠 Context-Aware Intelligence**: Injected with knowledge of your specific OS (**Ubuntu 25.10**), Shell (**ZSH**), and Dev Stack (**Rust/Cargo**).
*   **🔄 Self-Correcting Execution**: Automatically detects command failures (e.g., casing errors, missing packages), analyzes `stderr`, and attempts **self-repair** without human intervention.
*   **💬 Interactive "Tikita-ka" Mode**: Doesn't just exit. If an instruction is vague, Vega asks clarifying questions (e.g., *"Install via pip or cargo?"*) and remembers your answers using persistent memory.
*   **🛡️ Built-in Safety**: A robust Risk Assessment Engine prevents accidental destruction. Critical commands require explicit confirmation.
*   **🏢 Enterprise Ready**: Includes **SSH Remote Management**, **PDF Session Reporting**, and **API Key Validation**.

## 🎥 Demo
[![Vega Demo](https://img.youtube.com/vi/BA4aD9KdQuE/0.jpg)](https://youtu.be/BA4aD9KdQuE)
*Click to watch the initialization & self-repair demo.*

## 🛠️ Installation

### Prerequisites
*   Rust (Cargo)
*   A Google Gemini API Key

### Build from Source
```bash
# Clone the repository
git clone https://github.com/dogsinatas/vega.git
cd vega

# Build the static binary
cargo build --release

# (Recommended) Install to your local bin
cp target/release/vega ~/.local/bin/
```

## 🎮 Usage

### 1. Interactive Shell (Recommended)
Simply run `vega` to enter the sovereign shell. It remembers your context.
```bash
$ vega
🌌 Vega Interactive Shell (Type 'exit' to quit)
❯ "Check my hard drive usage"
```

### 2. One-Shot Command
Execute a single natural language request.
```bash
$ vega "Update my system packages"
```

### 3. Key Validation
Audit your system for valid API keys.
```bash
$ vega check-key
```
*   **Features available**: Key Masking, Source Detection (`.zshrc` line search), Active Validation.

### 4. Remote Management
Execute AI-driven commands on remote servers via SSH.
```bash
$ vega --remote user@192.168.1.10 "Check docker container status"
```

### 5. Generate Report
Export your session history to a PDF report.
```bash
$ vega --report
```

## ⚙️ Configuration

Vega automatically scans your shell configuration (`.zshrc`, `.bashrc`) for keys.
Ensure `GEMINI_API_KEY` is exported in your environment.

```bash
export GEMINI_API_KEY="AIzaSy..."
```

## 🏗️ Architecture

*   **Core**: Rust (Safety & Performance)
*   **Brain**: Google Gemini 2.5 Flash (via `generativelanguage.googleapis.com`)
*   **Memory**: SQLite (`vega.db`) for context and history.
*   **UI**: Interactive Event Loop with intelligent formatting and spinners.

## 🔮 Future Roadmap
*   **Multi-Model Support**: Upcoming integration with **Anthropic Claude 3.5 Sonnet** (for deep reasoning) and **OpenAI GPT-4o**.
*   **Offline AI**: Native bindings for Ollama/Llama.cpp for air-gapped environments.
*   **Plugin System**: Extensible architecture for community capabilities.

---
*Built with ❤️ by Antigravity & Dogsinatas*
