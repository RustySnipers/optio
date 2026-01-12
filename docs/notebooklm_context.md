# Optio Strategic Context (NotebookLM Export)

> Extracted strategic goals and capabilities from the Optio design notebook.
> Source: NotebookLM (requires authentication for live access)

## 1. Mission Statement

Optio is the **Enterprise IT Master Tool** - a security-focused, compliance-driven
orchestration platform for IT consultants operating in hostile network environments.

**Core Principles:**
- **Zero Trust:** Assume the network is hostile
- **Air-Gap Friendly:** Operate in disconnected or restricted environments
- **Compliance First:** NIST CSF 2.0 and SOC 2 Type II alignment
- **Passive Agent Model:** Agents never open inbound ports

## 2. The Dual-Stack Architecture

### The Hub (Command Center)
- **Stack:** Tauri v2 (Rust Backend) + React/TypeScript (Frontend)
- **Role:** Central orchestration, script generation, compliance tracking
- **Security:**
  - Ed25519 command signing
  - Encrypted SQLite vault
  - Heartbeat listener for Agent connections

### The Agent (Endpoint Sentinel)
- **Stack:** Pure Rust (Headless) or PowerShell Script
- **Role:** Passive telemetry collection, command execution
- **Communication:** Outbound-only polling (never opens inbound ports)
- **Security:**
  - Hardware-derived UUID identification
  - Signature verification before execution
  - TLS-secured callback connection

## 3. Core Capabilities

### Factory Module
Dynamic script generation engine that manufactures unique, state-aware
PowerShell scripts for each engagement with:
- Identity injection (Client IP, Auth Token, Callback Port)
- Idempotent operations
- Cryptographic signature embedding

### GRC Center
Compliance assessment and evidence collection aligned to:
- **NIST CSF 2.0:** Full control mapping
- **SOC 2 Type II:** CC6.1, CC6.6, CC6.8 focus
- **GDPR:** Data protection assessments

### Network Intelligence
- Native TCP scanner (no Nmap dependency for air-gap)
- Asset inventory management
- Vulnerability correlation

### Hub Listener
Server-side infrastructure for Agent communication:
- TCP listener on configurable port (default: 8443)
- Token-based Agent authentication
- Command queue for signed payload dispatch
- Session tracking and telemetry storage

## 4. Production Readiness Criteria

### Security (Must Have)
- [x] No `.unwrap()` in production code (panic-free)
- [x] Ed25519 command signing implemented
- [x] Hub listener for Agent heartbeats
- [ ] Command signature verification in Agent scripts
- [ ] Key rotation mechanism
- [ ] Audit logging for all operations

### Compliance (Must Have)
- [x] NIST CSF control framework
- [x] SOC 2 Type II controls (CC6.x)
- [ ] Evidence collection workflow
- [ ] Compliance report generation

### Operations (Should Have)
- [ ] Hub keypair persistence to database
- [ ] Agent session persistence
- [ ] Rate limiting on listener
- [ ] TLS support for listener

## 5. Coding Standards

### Rust (Backend)
- **Safety First:** No `.unwrap()` - use `Result` propagation
- **Async:** Tokio runtime for all I/O
- **Serialization:** Serde with strict typing
- **Error Handling:** Typed errors with meaningful codes

### React/TypeScript (Frontend)
- Functional components with hooks
- Strict prop typing
- Tauri IPC via typed wrappers

## 6. Version History

| Version | Milestone |
|---------|-----------|
| 0.1.0   | Initial prototype with Factory and GRC |
| 0.2.0   | Network Intelligence module |
| 0.3.0   | **Production Hardening** - Crypto + Listener |
| 1.0.0   | Production Release (target) |
