# 🗺️ VEGA Development Roadmap

This document tracks the remaining tasks and future development plans for VEGA.

## 🚀 Priority: SSH Connection Enhancements
Based on current implementation status, the following SSH improvements are prioritized:

- [ ] **SSH Key Management**: Support custom SSH key paths in `vega setup` and command execution.
- [ ] **Security Policy Control**: Allow users to toggle `StrictHostKeyChecking` for different environments.
- [ ] **Adaptive Timeouts**: Intelligently adjust SSH connection timeouts based on network latency.
- [ ] **Connection Multiplexing**: Implement `ControlMaster` support for faster repetitive commands.

## 🏢 Phase 5: Enterprise (Cloud & Persistence)
*Focus: Scaling VEGA for multiple environments and long-term reporting.*

- [ ] **rclone Cloud Sync**:
    - Automated project backup to Google Drive/S3 via `rclone`.
    - Cross-device state synchronization for `vega.db`.
- [ ] **Persistent Metadata Store**:
    - Dedicated `metadata` table in SQLite for long-term configuration.
    - At-rest encryption using local keyring-derived keys.
- [ ] **Advanced Reporting Engine**:
    - PDF/Markdown report export for SRE activities.
    - Automated email/Slack notifications via AI summaries.

## 🛡️ Phase 6: Autonomous Recovery (Planned)
*Focus: Hardening VEGA for extreme/offline environments.*

- [ ] **Offline Diagnostic Ruleset**: Enhanced regex/rule-based engine when LLM is unavailable.
- [ ] **GRUB Recovery Mode Guide**: Context-aware instructions for fixing boot issues.
- [ ] **Automatic Dependency Healing**: Proactive detection and fixing of missing system tools.

---
*Last Updated: 2026-02-18 by Antigravity*
