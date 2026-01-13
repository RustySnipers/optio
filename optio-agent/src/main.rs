//! Optio Agent - Lightweight endpoint sentinel
//!
//! The agent runs as a headless service that:
//! - Sends periodic heartbeats to the Hub with system metrics
//! - Receives and executes commands from the Hub
//! - Reports command results back to the Hub
//! - Provides interactive remote terminal sessions
//!
//! Communication is outbound-only (agent "phones home") using mTLS.
//!
//! # Running Modes
//!
//! - Console: `optio-agent` or `optio-agent --console`
//! - Windows Service: `optio-agent --service`
//! - Install Service: `optio-agent --install`
//! - Uninstall Service: `optio-agent --uninstall`

mod terminal;
mod update;

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use clap::Parser;
use futures::StreamExt;
use rand::Rng;
use update::UpdateConfig;
use optio_core::proto::collector_client::CollectorClient;
use optio_core::proto::terminal_service_server::{TerminalService, TerminalServiceServer};
use optio_core::proto::{
    ExecuteScriptResponse, HeartbeatRequest, PendingCommand, TerminalInput, TerminalOutput,
    TerminalStarted, TerminalOutputData, TerminalEnded, TerminalError as ProtoTerminalError,
};
use terminal::{TerminalConfig, TerminalManager, TerminalOutputEvent};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc as tokio_mpsc;
use tokio_stream::wrappers::ReceiverStream;
use sysinfo::System;
use thiserror::Error;
use tokio::process::Command;
use tokio::time::{interval, timeout};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity, Server, ServerTlsConfig};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

/// Agent version
const AGENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default heartbeat interval in seconds
const DEFAULT_HEARTBEAT_INTERVAL: u64 = 30;

/// Default command execution timeout in seconds
const DEFAULT_COMMAND_TIMEOUT: u64 = 300;

/// Namespace UUID for machine ID generation (using a fixed UUID5 namespace)
const MACHINE_ID_NAMESPACE: Uuid = Uuid::from_u128(0x6ba7b810_9dad_11d1_80b4_00c04fd430c8);

/// Service name for Windows SCM
const SERVICE_NAME: &str = "OptioAgent";

// ============================================================================
// Command Line Arguments
// ============================================================================

#[derive(Parser, Debug)]
#[command(name = "optio-agent", about = "Optio Agent - Endpoint sentinel for IT orchestration")]
struct Args {
    /// Run as a Windows service (called by SCM)
    #[arg(long)]
    service: bool,

    /// Install as a Windows service
    #[arg(long)]
    install: bool,

    /// Uninstall the Windows service
    #[arg(long)]
    uninstall: bool,

    /// Run in console mode (default)
    #[arg(long)]
    console: bool,

    /// Path to configuration file
    #[arg(long, short, default_value = "config.toml")]
    config: PathBuf,

    /// Show status and exit
    #[arg(long)]
    status: bool,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Connection to hub failed: {0}")]
    Connection(String),

    #[error("Heartbeat failed: {0}")]
    Heartbeat(String),

    #[error("Command execution failed: {0}")]
    CommandExecution(String),

    #[error("Command timeout after {0} seconds")]
    CommandTimeout(u64),

    #[error("TLS configuration error: {0}")]
    TlsConfig(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}

// ============================================================================
// Agent Configuration
// ============================================================================

/// Agent configuration loaded from file or environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent section
    #[serde(default)]
    pub agent: AgentSection,

    /// TLS configuration section
    #[serde(default)]
    pub tls: TlsSection,

    /// Logging configuration section
    #[serde(default)]
    pub logging: LoggingSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSection {
    /// Unique agent identifier
    #[serde(default)]
    pub id: Option<String>,

    /// Hub server address (e.g., "https://192.168.1.100:50051")
    #[serde(default = "default_hub_address")]
    pub hub_address: String,

    /// Heartbeat interval in seconds
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval: u64,

    /// Command execution timeout in seconds
    #[serde(default = "default_command_timeout")]
    pub command_timeout: u64,
}

impl Default for AgentSection {
    fn default() -> Self {
        Self {
            id: None,
            hub_address: default_hub_address(),
            heartbeat_interval: default_heartbeat_interval(),
            command_timeout: default_command_timeout(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TlsSection {
    /// Path to CA certificate PEM
    pub ca_path: Option<PathBuf>,

    /// Path to client certificate PEM
    pub cert_path: Option<PathBuf>,

    /// Path to client private key PEM
    pub key_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSection {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log to Windows Event Log when running as service
    #[serde(default)]
    pub event_log: bool,
}

impl Default for LoggingSection {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            event_log: false,
        }
    }
}

fn default_hub_address() -> String {
    "http://127.0.0.1:50051".to_string()
}

fn default_heartbeat_interval() -> u64 {
    DEFAULT_HEARTBEAT_INTERVAL
}

fn default_command_timeout() -> u64 {
    DEFAULT_COMMAND_TIMEOUT
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent: AgentSection::default(),
            tls: TlsSection::default(),
            logging: LoggingSection::default(),
        }
    }
}

impl AgentConfig {
    /// Load configuration from file
    pub fn from_file(path: &PathBuf) -> Result<Self, AgentError> {
        if !path.exists() {
            info!("Config file not found at {:?}, using defaults", path);
            return Ok(Self::from_env());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| AgentError::Config(format!("Failed to read config: {}", e)))?;

        let config: AgentConfig = toml::from_str(&content)
            .map_err(|e| AgentError::Config(format!("Failed to parse config: {}", e)))?;

        // Override with environment variables if set
        Ok(config.with_env_overrides())
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            agent: AgentSection {
                id: std::env::var("OPTIO_AGENT_ID").ok(),
                hub_address: std::env::var("OPTIO_HUB_ADDRESS")
                    .unwrap_or_else(|_| default_hub_address()),
                heartbeat_interval: std::env::var("OPTIO_HEARTBEAT_INTERVAL")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_HEARTBEAT_INTERVAL),
                command_timeout: std::env::var("OPTIO_COMMAND_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_COMMAND_TIMEOUT),
            },
            tls: TlsSection {
                cert_path: std::env::var("OPTIO_CERT_PATH").ok().map(PathBuf::from),
                key_path: std::env::var("OPTIO_KEY_PATH").ok().map(PathBuf::from),
                ca_path: std::env::var("OPTIO_CA_PATH").ok().map(PathBuf::from),
            },
            logging: LoggingSection::default(),
        }
    }

    /// Apply environment variable overrides
    fn with_env_overrides(mut self) -> Self {
        if let Ok(addr) = std::env::var("OPTIO_HUB_ADDRESS") {
            self.agent.hub_address = addr;
        }
        if let Ok(interval) = std::env::var("OPTIO_HEARTBEAT_INTERVAL") {
            if let Ok(val) = interval.parse() {
                self.agent.heartbeat_interval = val;
            }
        }
        if let Ok(path) = std::env::var("OPTIO_CERT_PATH") {
            self.tls.cert_path = Some(PathBuf::from(path));
        }
        if let Ok(path) = std::env::var("OPTIO_KEY_PATH") {
            self.tls.key_path = Some(PathBuf::from(path));
        }
        if let Ok(path) = std::env::var("OPTIO_CA_PATH") {
            self.tls.ca_path = Some(PathBuf::from(path));
        }
        self
    }

    /// Check if mTLS is configured
    pub fn has_mtls(&self) -> bool {
        self.tls.cert_path.is_some() && self.tls.key_path.is_some() && self.tls.ca_path.is_some()
    }

    /// Resolve paths relative to config file location
    pub fn resolve_paths(&mut self, config_dir: &PathBuf) {
        if let Some(ref path) = self.tls.ca_path {
            if path.is_relative() {
                self.tls.ca_path = Some(config_dir.join(path));
            }
        }
        if let Some(ref path) = self.tls.cert_path {
            if path.is_relative() {
                self.tls.cert_path = Some(config_dir.join(path));
            }
        }
        if let Some(ref path) = self.tls.key_path {
            if path.is_relative() {
                self.tls.key_path = Some(config_dir.join(path));
            }
        }
    }
}

// ============================================================================
// System Information Collector
// ============================================================================

/// Collects system information for heartbeats
pub struct SystemCollector {
    system: System,
    machine_id: String,
    hostname: String,
    os_info: String,
    start_time: Instant,
}

impl SystemCollector {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        // Generate a stable machine ID based on hostname and OS
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let os_info = format!(
            "{} {} ({})",
            System::name().unwrap_or_else(|| "Unknown".to_string()),
            System::os_version().unwrap_or_else(|| "".to_string()),
            System::kernel_version().unwrap_or_else(|| "".to_string()),
        );

        // Create a stable machine ID using UUID5 (deterministic based on hostname + OS)
        let machine_id = Uuid::new_v5(&MACHINE_ID_NAMESPACE, format!("{}:{}", hostname, os_info).as_bytes())
            .to_string();

        Self {
            system,
            machine_id,
            hostname,
            os_info,
            start_time: Instant::now(),
        }
    }

    /// Use a specific agent ID instead of auto-generated
    pub fn with_agent_id(mut self, id: String) -> Self {
        self.machine_id = id;
        self
    }

    /// Refresh system metrics
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    /// Get current CPU usage percentage
    pub fn cpu_usage(&self) -> f32 {
        self.system.global_cpu_usage()
    }

    /// Get current RAM usage percentage
    pub fn ram_usage(&self) -> f32 {
        let total = self.system.total_memory();
        let used = self.system.used_memory();
        if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Get total RAM in bytes
    pub fn ram_total(&self) -> u64 {
        self.system.total_memory()
    }

    /// Get available disk space (root partition)
    pub fn disk_info(&self) -> (u64, u64) {
        let disks = sysinfo::Disks::new_with_refreshed_list();
        for disk in disks.list() {
            // Get root partition on Unix or C: on Windows
            let mount = disk.mount_point().to_string_lossy();
            if mount == "/" || mount.starts_with("C:") {
                return (disk.available_space(), disk.total_space());
            }
        }
        (0, 0)
    }

    /// Get agent uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get local IP addresses
    pub fn ip_addresses(&self) -> String {
        // Simple approach - get from network interfaces via hostname
        local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "127.0.0.1".to_string())
    }

    /// Build a heartbeat request with current system state
    pub fn build_heartbeat(&mut self) -> HeartbeatRequest {
        self.refresh();
        let (disk_free, disk_total) = self.disk_info();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        HeartbeatRequest {
            machine_id: self.machine_id.clone(),
            agent_version: AGENT_VERSION.to_string(),
            hostname: self.hostname.clone(),
            os_info: self.os_info.clone(),
            cpu_usage: self.cpu_usage(),
            ram_usage: self.ram_usage(),
            ram_total: self.ram_total(),
            disk_free,
            disk_total,
            uptime_seconds: self.uptime_seconds(),
            ip_addresses: self.ip_addresses(),
            timestamp,
        }
    }

    pub fn machine_id(&self) -> &str {
        &self.machine_id
    }
}

// Simple local IP detection
mod local_ip_address {
    use std::net::IpAddr;

    pub fn local_ip() -> Result<IpAddr, std::io::Error> {
        use std::net::UdpSocket;
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect("8.8.8.8:80")?;
        let addr = socket.local_addr()?;
        Ok(addr.ip())
    }
}

// ============================================================================
// Command Executor
// ============================================================================

/// Executes commands received from the Hub
pub struct CommandExecutor {
    machine_id: String,
    default_timeout: Duration,
}

impl CommandExecutor {
    pub fn new(machine_id: String, timeout_seconds: u64) -> Self {
        Self {
            machine_id,
            default_timeout: Duration::from_secs(timeout_seconds),
        }
    }

    /// Execute a pending command from the hub
    pub async fn execute(&self, command: &PendingCommand) -> ExecuteScriptResponse {
        let started_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        info!(
            command_id = %command.command_id,
            command_type = %command.command_type,
            "Executing command"
        );

        let result = match command.command_type.as_str() {
            "execute_script" => self.execute_script(command).await,
            _ => Err(AgentError::CommandExecution(format!(
                "Unknown command type: {}",
                command.command_type
            ))),
        };

        let finished_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let duration_ms = (finished_at - started_at) * 1000;

        match result {
            Ok(response) => ExecuteScriptResponse {
                command_id: command.command_id.clone(),
                machine_id: self.machine_id.clone(),
                success: response.success,
                exit_code: response.exit_code,
                stdout: response.stdout,
                stderr: response.stderr,
                error_message: response.error_message,
                started_at,
                finished_at,
                duration_ms,
            },
            Err(e) => ExecuteScriptResponse {
                command_id: command.command_id.clone(),
                machine_id: self.machine_id.clone(),
                success: false,
                exit_code: -1,
                stdout: String::new(),
                stderr: String::new(),
                error_message: e.to_string(),
                started_at,
                finished_at,
                duration_ms,
            },
        }
    }

    /// Execute a script command (PowerShell, Bash, or CMD)
    async fn execute_script(&self, command: &PendingCommand) -> Result<ScriptResult, AgentError> {
        // Parse the payload JSON
        let payload: ScriptPayload = serde_json::from_str(&command.payload_json)
            .map_err(|e| AgentError::CommandExecution(format!("Invalid payload: {}", e)))?;

        let timeout_secs = if command.timeout_seconds > 0 {
            command.timeout_seconds as u64
        } else {
            self.default_timeout.as_secs()
        };

        let script_content = if payload.is_encoded {
            // Decode base64
            String::from_utf8(
                BASE64
                    .decode(&payload.script_content)
                    .map_err(|e| AgentError::CommandExecution(format!("Base64 decode failed: {}", e)))?,
            )
            .map_err(|e| AgentError::CommandExecution(format!("UTF8 decode failed: {}", e)))?
        } else {
            payload.script_content.clone()
        };

        // Build the command based on script type
        let (program, args) = match payload.script_type.to_lowercase().as_str() {
            "powershell" => {
                // Encode script for PowerShell -EncodedCommand
                let encoded = BASE64.encode(script_content.encode_utf16().flat_map(|c| c.to_le_bytes()).collect::<Vec<u8>>());
                (
                    "powershell.exe",
                    vec![
                        "-NoProfile".to_string(),
                        "-NonInteractive".to_string(),
                        "-ExecutionPolicy".to_string(),
                        "Bypass".to_string(),
                        "-EncodedCommand".to_string(),
                        encoded,
                    ],
                )
            }
            "bash" => ("bash", vec!["-c".to_string(), script_content]),
            "sh" => ("sh", vec!["-c".to_string(), script_content]),
            "cmd" => ("cmd.exe", vec!["/C".to_string(), script_content]),
            _ => {
                return Err(AgentError::CommandExecution(format!(
                    "Unsupported script type: {}",
                    payload.script_type
                )));
            }
        };

        // Execute with timeout
        let result = timeout(Duration::from_secs(timeout_secs), async {
            let mut cmd = Command::new(program);
            cmd.args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);

            // Set working directory if specified
            if !payload.working_directory.is_empty() {
                cmd.current_dir(&payload.working_directory);
            }

            cmd.output().await
        })
        .await
        .map_err(|_| AgentError::CommandTimeout(timeout_secs))?
        .map_err(AgentError::Io)?;

        let stdout = String::from_utf8_lossy(&result.stdout).to_string();
        let stderr = String::from_utf8_lossy(&result.stderr).to_string();
        let exit_code = result.status.code().unwrap_or(-1);

        debug!(
            exit_code = exit_code,
            stdout_len = stdout.len(),
            stderr_len = stderr.len(),
            "Script execution completed"
        );

        Ok(ScriptResult {
            success: result.status.success(),
            exit_code,
            stdout,
            stderr,
            error_message: String::new(),
        })
    }
}

/// Payload for execute_script commands
#[derive(Debug, Deserialize)]
struct ScriptPayload {
    script_type: String,
    script_content: String,
    #[serde(default)]
    is_encoded: bool,
    #[serde(default)]
    working_directory: String,
}

/// Result of script execution
struct ScriptResult {
    success: bool,
    exit_code: i32,
    stdout: String,
    stderr: String,
    error_message: String,
}

// ============================================================================
// Hub Client
// ============================================================================

/// Client for communicating with the Optio Hub
pub struct HubClient {
    client: CollectorClient<Channel>,
}

impl HubClient {
    /// Create a new hub client without TLS (development only)
    pub async fn connect(address: &str) -> Result<Self> {
        info!("Connecting to hub at {} (insecure)", address);

        let channel = Channel::from_shared(address.to_string())
            .context("Invalid hub address")?
            .connect()
            .await
            .context("Failed to connect to hub")?;

        Ok(Self {
            client: CollectorClient::new(channel),
        })
    }

    /// Create a new hub client with mTLS
    pub async fn connect_mtls(
        address: &str,
        cert_path: &PathBuf,
        key_path: &PathBuf,
        ca_path: &PathBuf,
    ) -> Result<Self> {
        info!("Connecting to hub at {} with mTLS", address);

        // Read certificate files
        let cert = std::fs::read_to_string(cert_path)
            .context("Failed to read client certificate")?;
        let key = std::fs::read_to_string(key_path)
            .context("Failed to read client key")?;
        let ca_cert = std::fs::read_to_string(ca_path)
            .context("Failed to read CA certificate")?;

        // Build TLS config
        let identity = Identity::from_pem(&cert, &key);
        let ca = Certificate::from_pem(&ca_cert);

        let tls_config = ClientTlsConfig::new()
            .identity(identity)
            .ca_certificate(ca)
            .domain_name("optio-hub");

        let channel = Channel::from_shared(address.to_string())
            .context("Invalid hub address")?
            .tls_config(tls_config)
            .context("TLS configuration failed")?
            .connect()
            .await
            .context("Failed to connect to hub")?;

        Ok(Self {
            client: CollectorClient::new(channel),
        })
    }

    /// Send a heartbeat and receive pending commands
    pub async fn heartbeat(
        &mut self,
        request: HeartbeatRequest,
    ) -> Result<Vec<PendingCommand>> {
        let response = self
            .client
            .heartbeat(request)
            .await
            .context("Heartbeat RPC failed")?
            .into_inner();

        if response.accepted {
            debug!(
                message = %response.message,
                next_interval = response.next_heartbeat_seconds,
                pending_commands = response.pending_commands.len(),
                "Heartbeat accepted"
            );
        } else {
            warn!(message = %response.message, "Heartbeat rejected");
        }

        Ok(response.pending_commands)
    }
}

// ============================================================================
// Terminal gRPC Service Implementation
// ============================================================================

/// gRPC Terminal Service implementation for interactive shell sessions
pub struct TerminalServiceImpl {
    terminal_manager: Arc<TerminalManager>,
    machine_id: String,
}

impl TerminalServiceImpl {
    pub fn new(machine_id: String) -> Self {
        Self {
            terminal_manager: Arc::new(TerminalManager::new()),
            machine_id,
        }
    }
}

#[tonic::async_trait]
impl TerminalService for TerminalServiceImpl {
    type StartTerminalStream = Pin<
        Box<dyn futures::Stream<Item = Result<TerminalOutput, tonic::Status>> + Send + 'static>,
    >;

    async fn start_terminal(
        &self,
        request: tonic::Request<tonic::Streaming<TerminalInput>>,
    ) -> Result<tonic::Response<Self::StartTerminalStream>, tonic::Status> {
        let mut input_stream = request.into_inner();
        let terminal_manager = Arc::clone(&self.terminal_manager);
        let machine_id = self.machine_id.clone();

        // Channel for sending output back to the client
        let (output_tx, output_rx) = tokio_mpsc::channel::<Result<TerminalOutput, tonic::Status>>(256);

        // Spawn task to handle the terminal session
        tokio::spawn(async move {
            let mut session_id: Option<String> = None;
            let mut pty_output_rx: Option<tokio_mpsc::Receiver<TerminalOutputEvent>> = None;

            loop {
                tokio::select! {
                    // Handle incoming input from the client
                    Some(input_result) = input_stream.next() => {
                        let input = match input_result {
                            Ok(i) => i,
                            Err(e) => {
                                error!("Error receiving terminal input: {}", e);
                                break;
                            }
                        };

                        // Validate machine_id
                        if !input.machine_id.is_empty() && input.machine_id != machine_id {
                            let _ = output_tx.send(Ok(TerminalOutput {
                                session_id: input.session_id.clone(),
                                output: Some(optio_core::proto::terminal_output::Output::Error(
                                    ProtoTerminalError {
                                        code: "INVALID_MACHINE".to_string(),
                                        message: "Machine ID mismatch".to_string(),
                                    }
                                )),
                            })).await;
                            continue;
                        }

                        match input.input {
                            // Start a new terminal session
                            Some(optio_core::proto::terminal_input::Input::Start(start_req)) => {
                                let sid = input.session_id.clone();
                                let config = TerminalConfig {
                                    shell_type: if start_req.shell_type.is_empty() {
                                        #[cfg(windows)]
                                        { "powershell".to_string() }
                                        #[cfg(not(windows))]
                                        { "bash".to_string() }
                                    } else {
                                        start_req.shell_type.clone()
                                    },
                                    cols: start_req.cols as u16,
                                    rows: start_req.rows as u16,
                                    working_directory: if start_req.working_directory.is_empty() {
                                        None
                                    } else {
                                        Some(start_req.working_directory.clone())
                                    },
                                };

                                match terminal_manager.start_session(sid.clone(), config.clone()) {
                                    Ok(()) => {
                                        info!(session_id = %sid, "Terminal session created");
                                        session_id = Some(sid.clone());

                                        // Get the output reader
                                        if let Ok(rx) = terminal_manager.with_session(&sid, |session| {
                                            session.start_output_reader()
                                        }) {
                                            pty_output_rx = Some(rx);
                                        }

                                        // Get the PID and send started message
                                        let (shell_type, pid) = terminal_manager.with_session(&sid, |session| {
                                            Ok((session.shell_type().to_string(), session.pid()))
                                        }).unwrap_or(("unknown".to_string(), 0));

                                        let _ = output_tx.send(Ok(TerminalOutput {
                                            session_id: sid.clone(),
                                            output: Some(optio_core::proto::terminal_output::Output::Started(
                                                TerminalStarted {
                                                    shell_type,
                                                    pid,
                                                }
                                            )),
                                        })).await;
                                    }
                                    Err(e) => {
                                        error!(session_id = %sid, error = %e, "Failed to start terminal");
                                        let _ = output_tx.send(Ok(TerminalOutput {
                                            session_id: sid,
                                            output: Some(optio_core::proto::terminal_output::Output::Error(
                                                ProtoTerminalError {
                                                    code: "START_FAILED".to_string(),
                                                    message: e.to_string(),
                                                }
                                            )),
                                        })).await;
                                    }
                                }
                            }

                            // Send data to the terminal
                            Some(optio_core::proto::terminal_input::Input::Data(data)) => {
                                if let Some(ref sid) = session_id {
                                    if let Err(e) = terminal_manager.with_session(sid, |session| {
                                        session.write_input(&data.data)
                                    }) {
                                        warn!(session_id = %sid, error = %e, "Failed to write to terminal");
                                    }
                                }
                            }

                            // Resize the terminal
                            Some(optio_core::proto::terminal_input::Input::Resize(resize)) => {
                                if let Some(ref sid) = session_id {
                                    if let Err(e) = terminal_manager.with_session(sid, |session| {
                                        session.resize(resize.cols as u16, resize.rows as u16)
                                    }) {
                                        warn!(session_id = %sid, error = %e, "Failed to resize terminal");
                                    }
                                }
                            }

                            // Handle signal
                            Some(optio_core::proto::terminal_input::Input::Signal(_signal)) => {
                                // Signal handling - for now just log
                                if let Some(ref sid) = session_id {
                                    debug!(session_id = %sid, "Signal received (not fully implemented)");
                                }
                            }

                            // Close the terminal
                            Some(optio_core::proto::terminal_input::Input::Close(close)) => {
                                if let Some(ref sid) = session_id {
                                    info!(session_id = %sid, reason = %close.reason, "Closing terminal session");
                                    terminal_manager.remove_session(sid);
                                    let _ = output_tx.send(Ok(TerminalOutput {
                                        session_id: sid.clone(),
                                        output: Some(optio_core::proto::terminal_output::Output::Ended(
                                            TerminalEnded {
                                                exit_code: 0,
                                                reason: close.reason,
                                            }
                                        )),
                                    })).await;
                                }
                                break;
                            }

                            None => {}
                        }
                    }

                    // Handle output from the PTY
                    Some(event) = async {
                        if let Some(ref mut rx) = pty_output_rx {
                            rx.recv().await
                        } else {
                            futures::future::pending::<Option<TerminalOutputEvent>>().await
                        }
                    } => {
                        if let Some(ref sid) = session_id {
                            match event {
                                TerminalOutputEvent::Data(data) => {
                                    let _ = output_tx.send(Ok(TerminalOutput {
                                        session_id: sid.clone(),
                                        output: Some(optio_core::proto::terminal_output::Output::Data(
                                            TerminalOutputData { data }
                                        )),
                                    })).await;
                                }
                                TerminalOutputEvent::Ended { exit_code, reason } => {
                                    let _ = output_tx.send(Ok(TerminalOutput {
                                        session_id: sid.clone(),
                                        output: Some(optio_core::proto::terminal_output::Output::Ended(
                                            TerminalEnded { exit_code, reason }
                                        )),
                                    })).await;
                                    terminal_manager.remove_session(sid);
                                    break;
                                }
                                TerminalOutputEvent::Error { code, message } => {
                                    let _ = output_tx.send(Ok(TerminalOutput {
                                        session_id: sid.clone(),
                                        output: Some(optio_core::proto::terminal_output::Output::Error(
                                            ProtoTerminalError { code, message }
                                        )),
                                    })).await;
                                }
                            }
                        }
                    }

                    else => break,
                }
            }

            // Cleanup session on exit
            if let Some(sid) = session_id {
                terminal_manager.remove_session(&sid);
            }
        });

        let output_stream = ReceiverStream::new(output_rx);
        Ok(tonic::Response::new(Box::pin(output_stream)))
    }
}

// ============================================================================
// Agent Main Loop
// ============================================================================

/// The main agent that ties everything together
pub struct Agent {
    config: AgentConfig,
    collector: SystemCollector,
    executor: CommandExecutor,
}

impl Agent {
    pub fn new(config: AgentConfig) -> Self {
        let mut collector = SystemCollector::new();

        // Use configured agent ID if provided
        if let Some(ref id) = config.agent.id {
            collector = collector.with_agent_id(id.clone());
        }

        let executor = CommandExecutor::new(
            collector.machine_id().to_string(),
            config.agent.command_timeout,
        );

        Self {
            config,
            collector,
            executor,
        }
    }

    /// Run the agent main loop
    pub async fn run(&mut self) -> Result<()> {
        info!(
            machine_id = %self.collector.machine_id(),
            hostname = %self.collector.hostname,
            version = AGENT_VERSION,
            "Starting Optio Agent"
        );

        // Start the update loop in the background
        let update_config = UpdateConfig {
            hub_address: self.config.agent.hub_address.replace("grpc://", "http://").replace(":50051", ":8080"),
            check_interval: Duration::from_secs(3600), // 1 hour
            current_exe: std::env::current_exe().unwrap_or_default(),
            ca_cert_path: self.config.tls.ca_path.clone(),
            client_cert_path: self.config.tls.cert_path.clone(),
            client_key_path: self.config.tls.key_path.clone(),
            enabled: true,
        };

        let _update_handle = update::spawn_update_loop(update_config);
        info!("Update loop started (checking every hour)");

        // Connect to hub
        let mut client = self.connect().await?;

        // Thundering Herd Protection: Add initial startup jitter
        // This prevents all agents from sending heartbeats at exactly the same time
        // after a mass restart (e.g., power outage recovery, mass deployment)
        let startup_jitter = rand::thread_rng().gen_range(0..30); // 0-30 seconds
        if startup_jitter > 0 {
            info!("Initial heartbeat delay: {}s (jitter for load distribution)", startup_jitter);
            tokio::time::sleep(Duration::from_secs(startup_jitter)).await;
        }

        // Main heartbeat loop
        let mut heartbeat_interval = interval(Duration::from_secs(self.config.agent.heartbeat_interval));

        loop {
            heartbeat_interval.tick().await;

            // Build and send heartbeat
            let heartbeat = self.collector.build_heartbeat();

            match client.heartbeat(heartbeat).await {
                Ok(pending_commands) => {
                    // Execute any pending commands
                    for command in pending_commands {
                        let result = self.executor.execute(&command).await;
                        info!(
                            command_id = %command.command_id,
                            success = result.success,
                            exit_code = result.exit_code,
                            "Command execution completed"
                        );
                        // TODO: Report result back to hub via a dedicated RPC
                    }
                }
                Err(e) => {
                    error!("Heartbeat failed: {}", e);
                    // Try to reconnect
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    match self.connect().await {
                        Ok(new_client) => {
                            client = new_client;
                            info!("Reconnected to hub");
                        }
                        Err(e) => {
                            error!("Reconnection failed: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Connect to the hub (with or without mTLS)
    async fn connect(&self) -> Result<HubClient> {
        if self.config.has_mtls() {
            HubClient::connect_mtls(
                &self.config.agent.hub_address,
                self.config.tls.cert_path.as_ref().expect("checked"),
                self.config.tls.key_path.as_ref().expect("checked"),
                self.config.tls.ca_path.as_ref().expect("checked"),
            )
            .await
        } else {
            warn!("Connecting without mTLS - development mode only!");
            HubClient::connect(&self.config.agent.hub_address).await
        }
    }
}

// ============================================================================
// Windows Service Support
// ============================================================================

#[cfg(windows)]
mod windows_service_impl {
    use super::*;
    use std::ffi::OsString;
    use std::time::Duration;
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
    };

    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    define_windows_service!(ffi_service_main, service_main);

    /// Entry point for service mode
    pub fn run_as_service() -> windows_service::Result<()> {
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
        Ok(())
    }

    /// Service main function called by SCM
    fn service_main(arguments: Vec<OsString>) {
        if let Err(e) = run_service(arguments) {
            error!("Service error: {}", e);
        }
    }

    fn run_service(_arguments: Vec<OsString>) -> windows_service::Result<()> {
        // Create a channel to receive stop events
        let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

        // Register the service control handler
        let status_handle = service_control_handler::register(SERVICE_NAME, move |control_event| {
            match control_event {
                ServiceControl::Stop => {
                    let _ = shutdown_tx.send(());
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        })?;

        // Set service status to Running
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        // Create the tokio runtime on a background thread
        let service_thread = std::thread::spawn(move || {
            // Build the runtime
            let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");

            // Load configuration
            let config_path = get_service_config_path();
            let config = match AgentConfig::from_file(&config_path) {
                Ok(mut c) => {
                    if let Some(parent) = config_path.parent() {
                        c.resolve_paths(&parent.to_path_buf());
                    }
                    c
                }
                Err(e) => {
                    error!("Failed to load config: {}", e);
                    return;
                }
            };

            // Run the agent
            runtime.block_on(async {
                let mut agent = Agent::new(config);

                tokio::select! {
                    result = agent.run() => {
                        if let Err(e) = result {
                            error!("Agent error: {}", e);
                        }
                    }
                    _ = wait_for_shutdown(&shutdown_rx) => {
                        info!("Received shutdown signal");
                    }
                }
            });
        });

        // Wait for the service to stop
        let _ = service_thread.join();

        // Set service status to Stopped
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    }

    async fn wait_for_shutdown(rx: &std::sync::mpsc::Receiver<()>) {
        loop {
            if rx.try_recv().is_ok() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    fn get_service_config_path() -> PathBuf {
        // Default to Program Files installation path
        PathBuf::from(r"C:\Program Files\Optio\config.toml")
    }

    /// Install the Windows service
    pub fn install_service(config_path: &PathBuf) -> Result<()> {
        use std::process::Command;

        let exe_path = std::env::current_exe()?;

        // Build the command line with --service and --config flags
        let bin_path = format!(
            "\"{}\" --service --config \"{}\"",
            exe_path.display(),
            config_path.display()
        );

        info!("Installing service with: {}", bin_path);

        // Use sc.exe to create the service
        let output = Command::new("sc.exe")
            .args([
                "create",
                SERVICE_NAME,
                &format!("binPath= {}", bin_path),
                "start= auto",
                "DisplayName= Optio Agent",
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create service: {}", stderr);
        }

        // Set description
        let _ = Command::new("sc.exe")
            .args([
                "description",
                SERVICE_NAME,
                "Optio endpoint management agent for secure IT orchestration",
            ])
            .output();

        // Configure recovery (restart on failure)
        let _ = Command::new("sc.exe")
            .args([
                "failure",
                SERVICE_NAME,
                "reset= 86400",
                "actions= restart/5000/restart/10000/restart/30000",
            ])
            .output();

        info!("Service installed successfully");
        Ok(())
    }

    /// Uninstall the Windows service
    pub fn uninstall_service() -> Result<()> {
        use std::process::Command;

        // Stop the service first
        let _ = Command::new("sc.exe")
            .args(["stop", SERVICE_NAME])
            .output();

        // Wait a moment for it to stop
        std::thread::sleep(Duration::from_secs(2));

        // Delete the service
        let output = Command::new("sc.exe")
            .args(["delete", SERVICE_NAME])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to delete service: {}", stderr);
        }

        info!("Service uninstalled successfully");
        Ok(())
    }
}

#[cfg(not(windows))]
mod windows_service_impl {
    use super::*;

    pub fn run_as_service() -> Result<()> {
        anyhow::bail!("Windows service mode is only available on Windows")
    }

    pub fn install_service(_config_path: &PathBuf) -> Result<()> {
        anyhow::bail!("Windows service installation is only available on Windows")
    }

    pub fn uninstall_service() -> Result<()> {
        anyhow::bail!("Windows service uninstallation is only available on Windows")
    }
}

// ============================================================================
// Entry Point
// ============================================================================

/// Set up a custom panic hook that logs to a file before crashing.
/// This allows us to debug remote failures by examining panic.log.
fn setup_panic_hook() {
    let default_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // Try to write to panic.log
        let panic_log_path = if cfg!(windows) {
            std::path::PathBuf::from(r"C:\Program Files\Optio\panic.log")
        } else {
            std::path::PathBuf::from("/var/log/optio/panic.log")
        };

        // Ensure directory exists
        if let Some(parent) = panic_log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Format the panic message
        let timestamp = chrono::Utc::now().to_rfc3339();
        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "unknown".to_string()
        };

        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };

        let panic_message = format!(
            "[{}] PANIC at {}\n  Message: {}\n  Agent Version: {}\n  OS: {} {}\n\n",
            timestamp,
            location,
            payload,
            AGENT_VERSION,
            std::env::consts::OS,
            std::env::consts::ARCH,
        );

        // Append to panic log
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&panic_log_path)
        {
            use std::io::Write;
            let _ = file.write_all(panic_message.as_bytes());
            let _ = file.sync_all();
        }

        // Also log via tracing if available
        tracing::error!("PANIC: {} at {}", payload, location);

        // Call the default hook to print to stderr
        default_hook(panic_info);
    }));
}

fn main() -> Result<()> {
    // Set up panic hook FIRST, before anything else
    setup_panic_hook();

    let args = Args::parse();

    // Handle service mode (called by Windows SCM)
    if args.service {
        // Initialize minimal logging for service mode
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new("optio_agent=info"))
            .with(tracing_subscriber::fmt::layer())
            .init();

        return windows_service_impl::run_as_service()
            .map_err(|e| anyhow::anyhow!("Service error: {}", e));
    }

    // Initialize tracing for console/interactive modes
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "optio_agent=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Handle install command
    if args.install {
        let config_path = args.config.canonicalize()
            .unwrap_or_else(|_| args.config.clone());
        return windows_service_impl::install_service(&config_path);
    }

    // Handle uninstall command
    if args.uninstall {
        return windows_service_impl::uninstall_service();
    }

    // Handle status command
    if args.status {
        println!("Optio Agent v{}", AGENT_VERSION);
        println!("Config file: {:?}", args.config);

        match AgentConfig::from_file(&args.config) {
            Ok(config) => {
                println!("Hub address: {}", config.agent.hub_address);
                println!("Heartbeat interval: {}s", config.agent.heartbeat_interval);
                println!("mTLS enabled: {}", config.has_mtls());
                if let Some(ref id) = config.agent.id {
                    println!("Agent ID: {}", id);
                }
            }
            Err(e) => {
                println!("Failed to load config: {}", e);
            }
        }
        return Ok(());
    }

    // Console mode (default)
    info!("Running in console mode");

    // Load configuration
    let mut config = AgentConfig::from_file(&args.config)?;

    // Resolve relative paths
    if let Some(parent) = args.config.parent() {
        config.resolve_paths(&parent.to_path_buf());
    }

    info!("Hub address: {}", config.agent.hub_address);
    info!("Heartbeat interval: {}s", config.agent.heartbeat_interval);
    info!("Command timeout: {}s", config.agent.command_timeout);
    info!("mTLS enabled: {}", config.has_mtls());

    // Create runtime and run agent
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let mut agent = Agent::new(config);
        agent.run().await
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_collector() {
        let mut collector = SystemCollector::new();
        let heartbeat = collector.build_heartbeat();

        assert!(!heartbeat.machine_id.is_empty());
        assert!(!heartbeat.hostname.is_empty());
        assert!(!heartbeat.os_info.is_empty());
        assert!(heartbeat.cpu_usage >= 0.0);
        assert!(heartbeat.ram_usage >= 0.0);
        assert!(heartbeat.ram_total > 0);
    }

    #[test]
    fn test_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.agent.hub_address, "http://127.0.0.1:50051");
        assert_eq!(config.agent.heartbeat_interval, DEFAULT_HEARTBEAT_INTERVAL);
        assert!(!config.has_mtls());
    }

    #[test]
    fn test_config_parse() {
        let toml_content = r#"
[agent]
id = "test-agent-123"
hub_address = "https://192.168.1.100:50051"
heartbeat_interval = 60

[tls]
ca_path = "certs/ca.pem"
cert_path = "certs/client.pem"
key_path = "certs/client.key"
"#;

        let config: AgentConfig = toml::from_str(toml_content).expect("parse config");
        assert_eq!(config.agent.id, Some("test-agent-123".to_string()));
        assert_eq!(config.agent.hub_address, "https://192.168.1.100:50051");
        assert_eq!(config.agent.heartbeat_interval, 60);
        assert!(config.has_mtls());
    }
}
