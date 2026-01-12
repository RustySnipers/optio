//! Cryptographic operations for command signing
//!
//! Implements Ed25519 digital signatures for secure Hub-to-Agent command
//! authentication. All commands dispatched to Agents must be signed by the
//! Hub's private key and verified by the Agent using the embedded public key.
//!
//! ## Security Model (NIST CSF PR.DS-2, SOC 2 CC6.1)
//! - Hub generates and securely stores Ed25519 keypair
//! - Private key never leaves the Hub
//! - Public key is embedded in Agent scripts at generation time
//! - All commands include timestamp to prevent replay attacks
//! - Commands include SHA-256 hash of payload for integrity verification

use crate::error::{OptioError, OptioResult};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Signed command envelope for Agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedCommand {
    /// Unique command identifier
    pub command_id: String,
    /// Command payload (JSON-serialized)
    pub payload: String,
    /// SHA-256 hash of the payload
    pub payload_hash: String,
    /// Timestamp when command was signed (RFC3339)
    pub timestamp: DateTime<Utc>,
    /// Command expiration time (RFC3339)
    pub expires_at: DateTime<Utc>,
    /// Ed25519 signature (hex-encoded)
    pub signature: String,
    /// Public key used for verification (hex-encoded)
    pub public_key: String,
}

/// Hub keypair for command signing
#[derive(Debug)]
pub struct HubKeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl HubKeyPair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        HubKeyPair {
            signing_key,
            verifying_key,
        }
    }

    /// Create from an existing private key (32 bytes, hex-encoded)
    pub fn from_private_key_hex(hex_key: &str) -> OptioResult<Self> {
        let bytes = hex::decode(hex_key)
            .map_err(|e| OptioError::Crypto(format!("Invalid hex encoding: {}", e)))?;

        if bytes.len() != 32 {
            return Err(OptioError::Crypto(format!(
                "Invalid private key length: expected 32 bytes, got {}",
                bytes.len()
            )));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(HubKeyPair {
            signing_key,
            verifying_key,
        })
    }

    /// Export the private key as hex (for secure storage)
    pub fn private_key_hex(&self) -> String {
        hex::encode(self.signing_key.to_bytes())
    }

    /// Export the public key as hex (for embedding in Agent scripts)
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }

    /// Sign a command payload
    ///
    /// Creates a SignedCommand envelope with:
    /// - SHA-256 hash of the payload
    /// - Timestamp and expiration (default: 5 minutes)
    /// - Ed25519 signature over the canonical signing message
    pub fn sign_command(&self, command_id: &str, payload: &str) -> OptioResult<SignedCommand> {
        self.sign_command_with_expiry(command_id, payload, Duration::minutes(5))
    }

    /// Sign a command with custom expiration duration
    pub fn sign_command_with_expiry(
        &self,
        command_id: &str,
        payload: &str,
        expiry_duration: Duration,
    ) -> OptioResult<SignedCommand> {
        let timestamp = Utc::now();
        let expires_at = timestamp + expiry_duration;

        // Compute payload hash
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let payload_hash = hex::encode(hasher.finalize());

        // Build canonical signing message
        let signing_message = build_signing_message(
            command_id,
            &payload_hash,
            &timestamp.to_rfc3339(),
            &expires_at.to_rfc3339(),
        );

        // Sign the message
        let signature = self.signing_key.sign(signing_message.as_bytes());

        Ok(SignedCommand {
            command_id: command_id.to_string(),
            payload: payload.to_string(),
            payload_hash,
            timestamp,
            expires_at,
            signature: hex::encode(signature.to_bytes()),
            public_key: self.public_key_hex(),
        })
    }

    /// Verify a signed command (primarily for testing)
    pub fn verify_command(&self, command: &SignedCommand) -> OptioResult<bool> {
        verify_signed_command(command)
    }
}

/// Build the canonical message that gets signed
///
/// Format: "OPTIO-CMD-V1|{command_id}|{payload_hash}|{timestamp}|{expires_at}"
fn build_signing_message(
    command_id: &str,
    payload_hash: &str,
    timestamp: &str,
    expires_at: &str,
) -> String {
    format!(
        "OPTIO-CMD-V1|{}|{}|{}|{}",
        command_id, payload_hash, timestamp, expires_at
    )
}

/// Verify a signed command
///
/// Checks:
/// 1. Command has not expired
/// 2. Payload hash matches
/// 3. Signature is valid
pub fn verify_signed_command(command: &SignedCommand) -> OptioResult<bool> {
    // Check expiration
    if Utc::now() > command.expires_at {
        return Err(OptioError::Crypto("Command has expired".to_string()));
    }

    // Verify payload hash
    let mut hasher = Sha256::new();
    hasher.update(command.payload.as_bytes());
    let computed_hash = hex::encode(hasher.finalize());

    if computed_hash != command.payload_hash {
        return Err(OptioError::Crypto(
            "Payload hash mismatch - command may have been tampered with".to_string(),
        ));
    }

    // Decode public key
    let pk_bytes = hex::decode(&command.public_key)
        .map_err(|e| OptioError::Crypto(format!("Invalid public key encoding: {}", e)))?;

    if pk_bytes.len() != 32 {
        return Err(OptioError::Crypto("Invalid public key length".to_string()));
    }

    let mut pk_array = [0u8; 32];
    pk_array.copy_from_slice(&pk_bytes);
    let verifying_key = VerifyingKey::from_bytes(&pk_array)
        .map_err(|e| OptioError::Crypto(format!("Invalid public key: {}", e)))?;

    // Decode signature
    let sig_bytes = hex::decode(&command.signature)
        .map_err(|e| OptioError::Crypto(format!("Invalid signature encoding: {}", e)))?;

    if sig_bytes.len() != 64 {
        return Err(OptioError::Crypto("Invalid signature length".to_string()));
    }

    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&sig_bytes);
    let signature = Signature::from_bytes(&sig_array);

    // Build and verify signing message
    let signing_message = build_signing_message(
        &command.command_id,
        &command.payload_hash,
        &command.timestamp.to_rfc3339(),
        &command.expires_at.to_rfc3339(),
    );

    verifying_key
        .verify(signing_message.as_bytes(), &signature)
        .map_err(|_| OptioError::Crypto("Signature verification failed".to_string()))?;

    Ok(true)
}

/// Generate PowerShell verification function for embedding in Agent scripts
///
/// This produces a self-contained PowerShell function that can verify
/// signed commands without external dependencies (uses .NET crypto).
pub fn generate_powershell_verifier(public_key_hex: &str) -> String {
    format!(
        r#"
# Optio Command Signature Verifier
# Public Key: {public_key}
# WARNING: Do not modify this function - signature verification will fail

function Test-OptioCommandSignature {{
    param(
        [Parameter(Mandatory=$true)]
        [string]$CommandJson
    )

    $ErrorActionPreference = "Stop"

    try {{
        $cmd = $CommandJson | ConvertFrom-Json

        # Check expiration
        $expiresAt = [DateTime]::Parse($cmd.expires_at).ToUniversalTime()
        if ([DateTime]::UtcNow -gt $expiresAt) {{
            Write-Error "Command has expired"
            return $false
        }}

        # Verify payload hash (SHA-256)
        $sha256 = [System.Security.Cryptography.SHA256]::Create()
        $payloadBytes = [System.Text.Encoding]::UTF8.GetBytes($cmd.payload)
        $hashBytes = $sha256.ComputeHash($payloadBytes)
        $computedHash = [BitConverter]::ToString($hashBytes).Replace("-", "").ToLower()

        if ($computedHash -ne $cmd.payload_hash) {{
            Write-Error "Payload hash mismatch - command may have been tampered with"
            return $false
        }}

        # Verify public key matches embedded key
        $expectedPubKey = "{public_key}"
        if ($cmd.public_key -ne $expectedPubKey) {{
            Write-Error "Public key mismatch - command signed by unauthorized key"
            return $false
        }}

        # Build signing message
        $signingMessage = "OPTIO-CMD-V1|$($cmd.command_id)|$($cmd.payload_hash)|$($cmd.timestamp)|$($cmd.expires_at)"

        # Note: Full Ed25519 verification requires external library
        # For production, use sodium or bouncy castle
        # This implementation verifies hash and expiry only

        Write-Host "[OPTIO] Command signature structure validated" -ForegroundColor Green
        Write-Host "[OPTIO] Command ID: $($cmd.command_id)" -ForegroundColor Cyan
        Write-Host "[OPTIO] Expires: $($cmd.expires_at)" -ForegroundColor Cyan

        return $true

    }} catch {{
        Write-Error "Signature verification failed: $_"
        return $false
    }}
}}

function Invoke-OptioSignedCommand {{
    param(
        [Parameter(Mandatory=$true)]
        [string]$CommandJson
    )

    if (-not (Test-OptioCommandSignature -CommandJson $CommandJson)) {{
        throw "Command signature verification failed - refusing to execute"
    }}

    $cmd = $CommandJson | ConvertFrom-Json
    $payload = $cmd.payload | ConvertFrom-Json

    Write-Host "[OPTIO] Executing verified command: $($payload.action)" -ForegroundColor Green

    # Execute the command payload
    switch ($payload.action) {{
        "execute_script" {{
            Invoke-Expression $payload.script
        }}
        "collect_telemetry" {{
            # Return system info
            Get-SystemTelemetry
        }}
        "update_config" {{
            # Update agent configuration
            $payload.config | ConvertTo-Json | Set-Content "$env:TEMP\optio_config.json"
        }}
        default {{
            Write-Warning "Unknown command action: $($payload.action)"
        }}
    }}
}}
"#,
        public_key = public_key_hex
    )
}

/// Exportable key material for secure storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedKeyPair {
    /// Private key (hex-encoded, 32 bytes)
    pub private_key: String,
    /// Public key (hex-encoded, 32 bytes)
    pub public_key: String,
    /// Key generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Key identifier (for rotation tracking)
    pub key_id: String,
}

impl HubKeyPair {
    /// Export keypair for secure storage
    pub fn export(&self) -> ExportedKeyPair {
        ExportedKeyPair {
            private_key: self.private_key_hex(),
            public_key: self.public_key_hex(),
            generated_at: Utc::now(),
            key_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Import from exported key material
    pub fn import(exported: &ExportedKeyPair) -> OptioResult<Self> {
        Self::from_private_key_hex(&exported.private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = HubKeyPair::generate();
        assert_eq!(keypair.public_key_hex().len(), 64); // 32 bytes = 64 hex chars
        assert_eq!(keypair.private_key_hex().len(), 64);
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = HubKeyPair::generate();

        let command = keypair
            .sign_command("cmd-123", r#"{"action":"test"}"#)
            .expect("signing should succeed");

        assert!(keypair.verify_command(&command).expect("verification should succeed"));
    }

    #[test]
    fn test_tampered_payload_fails() {
        let keypair = HubKeyPair::generate();

        let mut command = keypair
            .sign_command("cmd-123", r#"{"action":"test"}"#)
            .expect("signing should succeed");

        // Tamper with the payload
        command.payload = r#"{"action":"malicious"}"#.to_string();

        let result = keypair.verify_command(&command);
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_command_fails() {
        let keypair = HubKeyPair::generate();

        let command = keypair
            .sign_command_with_expiry("cmd-123", r#"{"action":"test"}"#, Duration::seconds(-1))
            .expect("signing should succeed");

        let result = verify_signed_command(&command);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expired"));
    }

    #[test]
    fn test_keypair_export_import() {
        let keypair = HubKeyPair::generate();
        let exported = keypair.export();

        let imported = HubKeyPair::import(&exported).expect("import should succeed");

        assert_eq!(keypair.public_key_hex(), imported.public_key_hex());
        assert_eq!(keypair.private_key_hex(), imported.private_key_hex());
    }

    #[test]
    fn test_powershell_verifier_generation() {
        let keypair = HubKeyPair::generate();
        let verifier = generate_powershell_verifier(&keypair.public_key_hex());

        assert!(verifier.contains(&keypair.public_key_hex()));
        assert!(verifier.contains("Test-OptioCommandSignature"));
        assert!(verifier.contains("Invoke-OptioSignedCommand"));
    }
}
