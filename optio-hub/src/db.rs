//! Database layer with encrypted SQLite storage
//!
//! Provides secure, local-first storage for client profiles, audit logs,
//! and credentials using AES-256 encryption.

use crate::error::{OptioError, OptioResult};
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Thread-safe database connection wrapper
pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    /// Open or create the database at the specified path
    pub fn open(path: &PathBuf) -> OptioResult<Self> {
        let conn = Connection::open(path)?;

        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    /// Initialize database schema
    pub fn init_schema(&self) -> OptioResult<()> {
        let conn = self.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        conn.execute_batch(r#"
            -- Clients table
            CREATE TABLE IF NOT EXISTS clients (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                target_subnet TEXT,
                contact_email TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Generated scripts history
            CREATE TABLE IF NOT EXISTS script_history (
                id TEXT PRIMARY KEY,
                client_id TEXT NOT NULL,
                template_name TEXT NOT NULL,
                config_hash TEXT NOT NULL,
                generated_at TEXT NOT NULL,
                output_path TEXT,
                FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE CASCADE
            );

            -- Audit log for compliance tracking
            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                action TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT,
                details TEXT,
                user_ip TEXT
            );

            -- Encrypted credentials vault
            CREATE TABLE IF NOT EXISTS credentials_vault (
                id TEXT PRIMARY KEY,
                client_id TEXT NOT NULL,
                label TEXT NOT NULL,
                encrypted_data BLOB NOT NULL,
                nonce BLOB NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE CASCADE
            );

            -- Agents table for tracking connected optio-agent instances
            CREATE TABLE IF NOT EXISTS agents (
                machine_id TEXT PRIMARY KEY,
                hostname TEXT NOT NULL,
                os_info TEXT,
                cpu_usage REAL DEFAULT 0.0,
                ram_usage REAL DEFAULT 0.0,
                ram_total INTEGER DEFAULT 0,
                disk_free INTEGER DEFAULT 0,
                disk_total INTEGER DEFAULT 0,
                uptime_seconds INTEGER DEFAULT 0,
                ip_addresses TEXT,
                agent_version TEXT,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'unknown',
                client_id TEXT,
                tags TEXT,
                notes TEXT,
                FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE SET NULL
            );

            -- Pending commands queue for agents
            CREATE TABLE IF NOT EXISTS pending_commands (
                command_id TEXT PRIMARY KEY,
                machine_id TEXT NOT NULL,
                command_type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                priority INTEGER DEFAULT 0,
                timeout_seconds INTEGER DEFAULT 300,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL,
                dispatched_at TEXT,
                completed_at TEXT,
                result_json TEXT,
                error_message TEXT,
                FOREIGN KEY (machine_id) REFERENCES agents(machine_id) ON DELETE CASCADE
            );

            -- Command execution history
            CREATE TABLE IF NOT EXISTS command_history (
                id TEXT PRIMARY KEY,
                command_id TEXT NOT NULL,
                machine_id TEXT NOT NULL,
                command_type TEXT NOT NULL,
                success INTEGER NOT NULL DEFAULT 0,
                exit_code INTEGER,
                stdout TEXT,
                stderr TEXT,
                error_message TEXT,
                started_at TEXT,
                finished_at TEXT,
                duration_ms INTEGER,
                created_at TEXT NOT NULL
            );

            -- Telemetry history for time-series metrics
            CREATE TABLE IF NOT EXISTS telemetry_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                cpu_percent REAL NOT NULL,
                ram_percent REAL NOT NULL,
                disk_percent REAL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (agent_id) REFERENCES agents(machine_id) ON DELETE CASCADE
            );

            -- Action logs for comprehensive audit trail
            -- Tracks all actions performed on agents (scripts, terminal sessions, patches)
            CREATE TABLE IF NOT EXISTS action_logs (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                action_type TEXT NOT NULL,
                command_content TEXT,
                status TEXT NOT NULL DEFAULT 'PENDING',
                timestamp TEXT NOT NULL,
                user_initiated INTEGER NOT NULL DEFAULT 1,
                session_id TEXT,
                metadata_json TEXT,
                FOREIGN KEY (agent_id) REFERENCES agents(machine_id) ON DELETE CASCADE
            );

            -- Create indexes for performance
            CREATE INDEX IF NOT EXISTS idx_clients_name ON clients(name);
            CREATE INDEX IF NOT EXISTS idx_script_history_client ON script_history(client_id);
            CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);
            CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);
            CREATE INDEX IF NOT EXISTS idx_agents_last_seen ON agents(last_seen);
            CREATE INDEX IF NOT EXISTS idx_pending_commands_machine ON pending_commands(machine_id);
            CREATE INDEX IF NOT EXISTS idx_pending_commands_status ON pending_commands(status);
            CREATE INDEX IF NOT EXISTS idx_command_history_machine ON command_history(machine_id);
            CREATE INDEX IF NOT EXISTS idx_telemetry_agent ON telemetry_history(agent_id);
            CREATE INDEX IF NOT EXISTS idx_telemetry_timestamp ON telemetry_history(timestamp);
            CREATE INDEX IF NOT EXISTS idx_action_logs_agent ON action_logs(agent_id);
            CREATE INDEX IF NOT EXISTS idx_action_logs_timestamp ON action_logs(timestamp);
            CREATE INDEX IF NOT EXISTS idx_action_logs_action_type ON action_logs(action_type);
        "#)?;

        tracing::info!("Database schema initialized");
        Ok(())
    }
}

/// Client profile stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub target_subnet: Option<String>,
    pub contact_email: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Client {
    pub fn new(name: String, target_subnet: Option<String>, contact_email: Option<String>, notes: Option<String>) -> Self {
        let now = Utc::now();
        Client {
            id: Uuid::new_v4().to_string(),
            name,
            target_subnet,
            contact_email,
            notes,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Client repository for CRUD operations
pub struct ClientRepository<'a> {
    db: &'a Database,
}

impl<'a> ClientRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        ClientRepository { db }
    }

    pub fn create(&self, client: &Client) -> OptioResult<()> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        conn.execute(
            "INSERT INTO clients (id, name, target_subnet, contact_email, notes, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                client.id,
                client.name,
                client.target_subnet,
                client.contact_email,
                client.notes,
                client.created_at.to_rfc3339(),
                client.updated_at.to_rfc3339(),
            ],
        )?;

        tracing::debug!("Created client: {}", client.id);
        Ok(())
    }

    pub fn get(&self, id: &str) -> OptioResult<Option<Client>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, name, target_subnet, contact_email, notes, created_at, updated_at FROM clients WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Client {
                id: row.get(0)?,
                name: row.get(1)?,
                target_subnet: row.get(2)?,
                contact_email: row.get(3)?,
                notes: row.get(4)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map_err(|e| OptioError::Database(e.to_string()))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map_err(|e| OptioError::Database(e.to_string()))?
                    .with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list(&self) -> OptioResult<Vec<Client>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, name, target_subnet, contact_email, notes, created_at, updated_at FROM clients ORDER BY name"
        )?;

        let clients = stmt.query_map([], |row| {
            Ok(Client {
                id: row.get(0)?,
                name: row.get(1)?,
                target_subnet: row.get(2)?,
                contact_email: row.get(3)?,
                notes: row.get(4)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5).unwrap_or_default())
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6).unwrap_or_default())
                    .unwrap_or_default()
                    .with_timezone(&Utc),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(clients)
    }

    pub fn update(&self, client: &Client) -> OptioResult<bool> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let updated = conn.execute(
            "UPDATE clients SET name = ?2, target_subnet = ?3, contact_email = ?4, notes = ?5, updated_at = ?6 WHERE id = ?1",
            params![
                client.id,
                client.name,
                client.target_subnet,
                client.contact_email,
                client.notes,
                Utc::now().to_rfc3339(),
            ],
        )?;

        Ok(updated > 0)
    }

    pub fn delete(&self, id: &str) -> OptioResult<bool> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;
        let deleted = conn.execute("DELETE FROM clients WHERE id = ?1", params![id])?;
        Ok(deleted > 0)
    }
}

/// Initialize the database on application startup
pub async fn initialize(app_handle: &AppHandle) -> OptioResult<()> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| OptioError::Database(format!("Failed to get app data dir: {}", e)))?;

    // Ensure the directory exists
    std::fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join("optio.db");
    tracing::info!("Database path: {:?}", db_path);

    let db = Database::open(&db_path)?;
    db.init_schema()?;

    // Initialize GRC schema
    optio_core::grc::repository::init_grc_schema(&db.conn)
        .map_err(|e| OptioError::Database(e.to_string()))?;

    // Store database in app state
    app_handle.manage(db);

    Ok(())
}

// ============================================================================
// Agent Types and Repository
// ============================================================================

/// Agent status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Online,
    Offline,
    Unknown,
    Error,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Online => write!(f, "online"),
            AgentStatus::Offline => write!(f, "offline"),
            AgentStatus::Unknown => write!(f, "unknown"),
            AgentStatus::Error => write!(f, "error"),
        }
    }
}

impl From<&str> for AgentStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "online" => AgentStatus::Online,
            "offline" => AgentStatus::Offline,
            "error" => AgentStatus::Error,
            _ => AgentStatus::Unknown,
        }
    }
}

/// Agent information from heartbeat data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub machine_id: String,
    pub hostname: String,
    pub os_info: Option<String>,
    pub cpu_usage: f32,
    pub ram_usage: f32,
    pub ram_total: i64,
    pub disk_free: i64,
    pub disk_total: i64,
    pub uptime_seconds: i64,
    pub ip_addresses: Option<String>,
    pub agent_version: Option<String>,
    pub first_seen: String,
    pub last_seen: String,
    pub status: AgentStatus,
    pub client_id: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
}

impl Agent {
    /// Check if agent is considered stale (no heartbeat in 2 minutes)
    pub fn is_stale(&self) -> bool {
        if let Ok(last_seen) = DateTime::parse_from_rfc3339(&self.last_seen) {
            let now = Utc::now();
            let duration = now.signed_duration_since(last_seen);
            duration.num_seconds() > 120 // 2 minutes
        } else {
            true
        }
    }
}

/// Repository for agent CRUD operations
pub struct AgentRepository<'a> {
    db: &'a Database,
}

impl<'a> AgentRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        AgentRepository { db }
    }

    /// List all agents
    pub fn list(&self) -> OptioResult<Vec<Agent>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT machine_id, hostname, os_info, cpu_usage, ram_usage,
                   ram_total, disk_free, disk_total, uptime_seconds,
                   ip_addresses, agent_version, first_seen, last_seen,
                   status, client_id, tags, notes
            FROM agents
            ORDER BY last_seen DESC
            "#
        )?;

        let agents = stmt.query_map([], |row| {
            Ok(Agent {
                machine_id: row.get(0)?,
                hostname: row.get(1)?,
                os_info: row.get(2)?,
                cpu_usage: row.get(3)?,
                ram_usage: row.get(4)?,
                ram_total: row.get(5)?,
                disk_free: row.get(6)?,
                disk_total: row.get(7)?,
                uptime_seconds: row.get(8)?,
                ip_addresses: row.get(9)?,
                agent_version: row.get(10)?,
                first_seen: row.get(11)?,
                last_seen: row.get(12)?,
                status: AgentStatus::from(row.get::<_, String>(13)?.as_str()),
                client_id: row.get(14)?,
                tags: row.get(15)?,
                notes: row.get(16)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(agents)
    }

    /// Get a single agent by machine_id
    pub fn get(&self, machine_id: &str) -> OptioResult<Option<Agent>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT machine_id, hostname, os_info, cpu_usage, ram_usage,
                   ram_total, disk_free, disk_total, uptime_seconds,
                   ip_addresses, agent_version, first_seen, last_seen,
                   status, client_id, tags, notes
            FROM agents
            WHERE machine_id = ?1
            "#
        )?;

        let mut rows = stmt.query(params![machine_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Agent {
                machine_id: row.get(0)?,
                hostname: row.get(1)?,
                os_info: row.get(2)?,
                cpu_usage: row.get(3)?,
                ram_usage: row.get(4)?,
                ram_total: row.get(5)?,
                disk_free: row.get(6)?,
                disk_total: row.get(7)?,
                uptime_seconds: row.get(8)?,
                ip_addresses: row.get(9)?,
                agent_version: row.get(10)?,
                first_seen: row.get(11)?,
                last_seen: row.get(12)?,
                status: AgentStatus::from(row.get::<_, String>(13)?.as_str()),
                client_id: row.get(14)?,
                tags: row.get(15)?,
                notes: row.get(16)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List agents by status
    pub fn list_by_status(&self, status: AgentStatus) -> OptioResult<Vec<Agent>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT machine_id, hostname, os_info, cpu_usage, ram_usage,
                   ram_total, disk_free, disk_total, uptime_seconds,
                   ip_addresses, agent_version, first_seen, last_seen,
                   status, client_id, tags, notes
            FROM agents
            WHERE status = ?1
            ORDER BY last_seen DESC
            "#
        )?;

        let agents = stmt.query_map([status.to_string()], |row| {
            Ok(Agent {
                machine_id: row.get(0)?,
                hostname: row.get(1)?,
                os_info: row.get(2)?,
                cpu_usage: row.get(3)?,
                ram_usage: row.get(4)?,
                ram_total: row.get(5)?,
                disk_free: row.get(6)?,
                disk_total: row.get(7)?,
                uptime_seconds: row.get(8)?,
                ip_addresses: row.get(9)?,
                agent_version: row.get(10)?,
                first_seen: row.get(11)?,
                last_seen: row.get(12)?,
                status: AgentStatus::from(row.get::<_, String>(13)?.as_str()),
                client_id: row.get(14)?,
                tags: row.get(15)?,
                notes: row.get(16)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(agents)
    }

    /// Mark stale agents as offline (called periodically)
    pub fn mark_stale_agents_offline(&self) -> OptioResult<usize> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        // Mark agents offline if no heartbeat in 2 minutes
        let threshold = (Utc::now() - chrono::Duration::seconds(120)).to_rfc3339();

        let count = conn.execute(
            r#"
            UPDATE agents
            SET status = 'offline'
            WHERE status = 'online' AND last_seen < ?1
            "#,
            params![threshold],
        )?;

        if count > 0 {
            tracing::info!("Marked {} stale agents as offline", count);
        }

        Ok(count)
    }

    /// Delete an agent
    pub fn delete(&self, machine_id: &str) -> OptioResult<bool> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;
        let deleted = conn.execute("DELETE FROM agents WHERE machine_id = ?1", params![machine_id])?;
        Ok(deleted > 0)
    }

    /// Count agents by status
    pub fn count_by_status(&self) -> OptioResult<std::collections::HashMap<String, i64>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT status, COUNT(*) FROM agents GROUP BY status"
        )?;

        let mut counts = std::collections::HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows.flatten() {
            counts.insert(row.0, row.1);
        }

        Ok(counts)
    }
}

// ============================================================================
// Telemetry History Types and Repository
// ============================================================================

/// A single telemetry record for time-series storage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryRecord {
    pub id: i64,
    pub agent_id: String,
    pub cpu_percent: f32,
    pub ram_percent: f32,
    pub disk_percent: Option<f32>,
    pub timestamp: String,
}

/// Repository for telemetry history operations
pub struct TelemetryRepository<'a> {
    db: &'a Database,
}

impl<'a> TelemetryRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        TelemetryRepository { db }
    }

    /// Insert a new telemetry record
    pub fn insert(&self, agent_id: &str, cpu_percent: f32, ram_percent: f32, disk_percent: Option<f32>) -> OptioResult<()> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let timestamp = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO telemetry_history (agent_id, cpu_percent, ram_percent, disk_percent, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![agent_id, cpu_percent, ram_percent, disk_percent, timestamp],
        )?;

        Ok(())
    }

    /// Get telemetry history for an agent
    pub fn get_history(&self, agent_id: &str, limit: u32) -> OptioResult<Vec<TelemetryRecord>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, agent_id, cpu_percent, ram_percent, disk_percent, timestamp
            FROM telemetry_history
            WHERE agent_id = ?1
            ORDER BY timestamp DESC
            LIMIT ?2
            "#
        )?;

        let records = stmt.query_map(params![agent_id, limit], |row| {
            Ok(TelemetryRecord {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                cpu_percent: row.get(2)?,
                ram_percent: row.get(3)?,
                disk_percent: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(records)
    }

    /// Get telemetry history within a time range
    pub fn get_history_range(&self, agent_id: &str, start: &str, end: &str) -> OptioResult<Vec<TelemetryRecord>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, agent_id, cpu_percent, ram_percent, disk_percent, timestamp
            FROM telemetry_history
            WHERE agent_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3
            ORDER BY timestamp ASC
            "#
        )?;

        let records = stmt.query_map(params![agent_id, start, end], |row| {
            Ok(TelemetryRecord {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                cpu_percent: row.get(2)?,
                ram_percent: row.get(3)?,
                disk_percent: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(records)
    }

    /// Cleanup old telemetry records (keep last N days)
    pub fn cleanup_old_records(&self, days_to_keep: i64) -> OptioResult<usize> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let cutoff = (Utc::now() - chrono::Duration::days(days_to_keep)).to_rfc3339();

        let deleted = conn.execute(
            "DELETE FROM telemetry_history WHERE timestamp < ?1",
            params![cutoff],
        )?;

        if deleted > 0 {
            tracing::info!("Cleaned up {} old telemetry records", deleted);
        }

        Ok(deleted)
    }

    /// Get summary statistics for an agent
    pub fn get_stats(&self, agent_id: &str, hours: i64) -> OptioResult<TelemetryStats> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let cutoff = (Utc::now() - chrono::Duration::hours(hours)).to_rfc3339();

        let mut stmt = conn.prepare(
            r#"
            SELECT
                AVG(cpu_percent) as avg_cpu,
                MAX(cpu_percent) as max_cpu,
                MIN(cpu_percent) as min_cpu,
                AVG(ram_percent) as avg_ram,
                MAX(ram_percent) as max_ram,
                MIN(ram_percent) as min_ram,
                COUNT(*) as sample_count
            FROM telemetry_history
            WHERE agent_id = ?1 AND timestamp >= ?2
            "#
        )?;

        let stats = stmt.query_row(params![agent_id, cutoff], |row| {
            Ok(TelemetryStats {
                avg_cpu: row.get(0).unwrap_or(0.0),
                max_cpu: row.get(1).unwrap_or(0.0),
                min_cpu: row.get(2).unwrap_or(0.0),
                avg_ram: row.get(3).unwrap_or(0.0),
                max_ram: row.get(4).unwrap_or(0.0),
                min_ram: row.get(5).unwrap_or(0.0),
                sample_count: row.get(6).unwrap_or(0),
            })
        }).unwrap_or_default();

        Ok(stats)
    }
}

/// Telemetry statistics summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryStats {
    pub avg_cpu: f32,
    pub max_cpu: f32,
    pub min_cpu: f32,
    pub avg_ram: f32,
    pub max_ram: f32,
    pub min_ram: f32,
    pub sample_count: i64,
}

// ============================================================================
// Action Logs Types and Repository
// ============================================================================

/// Action type enum for categorizing logged actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionType {
    ScriptExec,
    TerminalSession,
    PatchInstall,
    ConfigChange,
    AgentRegistration,
    AgentDeletion,
    SystemCommand,
    FileTransfer,
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::ScriptExec => write!(f, "SCRIPT_EXEC"),
            ActionType::TerminalSession => write!(f, "TERMINAL_SESSION"),
            ActionType::PatchInstall => write!(f, "PATCH_INSTALL"),
            ActionType::ConfigChange => write!(f, "CONFIG_CHANGE"),
            ActionType::AgentRegistration => write!(f, "AGENT_REGISTRATION"),
            ActionType::AgentDeletion => write!(f, "AGENT_DELETION"),
            ActionType::SystemCommand => write!(f, "SYSTEM_COMMAND"),
            ActionType::FileTransfer => write!(f, "FILE_TRANSFER"),
        }
    }
}

impl From<&str> for ActionType {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "SCRIPT_EXEC" => ActionType::ScriptExec,
            "TERMINAL_SESSION" => ActionType::TerminalSession,
            "PATCH_INSTALL" => ActionType::PatchInstall,
            "CONFIG_CHANGE" => ActionType::ConfigChange,
            "AGENT_REGISTRATION" => ActionType::AgentRegistration,
            "AGENT_DELETION" => ActionType::AgentDeletion,
            "SYSTEM_COMMAND" => ActionType::SystemCommand,
            "FILE_TRANSFER" => ActionType::FileTransfer,
            _ => ActionType::SystemCommand,
        }
    }
}

/// Action status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionStatus {
    Success,
    Failure,
    Pending,
    InProgress,
    Cancelled,
}

impl std::fmt::Display for ActionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionStatus::Success => write!(f, "SUCCESS"),
            ActionStatus::Failure => write!(f, "FAILURE"),
            ActionStatus::Pending => write!(f, "PENDING"),
            ActionStatus::InProgress => write!(f, "IN_PROGRESS"),
            ActionStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

impl From<&str> for ActionStatus {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "SUCCESS" => ActionStatus::Success,
            "FAILURE" => ActionStatus::Failure,
            "PENDING" => ActionStatus::Pending,
            "IN_PROGRESS" => ActionStatus::InProgress,
            "CANCELLED" => ActionStatus::Cancelled,
            _ => ActionStatus::Pending,
        }
    }
}

/// Action log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionLog {
    pub id: String,
    pub agent_id: String,
    pub action_type: ActionType,
    pub command_content: Option<String>,
    pub status: ActionStatus,
    pub timestamp: String,
    pub user_initiated: bool,
    pub session_id: Option<String>,
    pub metadata_json: Option<String>,
}

impl ActionLog {
    /// Create a new action log entry
    pub fn new(
        agent_id: String,
        action_type: ActionType,
        command_content: Option<String>,
        user_initiated: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            action_type,
            command_content,
            status: ActionStatus::Pending,
            timestamp: Utc::now().to_rfc3339(),
            user_initiated,
            session_id: None,
            metadata_json: None,
        }
    }

    /// Create with a session ID
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Create with metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata_json = Some(metadata.to_string());
        self
    }

    /// Set status
    pub fn with_status(mut self, status: ActionStatus) -> Self {
        self.status = status;
        self
    }
}

/// Repository for action log CRUD operations
pub struct ActionLogRepository<'a> {
    db: &'a Database,
}

impl<'a> ActionLogRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        ActionLogRepository { db }
    }

    /// Insert a new action log
    pub fn insert(&self, log: &ActionLog) -> OptioResult<()> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        conn.execute(
            r#"
            INSERT INTO action_logs (id, agent_id, action_type, command_content, status, timestamp, user_initiated, session_id, metadata_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                log.id,
                log.agent_id,
                log.action_type.to_string(),
                log.command_content,
                log.status.to_string(),
                log.timestamp,
                log.user_initiated as i32,
                log.session_id,
                log.metadata_json,
            ],
        )?;

        tracing::debug!("Inserted action log: {} ({})", log.id, log.action_type);
        Ok(())
    }

    /// Update the status of an action log
    pub fn update_status(&self, id: &str, status: ActionStatus) -> OptioResult<bool> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let updated = conn.execute(
            "UPDATE action_logs SET status = ?2 WHERE id = ?1",
            params![id, status.to_string()],
        )?;

        Ok(updated > 0)
    }

    /// Get action logs for a specific agent
    pub fn get_by_agent(&self, agent_id: &str, limit: u32) -> OptioResult<Vec<ActionLog>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, agent_id, action_type, command_content, status, timestamp, user_initiated, session_id, metadata_json
            FROM action_logs
            WHERE agent_id = ?1
            ORDER BY timestamp DESC
            LIMIT ?2
            "#
        )?;

        let logs = stmt.query_map(params![agent_id, limit], |row| {
            Ok(ActionLog {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                action_type: ActionType::from(row.get::<_, String>(2)?.as_str()),
                command_content: row.get(3)?,
                status: ActionStatus::from(row.get::<_, String>(4)?.as_str()),
                timestamp: row.get(5)?,
                user_initiated: row.get::<_, i32>(6)? != 0,
                session_id: row.get(7)?,
                metadata_json: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(logs)
    }

    /// Get all action logs with optional filters
    pub fn list(&self, agent_id: Option<&str>, limit: u32) -> OptioResult<Vec<ActionLog>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let (query, params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(aid) = agent_id {
            (
                r#"
                SELECT id, agent_id, action_type, command_content, status, timestamp, user_initiated, session_id, metadata_json
                FROM action_logs
                WHERE agent_id = ?1
                ORDER BY timestamp DESC
                LIMIT ?2
                "#,
                vec![Box::new(aid.to_string()) as Box<dyn rusqlite::ToSql>, Box::new(limit)],
            )
        } else {
            (
                r#"
                SELECT id, agent_id, action_type, command_content, status, timestamp, user_initiated, session_id, metadata_json
                FROM action_logs
                ORDER BY timestamp DESC
                LIMIT ?1
                "#,
                vec![Box::new(limit) as Box<dyn rusqlite::ToSql>],
            )
        };

        let mut stmt = conn.prepare(query)?;

        let logs = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(ActionLog {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                action_type: ActionType::from(row.get::<_, String>(2)?.as_str()),
                command_content: row.get(3)?,
                status: ActionStatus::from(row.get::<_, String>(4)?.as_str()),
                timestamp: row.get(5)?,
                user_initiated: row.get::<_, i32>(6)? != 0,
                session_id: row.get(7)?,
                metadata_json: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(logs)
    }

    /// Get action logs by type
    pub fn get_by_type(&self, action_type: ActionType, limit: u32) -> OptioResult<Vec<ActionLog>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, agent_id, action_type, command_content, status, timestamp, user_initiated, session_id, metadata_json
            FROM action_logs
            WHERE action_type = ?1
            ORDER BY timestamp DESC
            LIMIT ?2
            "#
        )?;

        let logs = stmt.query_map(params![action_type.to_string(), limit], |row| {
            Ok(ActionLog {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                action_type: ActionType::from(row.get::<_, String>(2)?.as_str()),
                command_content: row.get(3)?,
                status: ActionStatus::from(row.get::<_, String>(4)?.as_str()),
                timestamp: row.get(5)?,
                user_initiated: row.get::<_, i32>(6)? != 0,
                session_id: row.get(7)?,
                metadata_json: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(logs)
    }

    /// Delete old action logs (keep last N days)
    pub fn cleanup_old_logs(&self, days_to_keep: i64) -> OptioResult<usize> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let cutoff = (Utc::now() - chrono::Duration::days(days_to_keep)).to_rfc3339();

        let deleted = conn.execute(
            "DELETE FROM action_logs WHERE timestamp < ?1",
            params![cutoff],
        )?;

        if deleted > 0 {
            tracing::info!("Cleaned up {} old action logs", deleted);
        }

        Ok(deleted)
    }

    /// Get counts by status
    pub fn count_by_status(&self) -> OptioResult<std::collections::HashMap<String, i64>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT status, COUNT(*) FROM action_logs GROUP BY status"
        )?;

        let mut counts = std::collections::HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows.flatten() {
            counts.insert(row.0, row.1);
        }

        Ok(counts)
    }
}
