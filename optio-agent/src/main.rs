//! Optio Agent - Lightweight endpoint sentinel
//!
//! The agent runs as a headless service that:
//! - Sends periodic heartbeats to the Hub with system metrics
//! - Receives and executes commands from the Hub
//! - Reports command results back to the Hub
//!
//! Communication is outbound-only (agent "phones home") using mTLS.

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use optio_core::proto::collector_client::CollectorClient;
use optio_core::proto::{ExecuteScriptResponse, HeartbeatRequest, PendingCommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::System;
use thiserror::Error;
use tokio::process::Command;
use tokio::time::{interval, timeout};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
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
}

// ============================================================================
// Agent Configuration
// ============================================================================

/// Agent configuration loaded from environment or config file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Hub server address (e.g., "https://192.168.1.100:50051")
    pub hub_address: String,

    /// Path to client certificate PEM
    pub cert_path: Option<PathBuf>,

    /// Path to client private key PEM
    pub key_path: Option<PathBuf>,

    /// Path to CA certificate PEM
    pub ca_path: Option<PathBuf>,

    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,

    /// Command execution timeout in seconds
    pub command_timeout: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            hub_address: "http://127.0.0.1:50051".to_string(),
            cert_path: None,
            key_path: None,
            ca_path: None,
            heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL,
            command_timeout: DEFAULT_COMMAND_TIMEOUT,
        }
    }
}

impl AgentConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            hub_address: std::env::var("OPTIO_HUB_ADDRESS")
                .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string()),
            cert_path: std::env::var("OPTIO_CERT_PATH").ok().map(PathBuf::from),
            key_path: std::env::var("OPTIO_KEY_PATH").ok().map(PathBuf::from),
            ca_path: std::env::var("OPTIO_CA_PATH").ok().map(PathBuf::from),
            heartbeat_interval: std::env::var("OPTIO_HEARTBEAT_INTERVAL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_HEARTBEAT_INTERVAL),
            command_timeout: std::env::var("OPTIO_COMMAND_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_COMMAND_TIMEOUT),
        }
    }

    /// Check if mTLS is configured
    pub fn has_mtls(&self) -> bool {
        self.cert_path.is_some() && self.key_path.is_some() && self.ca_path.is_some()
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
        // In production, iterate over network interfaces
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

// Simple local IP detection - fallback if local_ip_address crate is not available
mod local_ip_address {
    use std::net::IpAddr;

    pub fn local_ip() -> Result<IpAddr, std::io::Error> {
        // Try to connect to a public DNS and get our local IP
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
        let collector = SystemCollector::new();
        let executor = CommandExecutor::new(
            collector.machine_id().to_string(),
            config.command_timeout,
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

        // Connect to hub
        let mut client = self.connect().await?;

        // Main heartbeat loop
        let mut heartbeat_interval = interval(Duration::from_secs(self.config.heartbeat_interval));

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
                &self.config.hub_address,
                self.config.cert_path.as_ref().expect("checked"),
                self.config.key_path.as_ref().expect("checked"),
                self.config.ca_path.as_ref().expect("checked"),
            )
            .await
        } else {
            warn!("Connecting without mTLS - development mode only!");
            HubClient::connect(&self.config.hub_address).await
        }
    }
}

// ============================================================================
// Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "optio_agent=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AgentConfig::from_env();
    info!("Hub address: {}", config.hub_address);
    info!("Heartbeat interval: {}s", config.heartbeat_interval);
    info!("Command timeout: {}s", config.command_timeout);
    info!("mTLS enabled: {}", config.has_mtls());

    // Create and run agent
    let mut agent = Agent::new(config);
    agent.run().await
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
    fn test_config_from_env() {
        // Test default config
        let config = AgentConfig::default();
        assert_eq!(config.hub_address, "http://127.0.0.1:50051");
        assert_eq!(config.heartbeat_interval, DEFAULT_HEARTBEAT_INTERVAL);
        assert!(!config.has_mtls());
    }
}
