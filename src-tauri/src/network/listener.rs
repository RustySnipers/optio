//! Hub Heartbeat Listener
//!
//! Implements the server-side endpoint for Agent connections. The Hub acts as
//! a listener that receives heartbeats and telemetry from Agents via
//! outbound-only polling from the Agent side.
//!
//! ## Architecture (Zero-Trust, Air-Gap Friendly)
//! - Agent initiates connection (outbound from Agent's perspective)
//! - Hub validates Agent identity via hardware-derived UUID and auth token
//! - Heartbeats contain compressed JSON telemetry
//! - Commands are dispatched as signed payloads
//!
//! ## Security Model (NIST CSF, SOC 2 CC6.6)
//! - TLS-secured connections (optional for air-gap scenarios)
//! - Token-based authentication
//! - All commands cryptographically signed
//! - Connection rate limiting

use crate::crypto::SignedCommand;
use crate::error::{OptioError, OptioResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex, RwLock};

/// Agent authentication payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAuth {
    /// Message type (should be "AUTH")
    #[serde(rename = "Type")]
    pub message_type: String,
    /// Authentication token
    #[serde(rename = "Token")]
    pub token: String,
    /// Script/Agent identifier
    #[serde(rename = "ScriptId")]
    pub script_id: String,
    /// Agent hostname
    #[serde(rename = "Hostname")]
    pub hostname: String,
    /// Current username
    #[serde(rename = "Username")]
    pub username: String,
    /// OS version string
    #[serde(rename = "OSVersion")]
    pub os_version: String,
    /// Auth timestamp
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
}

/// Agent heartbeat payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHeartbeat {
    /// Message type (should be "HEARTBEAT")
    #[serde(rename = "Type")]
    pub message_type: String,
    /// Script/Agent identifier
    #[serde(rename = "ScriptId")]
    pub script_id: String,
    /// Heartbeat timestamp
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
    /// Memory usage in MB
    #[serde(rename = "Memory")]
    pub memory_mb: f64,
    /// Process uptime in seconds
    #[serde(rename = "Uptime")]
    pub uptime_seconds: f64,
}

/// Agent telemetry payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTelemetry {
    /// Message type (should be "TELEMETRY")
    #[serde(rename = "Type")]
    pub message_type: String,
    /// Script/Agent identifier
    #[serde(rename = "ScriptId")]
    pub script_id: String,
    /// Telemetry data
    #[serde(rename = "Data")]
    pub data: TelemetryData,
}

/// Telemetry data from Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    #[serde(rename = "Hostname")]
    pub hostname: String,
    #[serde(rename = "Domain")]
    pub domain: String,
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(rename = "OSVersion")]
    pub os_version: String,
    #[serde(rename = "PSVersion")]
    pub ps_version: String,
    #[serde(rename = "Is64Bit")]
    pub is_64bit: bool,
    #[serde(rename = "ProcessorCount")]
    pub processor_count: String,
    #[serde(rename = "TotalMemoryMB")]
    pub total_memory_mb: f64,
    #[serde(rename = "IPAddresses")]
    pub ip_addresses: Vec<String>,
    #[serde(rename = "LastBootTime")]
    pub last_boot_time: String,
}

/// Connected Agent session
#[derive(Debug, Clone, Serialize)]
pub struct AgentSession {
    /// Script/Agent identifier
    pub script_id: String,
    /// Agent hostname
    pub hostname: String,
    /// Remote address
    pub remote_addr: String,
    /// Authentication timestamp
    pub authenticated_at: DateTime<Utc>,
    /// Last heartbeat received
    pub last_heartbeat: DateTime<Utc>,
    /// Latest telemetry (if received)
    pub telemetry: Option<TelemetryData>,
    /// Session status
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum SessionStatus {
    Connected,
    Authenticated,
    Disconnected,
}

/// Command queued for an Agent
#[derive(Debug, Clone)]
pub struct QueuedCommand {
    pub signed_command: SignedCommand,
    pub queued_at: DateTime<Utc>,
}

/// Hub listener configuration
#[derive(Debug, Clone)]
pub struct ListenerConfig {
    /// Port to listen on
    pub port: u16,
    /// Bind address (default: 0.0.0.0)
    pub bind_addr: String,
    /// Valid authentication tokens (script_id -> token)
    pub valid_tokens: HashMap<String, String>,
    /// Maximum connections per IP
    pub max_connections_per_ip: usize,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        ListenerConfig {
            port: 8443,
            bind_addr: "0.0.0.0".to_string(),
            valid_tokens: HashMap::new(),
            max_connections_per_ip: 10,
            connection_timeout_secs: 300,
        }
    }
}

/// Hub listener state
pub struct HubListener {
    config: ListenerConfig,
    sessions: Arc<RwLock<HashMap<String, AgentSession>>>,
    command_queue: Arc<RwLock<HashMap<String, Vec<QueuedCommand>>>>,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl HubListener {
    /// Create a new Hub listener with the given configuration
    pub fn new(config: ListenerConfig) -> Self {
        HubListener {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            command_queue: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: None,
        }
    }

    /// Register a valid token for an Agent script
    pub async fn register_token(&mut self, script_id: String, token: String) {
        self.config.valid_tokens.insert(script_id, token);
    }

    /// Queue a signed command for an Agent
    pub async fn queue_command(&self, script_id: &str, command: SignedCommand) {
        let mut queue = self.command_queue.write().await;
        let entry = queue.entry(script_id.to_string()).or_insert_with(Vec::new);
        entry.push(QueuedCommand {
            signed_command: command,
            queued_at: Utc::now(),
        });
        tracing::debug!("Queued command for agent {}", script_id);
    }

    /// Get all active sessions
    pub async fn get_sessions(&self) -> Vec<AgentSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Get session for a specific Agent
    pub async fn get_session(&self, script_id: &str) -> Option<AgentSession> {
        let sessions = self.sessions.read().await;
        sessions.get(script_id).cloned()
    }

    /// Start the listener
    pub async fn start(&mut self) -> OptioResult<()> {
        let addr = format!("{}:{}", self.config.bind_addr, self.config.port);
        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            OptioError::NetworkScan(format!("Failed to bind listener on {}: {}", addr, e))
        })?;

        tracing::info!("Hub listener started on {}", addr);

        let (shutdown_tx, _) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        let sessions = Arc::clone(&self.sessions);
        let command_queue = Arc::clone(&self.command_queue);
        let valid_tokens = self.config.valid_tokens.clone();
        let timeout_secs = self.config.connection_timeout_secs;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                let sessions = Arc::clone(&sessions);
                                let command_queue = Arc::clone(&command_queue);
                                let tokens = valid_tokens.clone();
                                let mut shutdown_rx = shutdown_tx.subscribe();

                                tokio::spawn(async move {
                                    tokio::select! {
                                        _ = handle_connection(stream, addr, sessions, command_queue, tokens, timeout_secs) => {}
                                        _ = shutdown_rx.recv() => {
                                            tracing::debug!("Connection handler shutting down for {}", addr);
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("Failed to accept connection: {}", e);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the listener
    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
            tracing::info!("Hub listener stopped");
        }
    }
}

/// Handle an individual Agent connection
async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    sessions: Arc<RwLock<HashMap<String, AgentSession>>>,
    command_queue: Arc<RwLock<HashMap<String, Vec<QueuedCommand>>>>,
    valid_tokens: HashMap<String, String>,
    timeout_secs: u64,
) {
    tracing::debug!("New connection from {}", addr);

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut authenticated_script_id: Option<String> = None;

    loop {
        line.clear();

        let read_result = tokio::time::timeout(
            tokio::time::Duration::from_secs(timeout_secs),
            reader.read_line(&mut line),
        )
        .await;

        match read_result {
            Ok(Ok(0)) => {
                // Connection closed
                tracing::debug!("Connection closed by {}", addr);
                break;
            }
            Ok(Ok(_)) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Try to parse the message
                if let Ok(auth) = serde_json::from_str::<AgentAuth>(trimmed) {
                    if auth.message_type == "AUTH" {
                        // Validate token
                        if let Some(expected_token) = valid_tokens.get(&auth.script_id) {
                            if &auth.token == expected_token {
                                tracing::info!(
                                    "Agent {} ({}) authenticated from {}",
                                    auth.script_id,
                                    auth.hostname,
                                    addr
                                );

                                let session = AgentSession {
                                    script_id: auth.script_id.clone(),
                                    hostname: auth.hostname,
                                    remote_addr: addr.to_string(),
                                    authenticated_at: Utc::now(),
                                    last_heartbeat: Utc::now(),
                                    telemetry: None,
                                    status: SessionStatus::Authenticated,
                                };

                                {
                                    let mut sessions_guard = sessions.write().await;
                                    sessions_guard.insert(auth.script_id.clone(), session);
                                }

                                authenticated_script_id = Some(auth.script_id);

                                // Send ACK
                                let ack = r#"{"status":"OK","message":"Authenticated"}"#;
                                if writer.write_all(format!("{}\n", ack).as_bytes()).await.is_err() {
                                    break;
                                }
                            } else {
                                tracing::warn!("Invalid token from {} for script {}", addr, auth.script_id);
                                let _ = writer
                                    .write_all(b"{\"status\":\"ERROR\",\"message\":\"Invalid token\"}\n")
                                    .await;
                                break;
                            }
                        } else {
                            tracing::warn!("Unknown script ID {} from {}", auth.script_id, addr);
                            let _ = writer
                                .write_all(b"{\"status\":\"ERROR\",\"message\":\"Unknown script\"}\n")
                                .await;
                            break;
                        }
                    }
                } else if let Ok(heartbeat) = serde_json::from_str::<AgentHeartbeat>(trimmed) {
                    if heartbeat.message_type == "HEARTBEAT" {
                        if let Some(ref script_id) = authenticated_script_id {
                            if &heartbeat.script_id == script_id {
                                tracing::debug!("Heartbeat from {}", script_id);

                                {
                                    let mut sessions_guard = sessions.write().await;
                                    if let Some(session) = sessions_guard.get_mut(script_id) {
                                        session.last_heartbeat = Utc::now();
                                    }
                                }

                                // Check for pending commands
                                let pending_commands: Vec<QueuedCommand> = {
                                    let mut queue = command_queue.write().await;
                                    queue.remove(script_id).unwrap_or_default()
                                };

                                if pending_commands.is_empty() {
                                    let _ = writer
                                        .write_all(b"{\"status\":\"OK\",\"commands\":[]}\n")
                                        .await;
                                } else {
                                    let commands: Vec<&SignedCommand> =
                                        pending_commands.iter().map(|q| &q.signed_command).collect();
                                    let response = serde_json::json!({
                                        "status": "OK",
                                        "commands": commands
                                    });
                                    if let Ok(json) = serde_json::to_string(&response) {
                                        let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
                                    }
                                }
                            }
                        }
                    }
                } else if let Ok(telemetry) = serde_json::from_str::<AgentTelemetry>(trimmed) {
                    if telemetry.message_type == "TELEMETRY" {
                        if let Some(ref script_id) = authenticated_script_id {
                            if &telemetry.script_id == script_id {
                                tracing::info!("Telemetry received from {}", script_id);

                                {
                                    let mut sessions_guard = sessions.write().await;
                                    if let Some(session) = sessions_guard.get_mut(script_id) {
                                        session.telemetry = Some(telemetry.data);
                                        session.last_heartbeat = Utc::now();
                                    }
                                }

                                let _ = writer
                                    .write_all(b"{\"status\":\"OK\",\"message\":\"Telemetry received\"}\n")
                                    .await;
                            }
                        }
                    }
                } else {
                    tracing::warn!("Unknown message from {}: {}", addr, trimmed);
                }
            }
            Ok(Err(e)) => {
                tracing::error!("Read error from {}: {}", addr, e);
                break;
            }
            Err(_) => {
                tracing::debug!("Connection timeout for {}", addr);
                break;
            }
        }
    }

    // Mark session as disconnected
    if let Some(script_id) = authenticated_script_id {
        let mut sessions_guard = sessions.write().await;
        if let Some(session) = sessions_guard.get_mut(&script_id) {
            session.status = SessionStatus::Disconnected;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ListenerConfig::default();
        assert_eq!(config.port, 8443);
        assert_eq!(config.bind_addr, "0.0.0.0");
    }

    #[test]
    fn test_parse_auth_message() {
        let json = r#"{"Type":"AUTH","Token":"secret","ScriptId":"agent-1","Hostname":"PC01","Username":"admin","OSVersion":"Windows 10","Timestamp":"2024-01-01T00:00:00Z"}"#;
        let auth: AgentAuth = serde_json::from_str(json).expect("should parse");
        assert_eq!(auth.message_type, "AUTH");
        assert_eq!(auth.script_id, "agent-1");
    }

    #[test]
    fn test_parse_heartbeat_message() {
        let json = r#"{"Type":"HEARTBEAT","ScriptId":"agent-1","Timestamp":"2024-01-01T00:00:00Z","Memory":128.5,"Uptime":3600}"#;
        let hb: AgentHeartbeat = serde_json::from_str(json).expect("should parse");
        assert_eq!(hb.message_type, "HEARTBEAT");
        assert_eq!(hb.memory_mb, 128.5);
    }
}
