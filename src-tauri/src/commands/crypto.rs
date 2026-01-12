//! Crypto command handlers for Hub key management and command signing
//!
//! Provides Tauri IPC commands for:
//! - Generating Hub signing keypairs (with database persistence)
//! - Signing commands for Agent execution
//! - Exporting public keys for Agent embedding
//! - Key rotation and management

use crate::crypto::{
    generate_powershell_verifier, ExportedKeyPair, HubKeyPair, KeyPairRepository, SignedCommand,
    StoredKeyPair,
};
use crate::db::Database;
use crate::error::OptioError;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Manager, State};

/// State container for the Hub's signing keypair
pub struct CryptoState {
    keypair: Mutex<Option<HubKeyPair>>,
    active_key_id: Mutex<Option<String>>,
}

impl Default for CryptoState {
    fn default() -> Self {
        CryptoState {
            keypair: Mutex::new(None),
            active_key_id: Mutex::new(None),
        }
    }
}

impl CryptoState {
    /// Load the active keypair from the database into memory
    pub fn load_from_db(&self, db: &Database) -> Result<bool, OptioError> {
        let repo = KeyPairRepository::new_with_default_key(db);

        if let Some((keypair, stored)) = repo.load_active()? {
            let mut kp_guard = self
                .keypair
                .lock()
                .map_err(|e| OptioError::Crypto(e.to_string()))?;
            let mut id_guard = self
                .active_key_id
                .lock()
                .map_err(|e| OptioError::Crypto(e.to_string()))?;

            *kp_guard = Some(keypair);
            *id_guard = Some(stored.key_id.clone());

            tracing::info!("Loaded active keypair from database: {}", stored.key_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Response containing the public key after key generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyGenerationResponse {
    pub public_key: String,
    pub key_id: String,
    pub generated_at: String,
    pub persisted: bool,
}

/// Request for signing a command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignCommandRequest {
    pub command_id: String,
    pub payload: String,
    /// Optional expiry in seconds (default: 300 = 5 minutes)
    pub expiry_seconds: Option<i64>,
}

/// Generate a new Hub signing keypair
///
/// Creates a new Ed25519 keypair for signing commands to Agents.
/// The keypair is persisted to the encrypted database and stored in memory.
#[tauri::command]
pub async fn generate_signing_keypair(
    app_handle: tauri::AppHandle,
    state: State<'_, CryptoState>,
) -> Result<KeyGenerationResponse, OptioError> {
    let keypair = HubKeyPair::generate();
    let exported = keypair.export();

    // Try to persist to database
    let persisted = if let Some(db) = app_handle.try_state::<Database>() {
        let repo = KeyPairRepository::new_with_default_key(&db);
        match repo.save(&keypair, &exported.key_id) {
            Ok(_) => {
                tracing::info!("Persisted keypair to database: {}", exported.key_id);
                true
            }
            Err(e) => {
                tracing::warn!("Failed to persist keypair: {}", e);
                false
            }
        }
    } else {
        tracing::warn!("Database not available, keypair stored in memory only");
        false
    };

    let response = KeyGenerationResponse {
        public_key: exported.public_key.clone(),
        key_id: exported.key_id.clone(),
        generated_at: exported.generated_at.to_rfc3339(),
        persisted,
    };

    // Store in memory state
    {
        let mut guard = state
            .keypair
            .lock()
            .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;
        *guard = Some(keypair);
    }
    {
        let mut guard = state
            .active_key_id
            .lock()
            .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;
        *guard = Some(exported.key_id.clone());
    }

    tracing::info!("Generated new Hub signing keypair: {}", exported.key_id);

    Ok(response)
}

/// Get the current public key (if a keypair exists)
///
/// Will attempt to load from database if no keypair is in memory.
#[tauri::command]
pub async fn get_public_key(
    app_handle: tauri::AppHandle,
    state: State<'_, CryptoState>,
) -> Result<Option<String>, OptioError> {
    // Check memory first
    {
        let guard = state
            .keypair
            .lock()
            .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;

        if let Some(ref kp) = *guard {
            return Ok(Some(kp.public_key_hex()));
        }
    }

    // Try to load from database
    if let Some(db) = app_handle.try_state::<Database>() {
        if state.load_from_db(&db)? {
            let guard = state
                .keypair
                .lock()
                .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;
            return Ok(guard.as_ref().map(|kp| kp.public_key_hex()));
        }
    }

    Ok(None)
}

/// Sign a command for Agent execution
///
/// Returns a SignedCommand envelope that can be dispatched to Agents.
/// The Agent will verify the signature before executing.
#[tauri::command]
pub async fn sign_command(
    request: SignCommandRequest,
    app_handle: tauri::AppHandle,
    state: State<'_, CryptoState>,
) -> Result<SignedCommand, OptioError> {
    // Ensure keypair is loaded
    ensure_keypair_loaded(&app_handle, &state)?;

    let guard = state
        .keypair
        .lock()
        .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;

    let keypair = guard.as_ref().ok_or_else(|| {
        OptioError::Crypto("No signing keypair available. Generate one first.".to_string())
    })?;

    let expiry = chrono::Duration::seconds(request.expiry_seconds.unwrap_or(300));

    let signed = keypair.sign_command_with_expiry(&request.command_id, &request.payload, expiry)?;

    tracing::debug!(
        "Signed command {} (expires: {})",
        signed.command_id,
        signed.expires_at
    );

    Ok(signed)
}

/// Generate PowerShell verification code for embedding in Agent scripts
///
/// Returns a self-contained PowerShell function that can verify signed
/// commands using the Hub's public key.
#[tauri::command]
pub async fn generate_agent_verifier(
    app_handle: tauri::AppHandle,
    state: State<'_, CryptoState>,
) -> Result<String, OptioError> {
    ensure_keypair_loaded(&app_handle, &state)?;

    let guard = state
        .keypair
        .lock()
        .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;

    let keypair = guard.as_ref().ok_or_else(|| {
        OptioError::Crypto("No signing keypair available. Generate one first.".to_string())
    })?;

    let verifier = generate_powershell_verifier(&keypair.public_key_hex());

    Ok(verifier)
}

/// Export the keypair for secure storage
///
/// WARNING: This exports the private key! Handle with extreme care.
#[tauri::command]
pub async fn export_keypair(
    app_handle: tauri::AppHandle,
    state: State<'_, CryptoState>,
) -> Result<ExportedKeyPair, OptioError> {
    ensure_keypair_loaded(&app_handle, &state)?;

    let guard = state
        .keypair
        .lock()
        .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;

    let keypair = guard.as_ref().ok_or_else(|| {
        OptioError::Crypto("No signing keypair available. Generate one first.".to_string())
    })?;

    Ok(keypair.export())
}

/// Import a previously exported keypair
#[tauri::command]
pub async fn import_keypair(
    exported: ExportedKeyPair,
    app_handle: tauri::AppHandle,
    state: State<'_, CryptoState>,
) -> Result<KeyGenerationResponse, OptioError> {
    let keypair = HubKeyPair::import(&exported)?;

    // Persist to database
    let persisted = if let Some(db) = app_handle.try_state::<Database>() {
        let repo = KeyPairRepository::new_with_default_key(&db);
        match repo.save(&keypair, &exported.key_id) {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("Failed to persist imported keypair: {}", e);
                false
            }
        }
    } else {
        false
    };

    let response = KeyGenerationResponse {
        public_key: exported.public_key.clone(),
        key_id: exported.key_id.clone(),
        generated_at: exported.generated_at.to_rfc3339(),
        persisted,
    };

    {
        let mut guard = state
            .keypair
            .lock()
            .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;
        *guard = Some(keypair);
    }
    {
        let mut guard = state
            .active_key_id
            .lock()
            .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;
        *guard = Some(exported.key_id.clone());
    }

    tracing::info!("Imported Hub signing keypair: {}", exported.key_id);

    Ok(response)
}

/// List all stored keypairs (metadata only, no private keys)
#[tauri::command]
pub async fn list_keypairs(app_handle: tauri::AppHandle) -> Result<Vec<StoredKeyPair>, OptioError> {
    let db = app_handle.try_state::<Database>().ok_or_else(|| {
        OptioError::Database("Database not available".to_string())
    })?;

    let repo = KeyPairRepository::new_with_default_key(&db);
    repo.list()
}

/// Rotate to a new keypair (deactivates current, creates new active key)
#[tauri::command]
pub async fn rotate_keypair(
    app_handle: tauri::AppHandle,
    state: State<'_, CryptoState>,
) -> Result<KeyGenerationResponse, OptioError> {
    let keypair = HubKeyPair::generate();
    let exported = keypair.export();

    let db = app_handle.try_state::<Database>().ok_or_else(|| {
        OptioError::Database("Database not available for key rotation".to_string())
    })?;

    let repo = KeyPairRepository::new_with_default_key(&db);
    repo.rotate(&keypair, &exported.key_id)?;

    let response = KeyGenerationResponse {
        public_key: exported.public_key.clone(),
        key_id: exported.key_id.clone(),
        generated_at: exported.generated_at.to_rfc3339(),
        persisted: true,
    };

    {
        let mut guard = state
            .keypair
            .lock()
            .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;
        *guard = Some(keypair);
    }
    {
        let mut guard = state
            .active_key_id
            .lock()
            .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;
        *guard = Some(exported.key_id.clone());
    }

    tracing::info!("Rotated to new keypair: {}", exported.key_id);

    Ok(response)
}

/// Get the active key ID
#[tauri::command]
pub async fn get_active_key_id(
    app_handle: tauri::AppHandle,
    state: State<'_, CryptoState>,
) -> Result<Option<String>, OptioError> {
    // Check memory first
    {
        let guard = state
            .active_key_id
            .lock()
            .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;

        if guard.is_some() {
            return Ok(guard.clone());
        }
    }

    // Try to load from database
    if let Some(db) = app_handle.try_state::<Database>() {
        if state.load_from_db(&db)? {
            let guard = state
                .active_key_id
                .lock()
                .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;
            return Ok(guard.clone());
        }
    }

    Ok(None)
}

/// Helper to ensure a keypair is loaded from DB if not in memory
fn ensure_keypair_loaded(
    app_handle: &tauri::AppHandle,
    state: &State<'_, CryptoState>,
) -> Result<(), OptioError> {
    // Check if already loaded
    {
        let guard = state
            .keypair
            .lock()
            .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;

        if guard.is_some() {
            return Ok(());
        }
    }

    // Try to load from database
    if let Some(db) = app_handle.try_state::<Database>() {
        state.load_from_db(&db)?;
    }

    Ok(())
}
