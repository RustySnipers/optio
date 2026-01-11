# Optio Development Roadmap

> Prioritized implementation checklist for driving Optio to production status.
> Based on "Brand-Builder" specification requirements and current codebase gaps.

---

## Current Status Overview

| Module | Status | Completeness |
|--------|--------|--------------|
| **Factory (Script Generation)** | ✅ Implemented | ~80% |
| **GRC (Compliance)** | ✅ Implemented | ~75% |
| **Network Intelligence** | ✅ Implemented | ~70% |
| **Infrastructure** | ✅ Implemented | ~75% |
| **Reporting** | ✅ Implemented | ~60% |
| **Core Database** | ✅ Implemented | ~85% |

---

## Phase 1: Factory Module Enhancement

**Priority**: High
**Owner**: @rust-backend

### Tasks

- [ ] **1.1 PowerShell Template Expansion**
  - [ ] Ingest "Dynamic Client Provisioning" PowerShell templates
  - [ ] Add templates for: domain join, GPO deployment, SCCM agent
  - [ ] Implement template validation with PowerShell syntax checking
  - [ ] Add template versioning support

- [ ] **1.2 Variable Injection System**
  - [ ] Expand `ScriptConfigOptions` with additional identity fields
  - [ ] Implement secure variable escaping (prevent injection)
  - [ ] Add support for environment-specific variable sets
  - [ ] Create variable validation rules per template

- [ ] **1.3 Script Signing Integration**
  - [ ] Implement code signing certificate handling
  - [ ] Add signing as optional post-generation step
  - [ ] Store signing history in audit log
  - [ ] Support for timestamp server configuration

- [ ] **1.4 Air-Gap Delivery Enhancements**
  - [ ] Generate self-extracting archives with scripts
  - [ ] Include offline dependency bundles
  - [ ] Create deployment manifest files
  - [ ] Add checksum verification for integrity

---

## Phase 2: Network Module - Nmap Sidecar

**Priority**: High
**Owner**: @rust-backend

### Tasks

- [ ] **2.1 Tauri Sidecar Configuration**
  - [ ] Configure Nmap as sidecar in `tauri.conf.json`
  - [ ] Handle cross-platform Nmap binary paths
  - [ ] Implement fallback to system-installed Nmap
  - [ ] Add Nmap version detection and compatibility checks

- [ ] **2.2 Scanner.rs Enhancements**
  - [ ] Implement async scan execution with progress reporting
  - [ ] Add scan scheduling (cron-like) functionality
  - [ ] Implement scan result diffing (detect changes)
  - [ ] Add scan profile presets (quick, standard, comprehensive)

- [ ] **2.3 Asset Discovery Improvements**
  - [ ] Auto-categorize discovered assets by type
  - [ ] Implement MAC address vendor lookup
  - [ ] Add banner grabbing for service identification
  - [ ] Create asset relationship mapping

- [ ] **2.4 Vulnerability Correlation**
  - [ ] Map discovered services to known CVEs
  - [ ] Integrate with offline CVE database
  - [ ] Generate vulnerability summary per asset
  - [ ] Priority scoring based on CVSS

---

## Phase 3: GRC Module - Framework Data

**Priority**: High
**Owner**: @rust-backend, @security-audit

### Tasks

- [ ] **3.1 NIST CSF 2.0 Complete Implementation**
  - [ ] Verify all 6 functions fully represented
  - [ ] Add all 22 categories with descriptions
  - [ ] Include implementation examples per control
  - [ ] Map to ISO 27001 controls

- [ ] **3.2 GDPR Compliance Structure**
  - [ ] Complete Article-by-Article checklist (99 articles)
  - [ ] Add data processing agreement templates
  - [ ] Include DPIA (Data Protection Impact Assessment) workflow
  - [ ] Add data subject rights tracking

- [ ] **3.3 SOC 2 Type II Enhancement**
  - [ ] Complete Trust Services Criteria coverage
  - [ ] Add evidence collection templates
  - [ ] Include audit period tracking
  - [ ] Add auditor communication templates

- [ ] **3.4 Framework JSON Structure**
  - [ ] Migrate framework data to embedded JSON assets
  - [ ] Implement framework versioning
  - [ ] Add custom framework import capability
  - [ ] Create framework comparison views

---

## Phase 4: Reporting Module - PDF Generation

**Priority**: Medium-High
**Owner**: @rust-backend, @react-frontend

### Tasks

- [ ] **4.1 PDF Engine Implementation**
  - [ ] Integrate PDF generation library (`printpdf` or `wkhtmltopdf` sidecar)
  - [ ] Create base template with branding placeholders
  - [ ] Implement page layout system (header, footer, body)
  - [ ] Add chart/graph rendering to PDF

- [ ] **4.2 Report Templates**
  - [ ] Executive Summary template with metrics dashboard
  - [ ] Technical Findings template with code snippets
  - [ ] Compliance Gap Analysis template with matrices
  - [ ] Network Assessment template with topology diagrams
  - [ ] Risk Assessment template with heat maps

- [ ] **4.3 Content Assembly**
  - [ ] Dynamic section selection per report type
  - [ ] Cross-reference linking within documents
  - [ ] Table of contents auto-generation
  - [ ] Appendix management

- [ ] **4.4 Export Formats**
  - [ ] PDF (primary)
  - [ ] HTML (web-viewable)
  - [ ] Markdown (documentation)
  - [ ] DOCX (editable)
  - [ ] JSON (data exchange)

---

## Phase 5: Security Hardening

**Priority**: High
**Owner**: @security-audit

### Tasks

- [ ] **5.1 Input Validation Audit**
  - [ ] Review all 76+ commands for input validation
  - [ ] Add length limits to all string inputs
  - [ ] Implement regex validation for structured data
  - [ ] Add rate limiting for resource-intensive operations

- [ ] **5.2 Encryption Review**
  - [ ] Audit AES-256 implementation correctness
  - [ ] Verify key derivation function (use Argon2/scrypt)
  - [ ] Implement secure key storage (OS keychain integration)
  - [ ] Add encryption at rest for all sensitive tables

- [ ] **5.3 Capability Restrictions**
  - [ ] Minimize Tauri capability scope
  - [ ] Implement per-window capability sets
  - [ ] Add runtime permission prompts for sensitive operations
  - [ ] Document all required permissions

- [ ] **5.4 Dependency Audit**
  - [ ] Run `cargo audit` and resolve vulnerabilities
  - [ ] Pin dependency versions
  - [ ] Review transitive dependencies
  - [ ] Set up automated security scanning

---

## Phase 6: UI/UX Polish

**Priority**: Medium
**Owner**: @react-frontend

### Tasks

- [ ] **6.1 Dashboard Enhancements**
  - [ ] Add real-time system metrics
  - [ ] Create activity feed/timeline
  - [ ] Implement quick action shortcuts
  - [ ] Add notification system

- [ ] **6.2 Workflow Improvements**
  - [ ] Add wizard-style multi-step forms
  - [ ] Implement auto-save for long forms
  - [ ] Add keyboard shortcuts for power users
  - [ ] Implement undo/redo for destructive actions

- [ ] **6.3 Data Visualization**
  - [ ] Add compliance score charts
  - [ ] Create network topology visualization
  - [ ] Implement risk heat maps
  - [ ] Add trend analysis graphs

- [ ] **6.4 Accessibility**
  - [ ] WCAG 2.1 AA compliance
  - [ ] Keyboard navigation support
  - [ ] Screen reader compatibility
  - [ ] High contrast mode

---

## Phase 7: Testing & Quality

**Priority**: High
**Owner**: All agents

### Tasks

- [ ] **7.1 Unit Tests**
  - [ ] Rust: Test all command handlers
  - [ ] Rust: Test encryption/decryption
  - [ ] Rust: Test database operations
  - [ ] TypeScript: Test utility functions

- [ ] **7.2 Integration Tests**
  - [ ] Test IPC round-trips
  - [ ] Test database migrations
  - [ ] Test file operations
  - [ ] Test sidecar execution

- [ ] **7.3 E2E Tests**
  - [ ] Client onboarding workflow
  - [ ] GRC assessment workflow
  - [ ] Network scan workflow
  - [ ] Report generation workflow

- [ ] **7.4 Performance Testing**
  - [ ] Database query optimization
  - [ ] Large dataset handling
  - [ ] Memory usage profiling
  - [ ] Startup time optimization

---

## Phase 8: Documentation & Release

**Priority**: Medium
**Owner**: All agents

### Tasks

- [ ] **8.1 User Documentation**
  - [ ] Getting Started guide
  - [ ] Feature walkthroughs
  - [ ] Troubleshooting guide
  - [ ] FAQ

- [ ] **8.2 Developer Documentation**
  - [ ] Architecture overview
  - [ ] API reference
  - [ ] Contributing guide
  - [ ] Security policy

- [ ] **8.3 Release Preparation**
  - [ ] Version numbering strategy
  - [ ] Changelog generation
  - [ ] Release notes template
  - [ ] Binary signing setup

- [ ] **8.4 Distribution**
  - [ ] Windows installer (MSI/MSIX)
  - [ ] macOS DMG
  - [ ] Linux packages (deb/rpm/AppImage)
  - [ ] Auto-update mechanism

---

## Immediate Next Steps (Sprint 1)

Based on priority and dependencies, the recommended order is:

1. **Factory Module** (1.1, 1.2) - Core revenue feature
2. **Network Module** (2.1, 2.2) - Enables scanning workflow
3. **GRC Module** (3.1, 3.4) - Framework data structure
4. **Security Hardening** (5.1, 5.2) - Critical for production
5. **Reporting** (4.1, 4.2) - Deliverable output

---

## Dependencies & Blockers

| Task | Depends On | Blocked By |
|------|------------|------------|
| 2.2 Scanner enhancements | 2.1 Sidecar config | Nmap binary availability |
| 4.1 PDF generation | None | PDF library selection |
| 3.4 JSON structure | 3.1-3.3 Framework data | None |
| 6.3 Data visualization | 4.4 Export formats | Chart library selection |

---

## Risk Register

| Risk | Impact | Mitigation |
|------|--------|------------|
| Nmap licensing on Windows | Medium | Use system Nmap, document requirements |
| PDF library size | Low | Use wkhtmltopdf sidecar |
| Framework data accuracy | High | Security review before release |
| Cross-platform testing | Medium | CI/CD for all platforms |

---

## Version Milestones

| Version | Features | Target |
|---------|----------|--------|
| **0.2.0** | Factory templates, Nmap sidecar | Sprint 1 |
| **0.3.0** | Complete GRC frameworks, PDF reports | Sprint 2 |
| **0.4.0** | Security hardening, E2E tests | Sprint 3 |
| **1.0.0** | Production release | Sprint 4 |

---

*Last updated: Phase 1 - Memory Bank Initialization*
