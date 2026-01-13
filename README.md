# Optio Enterprise RMM

Optio is a high-performance, security-first Remote Monitoring & Management (RMM) platform designed for IT Consultants and MSPs. It replaces bloated, insecure management tools with a lightweight Rust agent and a modern Tauri-based Command Hub.

## Architecture (Quick Context)

Optio operates on a Distributed Agent Topology secured by Mutual TLS (mTLS).

- **Optio Hub (`optio-hub`)**: The Command Center. A local Tauri desktop application (React frontend) that acts as the gRPC Server, Certificate Authority (CA), and Dashboard.
- **Optio Agent (`optio-agent`)**: The Execution Arm. A headless, self-updating Rust binary that runs as a system service on client endpoints. It polls the Hub for instructions.
- **Optio Core (`optio-core`)**: The Shared Brain. Contains Protocol Buffers (`optio.proto`), encryption logic, and domain models shared between Hub and Agent.

## 🚀 Getting Started (Step-by-Step)

### 1) Prerequisites

1. Install Rust (stable toolchain):
   - `rustup update stable`
2. Install Node.js (v18+):
   - `node -v`
3. Install Protocol Buffers compiler (`protoc`):
   - **Windows**: `choco install protoc`
   - **macOS**: `brew install protobuf`
   - **Linux (Debian/Ubuntu)**: `sudo apt-get install -y protobuf-compiler`
4. Install OS-specific build dependencies:
   - **Windows**:
     1. Visual Studio C++ Build Tools
     2. OpenSSL (Git Bash includes it, or install separately)
   - **macOS**:
     1. `brew install openssl`
   - **Linux (Debian/Ubuntu)**:
     1. `sudo apt-get install -y libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev pkg-config`

### 2) Clone & Install Dependencies

1. Clone the repo:
   ```bash
   git clone https://github.com/RustySnipers/optio.git
   cd optio
   ```
2. Install root dependencies:
   ```bash
   npm install
   ```
3. Install frontend dependencies:
   ```bash
   cd frontend
   npm install
   cd ..
   ```

### 3) Generate Development Certificates (mTLS)

> Optio will not run without mTLS certificates. These are **development-only**.

1. Run the helper script:
   ```bash
   chmod +x scripts/generate-test-certs.sh
   ./scripts/generate-test-certs.sh
   ```
2. Expected terminal output includes:
   ```text
   Optio mTLS Certificate Generator
   [1/3] Generating Certificate Authority...
   [2/3] Generating Server Certificate...
   [3/3] Generating Client Certificate...
   Certificates Generated Successfully
   Files:
   ca.crt  ca.key  client.crt  client.key  server.crt  server.key
   ```
3. Verify the files exist:
   ```bash
   ls -la certs
   ```

### 4) Run the Hub (Server)

1. Start the Tauri app (from repo root):
   ```bash
   npm run tauri dev
   ```
2. Expected log line:
   ```text
   gRPC Server listening on 0.0.0.0:50051
   ```

### 5) Run the Agent (Local Test)

1. In a new terminal (from repo root):
   ```bash
   cargo run -p optio-agent
   ```
2. Expected log line in the Agent terminal:
   ```text
   [INFO] Connected to Hub at 127.0.0.1:50051
   ```

### 6) Verify Success

1. Hub terminal should show:
   ```text
   [INFO] New Heartbeat from Agent <UUID>
   ```
2. Hub UI should show:
   - **Online Agents: 1**

### 7) Testing

Run these from the repo root:

1. All Rust workspace tests:
   ```bash
   cargo test
   ```
2. Targeted crate tests:
   ```bash
   cargo test -p optio-core
   cargo test -p optio-hub
   cargo test -p optio-agent
   ```
3. Frontend tests (Vitest):
   ```bash
   cd frontend
   npm run test
   cd ..
   ```
4. Optional frontend lint:
   ```bash
   cd frontend
   npm run lint
   cd ..
   ```

### 8) Operational Flows

#### Deploy an Agent
1. In the Hub, open the **Factory** tab.
2. Enter a client name (example: `Acme Corp`).
3. Click **Generate Installer**.
4. Copy the resulting `.zip` to the target machine.
5. Unzip and run `install.ps1` **as Administrator**.

#### Use the Interactive Terminal
1. From the Hub dashboard, select an active agent.
2. Click **Open Terminal**.
3. Execute PowerShell commands over the encrypted gRPC stream.

#### Audit Logs
1. Navigate to **Reporting Center → Audit Logs**.
2. Confirm immutable log entries for script execution, terminal access, and file transfers.

### 9) Production Build & Release

> Production builds are handled by GitHub Actions. Use these steps only for releases.

1. Create a version tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```
2. The workflow `.github/workflows/release.yml` will:
   - Compile `optio-agent` (Windows x64).
   - Compile `optio-hub` (Windows MSI Installer).
   - Sign binaries (if `SIGNING_CERT` is present).
   - Upload artifacts to GitHub Releases.

#### Self-Update Notes
1. Agents check for updates every 1 hour (with 0–10m jitter).
2. Agents download the new binary from the Hub `/updates` endpoint.
3. Agents verify the SHA256 hash.
4. Agents perform a rename-and-replace atomic update and restart the service.

### 10) Troubleshooting Checklist

- [ ] **"Database is locked"**
  - **Cause**: Too many concurrent writes in SQLite.
  - **Fix**: Verify WAL mode is enabled (look for `optio.db-wal` in app data).
- [ ] **Agent connection refused (mTLS error)**
  - **Cause**: Agent `ca.crt` does not match Hub `server.crt`.
  - **Fix**: Re-run `./scripts/generate-test-certs.sh` and redeploy certs.
- [ ] **Hub UI does not open**
  - **Cause**: Missing Tauri dependencies or frontend build issues.
  - **Fix**: Re-run `npm install` (root and `frontend/`) and retry `npm run tauri dev`.
- [ ] **Agent not showing Online**
  - **Cause**: Agent cannot reach Hub or certificates missing.
  - **Fix**: Confirm Hub running and `certs/` exists with `ca.crt`, `server.crt`, `client.crt`.
- [ ] **High CPU on Hub ("Thundering Herd")**
  - **Cause**: Too many agents connecting at once.
  - **Fix**: Keep the agent heartbeat jitter logic in place.

## 📜 License

This project is licensed under the MIT License.
