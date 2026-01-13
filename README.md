Optio Enterprise RMM

Optio is a high-performance, security-first Remote Monitoring & Management (RMM) platform designed for IT Consultants and MSPs. It replaces bloated, insecure management tools with a lightweight Rust agent and a modern Tauri-based Command Hub.

🏗 Architecture

Optio operates on a Distributed Agent Topology secured by Mutual TLS (mTLS).

Optio Hub (optio-hub): The Command Center. A local Tauri desktop application (React frontend) that acts as the gRPC Server, Certificate Authority (CA), and Dashboard.

Optio Agent (optio-agent): The Execution Arm. A headless, self-updating Rust binary that runs as a system service on client endpoints. It polls the Hub for instructions.

Optio Core (optio-core): The Shared Brain. Contains Protocol Buffers (optio.proto), encryption logic, and domain models shared between Hub and Agent.

🚀 Getting Started (The Golden Path)

Follow these steps exactly to go from Zero to Heartbeat.

1. Prerequisites

Ensure your development environment has the following installed:

Rust (Stable): rustup update stable

Node.js (v18+): node -v

Protocol Buffers Compiler (protoc):

Windows: choco install protoc

Mac: brew install protobuf

Linux: apt install -y protobuf-compiler

Build Tools:

Windows: Visual Studio C++ Build Tools.

Linux: libwebkit2gtk-4.0-dev, build-essential, curl, wget, libssl-dev.

2. Installation & Setup

Clone the repository:

git clone [https://github.com/RustySnipers/optio.git](https://github.com/RustySnipers/optio.git)
cd optio


Install Frontend Dependencies:

npm install
cd frontend
npm install
cd ..


🔐 CRITICAL STEP: Generate Security Certificates
Optio will not run without mTLS certificates. Use the helper script to generate a local Development CA.

Linux/Mac (Bash):

chmod +x scripts/generate-test-certs.sh
./scripts/generate-test-certs.sh


Windows (PowerShell):

# Ensure you have OpenSSL installed (git bash has it)
./scripts/generate-test-certs.sh


This creates a certs/ directory containing ca.pem, server.pem, server.key, client.pem, and client.key.

3. Running the Hub (Server)

Start the Tauri application. This will compile the Rust backend and launch the UI.

# From the root directory
npm run tauri dev


The Dashboard should appear. You will see "gRPC Server listening on 0.0.0.0:50051" in the terminal logs.

4. Running a Test Agent

Open a new terminal window to simulate a client machine.

# Run the agent locally (it will connect to localhost by default)
cargo run -p optio-agent


Success Criteria:

The Agent terminal shows: [INFO] Connected to Hub at 127.0.0.1:50051.

The Hub terminal shows: [INFO] New Heartbeat from Agent <UUID>.

The Hub Dashboard (UI) shows "Online Agents: 1".

🎮 Operating the Platform

Generating Deployable Agents

To deploy to a real remote machine:

Go to the "Factory" tab in the Hub.

Enter the Client Name (e.g., "Acme Corp").

Click "Generate Installer".

The Hub will sign a new certificate for this specific agent and bundle it into a .zip file with the optio-agent.exe.

Deploy: Copy the ZIP to the target machine, unzip, and run install.ps1 as Administrator.

Using the Interactive Terminal

Select an active agent from the Dashboard.

Click "Open Terminal".

A live PowerShell session will open. Commands are executed over the encrypted gRPC stream.

Audit Logs

All actions (Script execution, Terminal access, File transfers) are immutable logged.

Navigate to "Reporting Center" -> "Audit Logs" to view the chain of custody.

📦 Production Build & Release

We use GitHub Actions for CI/CD. Do not manually build production binaries unless testing.

Creating a Release

Tag the commit: git tag v1.0.0

Push tag: git push origin v1.0.0

The workflow .github/workflows/release.yml will:

Compile optio-agent (Windows x64).

Compile optio-hub (Windows MSI Installer).

Sign the binaries (if SIGNING_CERT secret is present).

Upload artifacts to GitHub Releases.

Self-Update Mechanism

Agents check for updates every 1 hour (with 0-10m random jitter).

They download the new binary from the Hub's /updates endpoint.

They verify the SHA256 hash.

They perform a "Rename-and-Replace" atomic update and restart the service.

🛠 Troubleshooting

Issue: "Database is locked"

Cause: Too many concurrent writes in SQLite.

Fix: Ensure WAL mode is enabled. The app handles this automatically on startup, but you can verify by checking for optio.db-wal in the app data folder.

Issue: Agent Connection Refused (mTLS Error)

Cause: The Agent's ca.pem does not match the Hub's server.pem.

Fix: Regenerate certs using the script and re-compile or re-copy the certs to the agent's folder.

Issue: "Thundering Herd" (High CPU on Hub)

Cause: 1000 agents connecting at once.

Fix: The agent includes randomization (Jitter) in its heartbeat loop. Do not remove the rand::thread_rng() logic in main.rs.

📜 License

This project is licensed under the MIT License.
