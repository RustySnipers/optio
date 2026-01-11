# CLAUDE.md - Optio Project Instructions

> Master instruction file for Claude Code sessions on the Optio project.
> **Optio**: Consultant-in-a-Box - Enterprise Architecture & Security Toolkit

---

## Build Commands

### Development
```bash
# Full Tauri development (backend + frontend)
npm run tauri dev

# Frontend only (hot reload)
cd frontend && npm run dev

# Backend only (Rust checks)
cd src-tauri && cargo build
```

### Production Build
```bash
# Complete production build
npm run tauri build

# Frontend production bundle
cd frontend && npm run build

# Rust release build
cd src-tauri && cargo build --release
```

### Testing & Linting
```bash
# Rust tests
npm run test:rust
# or: cd src-tauri && cargo test

# Rust linting & checks
npm run check:rust
# or: cd src-tauri && cargo clippy -- -D warnings

# Frontend linting
npm run lint
# or: cd frontend && npm run lint
```

### Dependency Installation
```bash
# Install all dependencies (root + frontend)
npm run install:all

# Frontend only
cd frontend && npm install

# Rust dependencies (automatic via cargo)
cd src-tauri && cargo fetch
```

---

## Code Style

### Rust (src-tauri/)
- **Edition**: 2021 (Rust 1.70+)
- **Linting**: Clippy compliant (`cargo clippy -- -D warnings`)
- **Error Handling**: Use `thiserror` for custom error types (see `error.rs`)
- **Async Runtime**: Tokio with `full` features
- **Serialization**: `serde` + `serde_json` for all IPC types
- **Logging**: `tracing` crate for structured logging
- **Database**: `rusqlite` with WAL mode
- **Encryption**: `aes-gcm` for sensitive data (credentials vault)

**Patterns**:
```rust
// Commands must be async and return Result<T, OptioError>
#[tauri::command]
async fn example_command(param: String) -> Result<ResponseType, OptioError> {
    // Validate input first
    // Perform operation
    // Return structured response
}

// Use thiserror for error definitions
#[derive(Debug, thiserror::Error, Serialize)]
pub enum OptioError {
    #[error("Database error: {0}")]
    Database(String),
    // ...
}
```

### TypeScript (frontend/)
- **Mode**: Strict TypeScript (`strict: true` in tsconfig.json)
- **Components**: Functional components with hooks (React 18)
- **Styling**: Tailwind CSS 3.4+ (utility-first)
- **State**: React Context or Zustand for global state
- **Icons**: Lucide React
- **IPC**: Type-safe wrappers in `lib/commands.ts`

**Patterns**:
```tsx
// Functional components with proper typing
interface Props {
  client: Client;
  onUpdate: (client: Client) => void;
}

const ClientCard: React.FC<Props> = ({ client, onUpdate }) => {
  // Use hooks for state and effects
  const [loading, setLoading] = useState(false);

  // Call Tauri commands via typed wrappers
  const handleSave = async () => {
    const result = await commands.updateClient(client);
  };

  return (/* JSX */);
};
```

---

## Architecture Map

### Backend → Frontend Module Mapping

| Backend Module | Commands Location | Frontend Component | Types Location |
|----------------|-------------------|-------------------|----------------|
| **Factory** | `commands/factory.rs` | `ClientOnboarding.tsx` | `types/index.ts` (Factory*) |
| **GRC** | `commands/grc.rs` | `GRCCommandCenter.tsx` | `types/index.ts` (Assessment*, Control*) |
| **Network** | `commands/network.rs` | `NetworkIntelligence.tsx` | `types/index.ts` (Scan*, Asset*) |
| **Infrastructure** | `commands/infrastructure.rs` | `InfrastructureMigration.tsx` | `types/index.ts` (Cloud*, K8s*, FinOps*) |
| **Reporting** | `commands/reporting.rs` | `ReportingCenter.tsx` | `types/index.ts` (Report*) |
| **Clients** | `commands/clients.rs` | `ClientOnboarding.tsx` | `types/index.ts` (Client*) |
| **System** | `commands/system.rs` | `Header.tsx`, `Dashboard.tsx` | `types/index.ts` (SystemInfo) |

### Directory Structure
```
optio/
├── src-tauri/                    # Rust Backend
│   ├── src/
│   │   ├── commands/             # Tauri IPC handlers (76+ commands)
│   │   │   ├── factory.rs        # Script generation commands
│   │   │   ├── clients.rs        # Client CRUD
│   │   │   ├── grc.rs            # Compliance commands
│   │   │   ├── infrastructure.rs # Cloud/K8s commands
│   │   │   ├── network.rs        # Nmap/scanning commands
│   │   │   ├── reporting.rs      # Report generation
│   │   │   └── system.rs         # System info
│   │   ├── factory/              # Script generation engine
│   │   ├── grc/                  # Governance/Risk/Compliance
│   │   │   ├── frameworks.rs     # NIST, SOC2, GDPR definitions
│   │   │   ├── models.rs         # Assessment data structures
│   │   │   └── repository.rs     # Database persistence
│   │   ├── network/              # Network intelligence
│   │   │   ├── scanner.rs        # Nmap integration
│   │   │   ├── inventory.rs      # Asset tracking
│   │   │   └── models.rs         # Scan/asset types
│   │   ├── infrastructure/       # Cloud & K8s
│   │   │   ├── cloud_readiness.rs
│   │   │   ├── k8s_hardening.rs
│   │   │   └── finops.rs
│   │   ├── reporting/            # Report generation
│   │   │   ├── generator.rs      # Content generation
│   │   │   └── templates.rs      # Built-in templates
│   │   ├── db.rs                 # SQLite + AES-256 encryption
│   │   ├── error.rs              # OptioError types
│   │   ├── lib.rs                # App initialization
│   │   └── main.rs               # Entry point
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── frontend/                     # React Frontend
│   ├── src/
│   │   ├── components/           # UI Components
│   │   │   ├── App.tsx           # Root + routing
│   │   │   ├── Sidebar.tsx       # Navigation
│   │   │   ├── Header.tsx        # Top bar
│   │   │   ├── Dashboard.tsx     # Overview
│   │   │   ├── ClientOnboarding.tsx    # Factory UI
│   │   │   ├── GRCCommandCenter.tsx    # Compliance
│   │   │   ├── NetworkIntelligence.tsx # Scanning
│   │   │   ├── InfrastructureMigration.tsx  # Cloud/K8s
│   │   │   └── ReportingCenter.tsx     # Reports
│   │   ├── lib/
│   │   │   ├── commands.ts       # Tauri command bindings
│   │   │   └── utils.ts          # Utilities
│   │   ├── types/
│   │   │   └── index.ts          # All TypeScript interfaces
│   │   ├── hooks/
│   │   │   └── index.ts          # Custom React hooks
│   │   ├── main.tsx              # React entry
│   │   └── index.css             # Global styles
│   ├── package.json
│   └── tailwind.config.js
│
├── CLAUDE.md                     # This file
├── AGENTS.md                     # Subagent definitions
├── roadmap.md                    # Implementation roadmap
└── README.md                     # Project documentation
```

---

## Security Requirements

### Tauri Capabilities
Located in `src-tauri/capabilities/default.json`:
- **shell**: Restricted to `open` command only
- **fs**: Scoped to `$APPDATA` and `$TEMP` directories
- **CSP**: Self-origin only (`default-src 'self'`)

### Sensitive Data Handling
1. **Credentials Vault**: AES-256-GCM encryption in SQLite
2. **No Frontend Secrets**: All sensitive operations in Rust
3. **Audit Logging**: All compliance actions logged to `audit_log` table
4. **Input Validation**: Mandatory in all Rust command handlers

### Code Signing
- All PowerShell scripts must be signable
- Release builds use code signing certificates
- Air-gap delivery supported (USB/network share)

---

## Database Schema

### Core Tables
| Table | Purpose |
|-------|---------|
| `clients` | Client profiles and metadata |
| `script_history` | Generated script audit trail |
| `audit_log` | Compliance audit records |
| `credentials_vault` | AES-256 encrypted credentials |
| `assessments` | GRC assessment records |
| `controls` | Framework control mappings |
| `evidence` | Compliance evidence attachments |
| `scans` | Network scan jobs |
| `assets` | Discovered network assets |
| `reports` | Generated reports |

---

## Key Dependencies

### Rust (Cargo.toml)
- `tauri` v2 + plugins (shell, dialog, fs, tray-icon)
- `tokio` (async runtime)
- `rusqlite` (SQLite)
- `aes-gcm` (encryption)
- `serde`/`serde_json` (serialization)
- `uuid`, `chrono` (IDs, timestamps)
- `tracing` (logging)

### Frontend (package.json)
- `react` 18.2.0
- `@tauri-apps/api` v2
- `lucide-react` (icons)
- `tailwindcss` 3.4.1
- `typescript` 5.4.2
- `vite` 5.1.6

---

## Common Tasks

### Adding a New Command
1. Define types in `src-tauri/src/<module>/models.rs`
2. Implement handler in `src-tauri/src/commands/<module>.rs`
3. Register in `lib.rs` invoke_handler
4. Add TypeScript types in `frontend/src/types/index.ts`
5. Add wrapper in `frontend/src/lib/commands.ts`
6. Use in component

### Adding a New Framework (GRC)
1. Define controls in `src-tauri/src/grc/frameworks.rs`
2. Add to `get_framework_controls()` function
3. Update frontend types if needed
4. Test via `GRCCommandCenter.tsx`

### Running Nmap Scans
1. Ensure Nmap is installed on system
2. Configure sidecar in `tauri.conf.json` (if needed)
3. Use `NetworkIntelligence.tsx` for UI
4. Scans execute via `network/scanner.rs`
