# Optio

**Consultant-in-a-Box**: A high-performance, local-first security toolkit for Enterprise Architects and IT Security Consultants.

## Overview

Optio is a Tauri-based desktop application that provides surgical, framework-agnostic tools for IT consulting engagements. Unlike traditional RMM solutions that are SaaS-dependent and noisy, Optio operates locally with a minimal footprint.

### Key Features

- **The Factory**: Dynamic script generation engine that manufactures unique, state-aware PowerShell scripts for each engagement
- **GRC Command Center**: Interactive audit, gap analysis, and policy generation (NIST CSF, SOC 2, GDPR)
- **Network Intelligence**: Asset discovery and vulnerability surfacing via Nmap integration
- **Local-First Security**: All sensitive data encrypted locally before optional cloud sync

## Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Core Runtime | Tauri v2 (Rust) | Minimal footprint (<10MB), high security |
| Frontend | React + TypeScript | Component modularity, rich UI ecosystem |
| Styling | Tailwind CSS | Rapid UI development, consistency |
| Automation | PowerShell Core | Native Windows management |
| Data Layer | SQLite (Encrypted) | Local storage with AES-256 encryption |

## Project Structure

```
optio/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri command handlers
│   │   ├── factory/        # Script generation engine
│   │   ├── db.rs           # SQLite database layer
│   │   ├── error.rs        # Error types
│   │   └── lib.rs          # Application entry
│   └── Cargo.toml
├── frontend/               # React frontend
│   ├── src/
│   │   ├── components/     # React components
│   │   ├── lib/            # Utilities & Tauri bindings
│   │   └── types/          # TypeScript interfaces
│   └── package.json
└── templates/              # PowerShell templates
```

## Getting Started

### Prerequisites

- Rust 1.70+ and Cargo
- Node.js 18+ and npm
- Tauri CLI v2

#### Linux System Dependencies

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libxdo-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel

# Arch
sudo pacman -Syu webkit2gtk-4.1 \
  base-devel \
  curl \
  wget \
  file \
  openssl \
  appmenu-gtk-module \
  gtk3 \
  libappindicator-gtk3 \
  librsvg
```

#### macOS

Xcode Command Line Tools are required:
```bash
xcode-select --install
```

#### Windows

No additional system dependencies required. Visual Studio Build Tools with C++ workload recommended.

### Installation

```bash
# Clone the repository
git clone https://github.com/RustySnipers/optio.git
cd optio

# Install dependencies
npm run install:all

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

## Modules

### Phase 1: The Factory (Current)

The Factory is Optio's dynamic client provisioning engine. It generates unique, idempotent PowerShell scripts with:

- **Identity Injection**: Consultant's IP/Certificate injected at generation time
- **State Auditing**: Scripts check existing state before applying changes
- **Air-Gap Friendly**: Scripts can be delivered via USB/Network share

### Phase 2: GRC Command Center (Planned)

- Framework Toggle (NIST CSF 2.0, SOC 2 Type II, GDPR)
- Evidence Collection & Control Mapping
- Compliance Gap Heatmaps

### Phase 3: Network Intelligence (Planned)

- Nmap wrapper for safe network scanning
- Asset inventory with OS/Port detection
- STRIDE threat modeling wizard

## Security Considerations

- **No Telemetry**: Operates assuming a hostile, air-gapped environment
- **Vault Storage**: Credentials stored using OS Keychain or AES-256 encrypted SQLite
- **Code Signing**: All binaries and generated scripts are signed

## Development

```bash
# Run Rust tests
npm run test:rust

# Type check Rust code
npm run check:rust

# Lint frontend
npm run lint

# Development server
npm run dev
```

## License

MIT License - See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read the contributing guidelines before submitting pull requests.
