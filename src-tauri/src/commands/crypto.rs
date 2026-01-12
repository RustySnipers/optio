//! Crypto command handlers for Hub key management and command signing
//!
//! Provides Tauri IPC commands for:
//! - Generating Hub signing keypairs
//! - Signing commands for Agent execution
//! - Exporting public keys for Agent embedding

use crate::crypto::{generate_powershell_verifier, ExportedKeyPair, HubKeyPair, SignedCommand};
use crate::error::{OptioError, OptioResult};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// State container for the Hub's signing keypair
pub struct CryptoState {
    keypair: Mutex<Option<HubKeyPair>>,
}

impl Default for CryptoState {
    fn default() -> Self {
        CryptoState {
            keypair: Mutex::new(None),
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
/// This creates a new Ed25519 keypair for signing commands to Agents.
/// The private key is stored in memory (state) and should also be
/// persisted securely to the encrypted database.
#[tauri::command]
pub async fn generate_signing_keypair(
    state: State<'_, CryptoState>,
) -> Result<KeyGenerationResponse, OptioError> {
    let keypair = HubKeyPair::generate();
    let exported = keypair.export();

    let response = KeyGenerationResponse {
        public_key: exported.public_key.clone(),
        key_id: exported.key_id.clone(),
        generated_at: exported.generated_at.to_rfc3339(),
    };

    // Store in state
    let mut guard = state
        .keypair
        .lock()
        .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;
    *guard = Some(keypair);

    tracing::info!("Generated new Hub signing keypair: {}", exported.key_id);

    Ok(response)
}

/// Get the current public key (if a keypair exists)
#[tauri::command]
pub async fn get_public_key(state: State<'_, CryptoState>) -> Result<Option<String>, OptioError> {
    let guard = state
        .keypair
        .lock()
        .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;

    Ok(guard.as_ref().map(|kp| kp.public_key_hex()))
}

/// Sign a command for Agent execution
///
/// Returns a SignedCommand envelope that can be dispatched to Agents.
/// The Agent will verify the signature before executing.
#[tauri::command]
pub async fn sign_command(
    request: SignCommandRequest,
    state: State<'_, CryptoState>,
) -> Result<SignedCommand, OptioError> {
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
    state: State<'_, CryptoState>,
) -> Result<String, OptioError> {
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
/// Should be encrypted before persistence.
#[tauri::command]
pub async fn export_keypair(state: State<'_, CryptoState>) -> Result<ExportedKeyPair, OptioError> {
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
    state: State<'_, CryptoState>,
) -> Result<KeyGenerationResponse, OptioError> {
    let keypair = HubKeyPair::import(&exported)?;

    let response = KeyGenerationResponse {
        public_key: exported.public_key.clone(),
        key_id: exported.key_id.clone(),
        generated_at: exported.generated_at.to_rfc3339(),
    };

    let mut guard = state
        .keypair
        .lock()
        .map_err(|e| OptioError::Crypto(format!("Failed to acquire lock: {}", e)))?;
    *guard = Some(keypair);

    tracing::info!("Imported Hub signing keypair: {}", exported.key_id);

    Ok(response)
}
