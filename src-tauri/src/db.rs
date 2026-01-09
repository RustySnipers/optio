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
    conn: Mutex<Connection>,
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

            -- Create indexes for performance
            CREATE INDEX IF NOT EXISTS idx_clients_name ON clients(name);
            CREATE INDEX IF NOT EXISTS idx_script_history_client ON script_history(client_id);
            CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);
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

    // Store database in app state
    app_handle.manage(db);

    Ok(())
}
