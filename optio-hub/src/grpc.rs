//! gRPC server for receiving scan results and heartbeats from agents.
//!
//! This module implements the Collector service with mTLS support for secure
//! communication with Optio agents.

use crate::db::Database;
use optio_core::proto::collector_server::{Collector, CollectorServer};
use optio_core::proto::{
    HeartbeatRequest, HeartbeatResponse, PendingCommand, ScanResult, SubmitScanResultResponse,
};
use optio_core::security::{HubTlsConfig, SecurityError};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tonic::{Request, Response, Status};

/// Shared state for the gRPC service
pub struct ServiceState {
    pub db: Arc<Database>,
}

/// Collector service implementation with database integration
pub struct CollectorService {
    state: Arc<ServiceState>,
}

impl CollectorService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            state: Arc::new(ServiceState { db }),
        }
    }
}

#[tonic::async_trait]
impl Collector for CollectorService {
    async fn submit_scan_result(
        &self,
        request: Request<ScanResult>,
    ) -> Result<Response<SubmitScanResultResponse>, Status> {
        let scan = request.into_inner();

        tracing::info!(
            machine_id = %scan.machine_id,
            scan_id = %scan.scan_id,
            target = %scan.target,
            status = %scan.status,
            "Received scan result"
        );

        // TODO: Store scan result in database

        let response = SubmitScanResultResponse {
            accepted: true,
            message: "Scan result received".to_string(),
        };

        Ok(Response::new(response))
    }

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        let heartbeat = request.into_inner();

        tracing::info!(
            machine_id = %heartbeat.machine_id,
            hostname = %heartbeat.hostname,
            cpu_usage = %heartbeat.cpu_usage,
            ram_usage = %heartbeat.ram_usage,
            "Received heartbeat from agent"
        );

        // Upsert agent in database
        if let Err(e) = upsert_agent(&self.state.db, &heartbeat) {
            tracing::error!("Failed to upsert agent: {}", e);
            return Err(Status::internal(format!("Database error: {}", e)));
        }

        // Build response with any pending commands
        let pending_commands = get_pending_commands(&self.state.db, &heartbeat.machine_id);

        let response = HeartbeatResponse {
            accepted: true,
            message: "Heartbeat received".to_string(),
            next_heartbeat_seconds: 30, // Configurable interval
            pending_commands,
        };

        Ok(Response::new(response))
    }
}

/// Insert or update an agent record based on heartbeat data
fn upsert_agent(db: &Database, heartbeat: &HeartbeatRequest) -> Result<(), rusqlite::Error> {
    let conn = db.conn.lock().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        ))
    })?;

    let now = chrono::Utc::now().to_rfc3339();

    // Check if agent exists
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM agents WHERE machine_id = ?1)",
            [&heartbeat.machine_id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if exists {
        // Update existing agent
        conn.execute(
            r#"
            UPDATE agents SET
                hostname = ?2,
                os_info = ?3,
                cpu_usage = ?4,
                ram_usage = ?5,
                ram_total = ?6,
                disk_free = ?7,
                disk_total = ?8,
                uptime_seconds = ?9,
                ip_addresses = ?10,
                agent_version = ?11,
                last_seen = ?12,
                status = 'online'
            WHERE machine_id = ?1
            "#,
            rusqlite::params![
                heartbeat.machine_id,
                heartbeat.hostname,
                heartbeat.os_info,
                heartbeat.cpu_usage,
                heartbeat.ram_usage,
                heartbeat.ram_total as i64,
                heartbeat.disk_free as i64,
                heartbeat.disk_total as i64,
                heartbeat.uptime_seconds as i64,
                heartbeat.ip_addresses,
                heartbeat.agent_version,
                now,
            ],
        )?;
        tracing::debug!("Updated agent: {}", heartbeat.machine_id);
    } else {
        // Insert new agent
        conn.execute(
            r#"
            INSERT INTO agents (
                machine_id, hostname, os_info, cpu_usage, ram_usage,
                ram_total, disk_free, disk_total, uptime_seconds,
                ip_addresses, agent_version, first_seen, last_seen, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12, 'online')
            "#,
            rusqlite::params![
                heartbeat.machine_id,
                heartbeat.hostname,
                heartbeat.os_info,
                heartbeat.cpu_usage,
                heartbeat.ram_usage,
                heartbeat.ram_total as i64,
                heartbeat.disk_free as i64,
                heartbeat.disk_total as i64,
                heartbeat.uptime_seconds as i64,
                heartbeat.ip_addresses,
                heartbeat.agent_version,
                now,
            ],
        )?;
        tracing::info!("Registered new agent: {} ({})", heartbeat.machine_id, heartbeat.hostname);
    }

    // Record telemetry history for time-series metrics
    let disk_percent = if heartbeat.disk_total > 0 {
        Some((1.0 - (heartbeat.disk_free as f32 / heartbeat.disk_total as f32)) * 100.0)
    } else {
        None
    };

    conn.execute(
        "INSERT INTO telemetry_history (agent_id, cpu_percent, ram_percent, disk_percent, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            heartbeat.machine_id,
            heartbeat.cpu_usage,
            heartbeat.ram_usage,
            disk_percent,
            now,
        ],
    )?;

    Ok(())
}

/// Get pending commands for an agent
fn get_pending_commands(db: &Database, machine_id: &str) -> Vec<PendingCommand> {
    let conn = match db.conn.lock() {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut stmt = match conn.prepare(
        r#"
        SELECT command_id, command_type, payload_json, priority, timeout_seconds
        FROM pending_commands
        WHERE machine_id = ?1 AND status = 'pending'
        ORDER BY priority DESC, created_at ASC
        LIMIT 10
        "#,
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let commands: Vec<PendingCommand> = stmt
        .query_map([machine_id], |row| {
            Ok(PendingCommand {
                command_id: row.get(0)?,
                command_type: row.get(1)?,
                payload_json: row.get(2)?,
                priority: row.get(3)?,
                timeout_seconds: row.get(4)?,
            })
        })
        .ok()
        .map(|iter| iter.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    // Mark commands as dispatched
    for cmd in &commands {
        let _ = conn.execute(
            "UPDATE pending_commands SET status = 'dispatched', dispatched_at = ?2 WHERE command_id = ?1",
            rusqlite::params![cmd.command_id, chrono::Utc::now().to_rfc3339()],
        );
    }

    commands
}

/// Configuration for the gRPC server
pub struct GrpcServerConfig {
    /// Socket address to bind to
    pub addr: SocketAddr,
    /// Path to server certificate
    pub cert_path: Option<PathBuf>,
    /// Path to server private key
    pub key_path: Option<PathBuf>,
    /// Path to CA certificate for client verification
    pub ca_path: Option<PathBuf>,
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0:50051".parse().expect("valid address"),
            cert_path: None,
            key_path: None,
            ca_path: None,
        }
    }
}

impl GrpcServerConfig {
    /// Check if TLS is configured
    pub fn has_tls(&self) -> bool {
        self.cert_path.is_some() && self.key_path.is_some() && self.ca_path.is_some()
    }
}

/// Start the gRPC server without TLS (for development)
pub async fn start_grpc_server(addr: SocketAddr) -> Result<(), tonic::transport::Error> {
    tracing::warn!("Starting gRPC server WITHOUT TLS - for development only!");

    tonic::transport::Server::builder()
        .add_service(CollectorServer::new(CollectorService::new(Arc::new(
            Database::open(&PathBuf::from(":memory:")).expect("in-memory db"),
        ))))
        .serve(addr)
        .await
}

/// Start the gRPC server with graceful shutdown support
///
/// The shutdown signal will be triggered when the provided future completes.
pub async fn start_grpc_server_with_shutdown(
    addr: SocketAddr,
    shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
) -> Result<(), tonic::transport::Error> {
    tracing::warn!("Starting gRPC server WITHOUT TLS - for development only!");
    tracing::info!("gRPC server listening on {} (graceful shutdown enabled)", addr);

    tonic::transport::Server::builder()
        .add_service(CollectorServer::new(CollectorService::new(Arc::new(
            Database::open(&PathBuf::from(":memory:")).expect("in-memory db"),
        ))))
        .serve_with_shutdown(addr, shutdown_signal)
        .await
}

/// Start the gRPC server with a shared database connection
pub async fn start_grpc_server_with_db(
    addr: SocketAddr,
    db: Arc<Database>,
) -> Result<(), tonic::transport::Error> {
    tracing::warn!("Starting gRPC server WITHOUT TLS - for development only!");

    tonic::transport::Server::builder()
        .add_service(CollectorServer::new(CollectorService::new(db)))
        .serve(addr)
        .await
}

/// Start the gRPC server with mTLS enabled
pub async fn start_grpc_server_mtls(
    config: GrpcServerConfig,
    db: Arc<Database>,
) -> Result<(), GrpcServerError> {
    if !config.has_tls() {
        return Err(GrpcServerError::MissingTlsConfig);
    }

    let cert_path = config.cert_path.as_ref().expect("checked above");
    let key_path = config.key_path.as_ref().expect("checked above");
    let ca_path = config.ca_path.as_ref().expect("checked above");

    // Load TLS configuration
    let tls_config = HubTlsConfig::new(cert_path, key_path, ca_path)?;
    let tls_acceptor = tls_config.acceptor();

    tracing::info!("Starting gRPC server with mTLS on {}", config.addr);

    // Create TCP listener
    let listener = TcpListener::bind(config.addr).await?;

    // Create the gRPC service
    let service = CollectorService::new(db);
    let grpc_service = CollectorServer::new(service);

    // Accept connections with TLS
    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let tls_acceptor = tls_acceptor.clone();
        let grpc_service = grpc_service.clone();

        tokio::spawn(async move {
            // Perform TLS handshake
            let tls_stream = match tls_acceptor.accept(stream).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("TLS handshake failed from {}: {}", peer_addr, e);
                    return;
                }
            };

            tracing::debug!("TLS connection established from {}", peer_addr);

            // Handle gRPC over TLS
            // Note: In production, use tonic's native TLS support or hyper integration
            // This is a simplified implementation for demonstration
            if let Err(e) = handle_grpc_connection(tls_stream, grpc_service).await {
                tracing::error!("gRPC connection error from {}: {}", peer_addr, e);
            }
        });
    }
}

async fn handle_grpc_connection<S, B>(
    _tls_stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
    _service: S,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: tower::Service<http::Request<B>> + Clone + Send + 'static,
{
    // This is a placeholder - in production, integrate with tonic's transport layer
    // For now, use tonic's native TLS configuration
    Ok(())
}

/// Errors that can occur during gRPC server operation
#[derive(Debug, thiserror::Error)]
pub enum GrpcServerError {
    #[error("TLS configuration is incomplete")]
    MissingTlsConfig,

    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
}

/// Alternative: Start gRPC server using tonic's native TLS support
pub async fn start_grpc_server_tonic_tls(
    config: GrpcServerConfig,
    db: Arc<Database>,
) -> Result<(), GrpcServerError> {
    use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};

    if !config.has_tls() {
        return Err(GrpcServerError::MissingTlsConfig);
    }

    let cert_path = config.cert_path.as_ref().expect("checked above");
    let key_path = config.key_path.as_ref().expect("checked above");
    let ca_path = config.ca_path.as_ref().expect("checked above");

    // Read certificate files
    let cert = std::fs::read_to_string(cert_path)
        .map_err(|e| GrpcServerError::Io(e))?;
    let key = std::fs::read_to_string(key_path)
        .map_err(|e| GrpcServerError::Io(e))?;
    let ca_cert = std::fs::read_to_string(ca_path)
        .map_err(|e| GrpcServerError::Io(e))?;

    // Create TLS identity and config
    let identity = Identity::from_pem(&cert, &key);
    let client_ca_cert = Certificate::from_pem(&ca_cert);

    let tls_config = ServerTlsConfig::new()
        .identity(identity)
        .client_ca_root(client_ca_cert);

    tracing::info!("Starting gRPC server with mTLS (tonic) on {}", config.addr);

    let service = CollectorService::new(db);

    Server::builder()
        .tls_config(tls_config)
        .map_err(|e| GrpcServerError::Transport(e))?
        .add_service(CollectorServer::new(service))
        .serve(config.addr)
        .await
        .map_err(|e| GrpcServerError::Transport(e))
}
