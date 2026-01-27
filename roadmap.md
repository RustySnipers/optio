# Optio Phase 2: Post-Prototype Strategic Plan

> **Date:** 2026-01-27
> **Status:** Draft
> **Target:** Production Release 1.0

## 1. Executive Summary

The prototype confirms our core hypothesis: **A Rust-based, mTLS-secured agent topology is viable and superior to legacy RMMs in speed and resource footprint.** The nervous system (gRPC/mTLS), memory (SQLite), and limbs (Agent execution) are functional.

However, to move from "Prototype" to "Product," we must bridge three critical gaps: **Visual Remote Control**, **Native Patch Management**, and **Telemetry Scalability**.

This roadmap outlines the technical strategy to close these gaps and refine Optio into a production-grade Enterprise RMM.

---

## 2. Technical Gap Analysis

### A. The "Visual" Gap (Remote Control)
* **Current Status:** We can scan ports and run shell commands.
* **Deficiency:** RMMs are defined by the ability to "see" the screen. Without this, we are just a fancy SSH wrapper.
* **Recommendation:** Implement a native Remote Desktop implementation over our existing gRPC tunnel. We should *not* rely on external tools like TeamViewer or AnyDesk.

### B. The "Automation" Gap (Patching)
* **Current Status:** We can run scripts.
* **Deficiency:** Relying on PowerShell scripts for Windows Updates is brittle.
* **Recommendation:** Implement native Rust bindings to the Windows Update Agent (WUA) COM interface using the `windows` crate. This allows granular control (scan, list, approve, install) with proper return codes, rather than parsing text output from `Get-WindowsUpdate`.

### C. The "Data" Gap (Telemetry Scalability)
* **Current Status:** We log raw CPU/RAM data to SQLite.
* **Deficiency:** Over time, this table will explode. Querying 30 days of raw 5-second interval data will freeze the Hub.
* **Recommendation:** Implement "Downsampling" or "Rollups" within SQLite.

---

## 3. Strategic Roadmap (Sprints)

We will restructure development into four targeted sprints focused on *capabilities* rather than modules.

### Sprint 1: The "Watchtower" (Telemetry & Dashboard Polish)
**Goal:** Stabilize data ingestion and make the dashboard usable for 100+ agents.

* [ ] **Database Rollups:**
    * Create `telemetry_hourly` and `telemetry_daily` tables.
    * Implement a background job in `optio-hub` that aggregates raw 5-minute samples into hourly averages/max/min every hour.
    * Prune raw data older than 24 hours.
* [ ] **"Thundering Herd" Management:**
    * Implement a **Command Queue Buffer** in the Hub.
    * Throttle UI updates using `useThrottle` or `react-query` to prevent UI lag when 100+ agents connect simultaneously.
* [ ] **Alerting Engine:**
    * Create `alerts` table.
    * Implement logic: If `cpu > 90%` for 3 checks -> Create Alert.
    * (Optional) Webhook dispatch (Slack/Discord).

### Sprint 2: The "Ghost" (Remote Control)
**Goal:** Pixel-perfect remote desktop via existing mTLS tunnel.

* [ ] **Screen Capture (Agent):**
    * Use the `dxgi-capture-rs` crate (Windows Desktop Duplication API) for high-performance capture.
    * Implement fallback to `BitBlt` for older systems.
* [ ] **Encoding:**
    * Implement LZ4 compression or H.264 encoding (if binary size permits) to replace raw bitmap transfer.
* [ ] **Transport:**
    * Define a `bi-directional streaming` gRPC method: `StreamDesktop(stream InputEvent) returns (stream FrameData)`.
* [ ] **Input Injection:**
    * Use `enigo` or `rdev` crate on the Agent to simulate mouse/keyboard events received from the Hub.

### Sprint 3: The "Mechanic" (Native Patching & Automation)
**Goal:** Move beyond "scripts" to "native management".

* [ ] **Windows Update COM Wrapper:**
    * Build a safe Rust wrapper around `windows::Win32::System::UpdateAssessment`.
    * Capabilities: `CheckForUpdates`, `InstallUpdates(kb_ids)`, `GetUpdateHistory`.
* [ ] **Service Management:**
    * Implement native commands to `List`, `Start`, `Stop`, `Restart` Windows services without shelling out to `sc.exe`.
* [ ] **Registry Editor:**
    * Implement Remote Registry CRUD operations via Rust's `winreg` crate.

### Sprint 4: The "Fortress" (Security & Polish)
**Goal:** Prepare for public release.

* [ ] **Code Signing:**
    * Sign the Agent binary to avoid SmartScreen blocks (requires EV Certificate).
* [ ] **Hub Multi-Window Support:**
    * Utilize Tauri v2 multi-window capabilities to spawn Remote Desktop sessions in separate native windows.
* [ ] **Audit Log Export:**
    * Implement CSV/PDF export of the `audit_log` table for compliance reporting.

---

## 4. Immediate Action Items (Developer Checklist)

### @rust-backend
* [ ] **Research `windows` crate:** Specifically `ISearcher` and `IUpdateSession` for the Patch Management module.
* [ ] **Prototype Screen Capture:** Create a POC using `dxgi-capture-rs` to verify frame capture efficiency.
* [ ] **Optimize SQLite:** Write the SQL migration for `telemetry_hourly` rollup tables.

### @react-frontend
* [ ] **Virtualize Lists:** Implement `react-window` or `tanstack-virtual` for the Agent Grid to handle large lists.
* [ ] **Canvas Renderer:** Prototype a React component capable of rendering a stream of Blob/Image data for Remote Desktop.

### @security-audit
* [ ] **Input Injection Safety:** Define security boundaries to ensure Remote Desktop input streams cannot be hijacked or injected into from outside the mTLS context.

---

## 5. Risks & Blind Spots

**Blind Spot:** Antivirus Evasion.
As we add "Remote Control" (screen capture/input injection) and "Patching" (system modification), Windows Defender and EDRs will increasingly flag `optio-agent.exe` as malware or PUA.

**Mitigation Strategy:**
1.  **EV Code Signing:** Mandatory for reputation building.
2.  **Microsoft Submission:** Submit the signed binary for whitelisting.
3.  **Behavioral Analysis:** Ensure Agent behavior is predictable and tied strictly to authenticated Hub commands.
