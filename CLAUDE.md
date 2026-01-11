# CLAUDE.md - Optio Memory Bank & Context

## 1. Project Overview & Identity
- **Name:** Optio Enterprise IT Management Platform
- **Core Function:** IT Orchestration Hub & "Passive" Agent System.
- **Mission:** Provide a secure, air-gap friendly, compliance-driven (NIST CSF/SOC 2) master tool for IT consultants.
- **Current Phase:** Production Hardening & v1.0 Release.

## 2. Architecture & Constraints
### The Hub (Control Plane)
- **Stack:** Tauri v2 (Rust Backend) + React/TypeScript (Frontend).
- **Storage:** Local SQLite (encrypted/vaulted).
- **Network:** Acts as the listener for Agent heartbeats or generates static payloads.
- **UI:** Tailwind CSS + Shadcn/UI patterns.

### The Agent (Endpoint Sentinel)
- **Stack:** Pure Rust (Headless Service).
- **Communication Protocol:** **Outbound-Only Polling.**
    - The Agent *never* opens an inbound port.
    - It "phones home" (Heartbeat) to fetch signed commands.
- **Execution:** Wraps PowerShell safely (supports Constrained Language Mode).
- **Security:** All commands must be cryptographically signed by the Hub.

### Compliance & Security
- **Standards:** NIST CSF 2.0, SOC 2 Type II (specifically CC6.1, CC6.6, CC6.8).
- **Zero Trust:** Assume the network is hostile.
- **Identity:** Agents use hardware-derived UUIDs; Hub uses generated Certificates.

## 3. Development Commands
| Action | Command | Context |
| :--- | :--- | :--- |
| **Dev Server** | `npm run tauri:dev` | Starts Frontend + Rust Backend |
| **Build Prod** | `npm run tauri:build` | Generates optimized binaries |
| **Lint Frontend**| `npm run lint` | ESLint + Prettier checks |
| **Lint Backend** | `cd src-tauri && cargo clippy` | Rust lints (Pedantic) |
| **Test Backend** | `cd src-tauri && cargo test` | Runs Rust unit/integration tests |
| **Format** | `npm run format` | Prettier formatting |

## 4. Coding Standards & Guidelines
### Rust (Backend/Agent)
- **Safety First:** **NEVER** use `.unwrap()`. Use `match`, `if let`, or `?` with `anyhow`/`thiserror`.
- **Async:** Use `tokio` runtime for all I/O and task scheduling.
- **Serialization:** Use `serde` with `serde_json` for all IPC and Heartbeats.
- **Error Handling:** Errors must be typed and propagated to the UI with meaningful codes.

### React/TypeScript (Frontend)
- **State:** Use React Hooks. Avoid complex global state unless necessary (Context API preferred).
- **Components:** Functional components only. Strict typing on `props`.
- **Communication:** Use `tauri/api` for invoking Rust commands (defined in `src-tauri/src/commands`).

## 5. Directory Structure Map
- `src-tauri/src/network`: Heartbeat logic, HTTP clients, and listeners.
- `src-tauri/src/factory`: Script generation engine (PowerShell templating).
- `src-tauri/src/grc`: Compliance mapping logic (NIST/SOC2).
- `src-tauri/src/commands`: Tauri command handlers (The API layer).
- `frontend/src/components`: UI logic (Dashboard, GRC Center).

## 6. Critical Implementation Details (Do Not Forget)
- **Heartbeats:** Must be compressed JSON.
- **Factory:** Generated scripts must be **idempotent** (can run multiple times without side effects).
- **Identity Injection:** The Factory injects the Consultant's IP/Cert into the Agent at build time.
