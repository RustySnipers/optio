//! Security module for mTLS certificate management
//!
//! Provides helpers for loading and configuring TLS certificates for both
//! the Hub (server) and Agent (client) components of the Optio platform.

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio_rustls::rustls::{ClientConfig, RootCertStore, ServerConfig};
use tokio_rustls::{TlsAcceptor, TlsConnector};

/// Security-related errors
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Failed to read certificate file: {0}")]
    CertificateRead(String),

    #[error("Failed to read private key file: {0}")]
    PrivateKeyRead(String),

    #[error("No certificates found in file: {0}")]
    NoCertificates(String),

    #[error("No private key found in file: {0}")]
    NoPrivateKey(String),

    #[error("Failed to build TLS config: {0}")]
    TlsConfigError(String),

    #[error("Certificate verification failed: {0}")]
    VerificationError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Load certificates from a PEM file
///
/// # Arguments
/// * `path` - Path to the PEM certificate file
///
/// # Returns
/// A vector of DER-encoded certificates
pub fn load_certs<P: AsRef<Path>>(path: P) -> SecurityResult<Vec<CertificateDer<'static>>> {
    let path = path.as_ref();
    let file = File::open(path).map_err(|e| {
        SecurityError::CertificateRead(format!("{}: {}", path.display(), e))
    })?;
    let mut reader = BufReader::new(file);

    let certs_result: Result<Vec<_>, _> = certs(&mut reader).collect();
    let certs = certs_result.map_err(|e| {
        SecurityError::CertificateRead(format!("{}: {}", path.display(), e))
    })?;

    if certs.is_empty() {
        return Err(SecurityError::NoCertificates(path.display().to_string()));
    }

    tracing::debug!("Loaded {} certificate(s) from {}", certs.len(), path.display());
    Ok(certs)
}

/// Load a private key from a PEM file
///
/// Supports RSA, PKCS8, and EC private keys.
///
/// # Arguments
/// * `path` - Path to the PEM private key file
///
/// # Returns
/// The DER-encoded private key
pub fn load_private_key<P: AsRef<Path>>(path: P) -> SecurityResult<PrivateKeyDer<'static>> {
    let path = path.as_ref();
    let file = File::open(path).map_err(|e| {
        SecurityError::PrivateKeyRead(format!("{}: {}", path.display(), e))
    })?;
    let mut reader = BufReader::new(file);

    let key = private_key(&mut reader)
        .map_err(|e| SecurityError::PrivateKeyRead(format!("{}: {}", path.display(), e)))?
        .ok_or_else(|| SecurityError::NoPrivateKey(path.display().to_string()))?;

    tracing::debug!("Loaded private key from {}", path.display());
    Ok(key)
}

/// Load CA certificates into a root certificate store
///
/// # Arguments
/// * `path` - Path to the CA certificate PEM file
///
/// # Returns
/// A RootCertStore containing the CA certificates
pub fn load_ca_certs<P: AsRef<Path>>(path: P) -> SecurityResult<RootCertStore> {
    let certs = load_certs(path)?;
    let mut roots = RootCertStore::empty();

    for cert in certs {
        roots.add(cert).map_err(|e| {
            SecurityError::TlsConfigError(format!("Failed to add CA cert: {}", e))
        })?;
    }

    Ok(roots)
}

/// Server TLS configuration for the Hub
///
/// Creates a ServerConfig that:
/// - Presents the server certificate
/// - Requires client certificates (mTLS)
/// - Validates clients against the CA
pub struct HubTlsConfig {
    pub server_config: Arc<ServerConfig>,
}

impl HubTlsConfig {
    /// Create a new Hub TLS configuration
    ///
    /// # Arguments
    /// * `cert_path` - Path to server certificate PEM
    /// * `key_path` - Path to server private key PEM
    /// * `ca_path` - Path to CA certificate PEM (for client verification)
    pub fn new<P: AsRef<Path>>(
        cert_path: P,
        key_path: P,
        ca_path: P,
    ) -> SecurityResult<Self> {
        let certs = load_certs(&cert_path)?;
        let key = load_private_key(&key_path)?;
        let ca_certs = load_ca_certs(&ca_path)?;

        // Build client cert verifier
        let client_cert_verifier = rustls::server::WebPkiClientVerifier::builder(
            Arc::new(ca_certs),
        )
        .build()
        .map_err(|e| SecurityError::TlsConfigError(format!("Client verifier: {}", e)))?;

        let config = ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|e| SecurityError::TlsConfigError(format!("Protocol versions: {}", e)))?
            .with_client_cert_verifier(client_cert_verifier)
            .with_single_cert(certs, key)
            .map_err(|e| SecurityError::TlsConfigError(format!("Server config: {}", e)))?;

        tracing::info!("Hub TLS configuration initialized with mTLS enabled");

        Ok(Self {
            server_config: Arc::new(config),
        })
    }

    /// Create a TLS acceptor for incoming connections
    pub fn acceptor(&self) -> TlsAcceptor {
        TlsAcceptor::from(self.server_config.clone())
    }
}

/// Client TLS configuration for the Agent
///
/// Creates a ClientConfig that:
/// - Presents the client certificate
/// - Validates the server against the CA
pub struct AgentTlsConfig {
    pub client_config: Arc<ClientConfig>,
}

impl AgentTlsConfig {
    /// Create a new Agent TLS configuration
    ///
    /// # Arguments
    /// * `cert_path` - Path to client certificate PEM
    /// * `key_path` - Path to client private key PEM
    /// * `ca_path` - Path to CA certificate PEM (for server verification)
    pub fn new<P: AsRef<Path>>(
        cert_path: P,
        key_path: P,
        ca_path: P,
    ) -> SecurityResult<Self> {
        let certs = load_certs(&cert_path)?;
        let key = load_private_key(&key_path)?;
        let ca_certs = load_ca_certs(&ca_path)?;

        let config = ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|e| SecurityError::TlsConfigError(format!("Protocol versions: {}", e)))?
            .with_root_certificates(ca_certs)
            .with_client_auth_cert(certs, key)
            .map_err(|e| SecurityError::TlsConfigError(format!("Client config: {}", e)))?;

        tracing::info!("Agent TLS configuration initialized with mTLS enabled");

        Ok(Self {
            client_config: Arc::new(config),
        })
    }

    /// Create a TLS connector for outgoing connections
    pub fn connector(&self) -> TlsConnector {
        TlsConnector::from(self.client_config.clone())
    }
}

/// Generate test certificates for development
///
/// Creates a CA, server, and client certificate chain for local testing.
/// These should NOT be used in production.
pub mod test_certs {
    use rcgen::{
        BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose,
        IsCa, KeyPair, KeyUsagePurpose,
    };
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    /// Generate a complete test certificate chain
    ///
    /// Creates:
    /// - certs/ca.crt - Certificate Authority
    /// - certs/server.crt, certs/server.key - Server certificate/key
    /// - certs/client.crt, certs/client.key - Client certificate/key
    pub fn generate_test_certs<P: AsRef<Path>>(
        output_dir: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_dir = output_dir.as_ref();
        fs::create_dir_all(output_dir)?;

        // Generate CA
        let ca_key_pair = KeyPair::generate()?;
        let mut ca_params = CertificateParams::new(vec!["Optio Test CA".to_string()])?;
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
        ];
        ca_params
            .distinguished_name
            .push(DnType::CommonName, "Optio Test CA");
        ca_params
            .distinguished_name
            .push(DnType::OrganizationName, "Optio Dev");

        let ca_cert = ca_params.self_signed(&ca_key_pair)?;

        // Save CA certificate
        let ca_cert_pem = ca_cert.pem();
        let mut ca_file = File::create(output_dir.join("ca.crt"))?;
        ca_file.write_all(ca_cert_pem.as_bytes())?;

        // Generate Server certificate
        let server_key_pair = KeyPair::generate()?;
        let mut server_params = CertificateParams::new(vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "optio-hub".to_string(),
        ])?;
        server_params.is_ca = IsCa::NoCa;
        server_params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        server_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        server_params
            .distinguished_name
            .push(DnType::CommonName, "Optio Hub");

        let server_cert = server_params.signed_by(&server_key_pair, &ca_cert, &ca_key_pair)?;

        // Save Server certificate and key
        let server_cert_pem = server_cert.pem();
        let server_key_pem = server_key_pair.serialize_pem();
        let mut server_cert_file = File::create(output_dir.join("server.crt"))?;
        server_cert_file.write_all(server_cert_pem.as_bytes())?;
        let mut server_key_file = File::create(output_dir.join("server.key"))?;
        server_key_file.write_all(server_key_pem.as_bytes())?;

        // Generate Client certificate
        let client_key_pair = KeyPair::generate()?;
        let mut client_params =
            CertificateParams::new(vec!["optio-agent".to_string()])?;
        client_params.is_ca = IsCa::NoCa;
        client_params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        client_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
        client_params
            .distinguished_name
            .push(DnType::CommonName, "Optio Agent");

        let client_cert = client_params.signed_by(&client_key_pair, &ca_cert, &ca_key_pair)?;

        // Save Client certificate and key
        let client_cert_pem = client_cert.pem();
        let client_key_pem = client_key_pair.serialize_pem();
        let mut client_cert_file = File::create(output_dir.join("client.crt"))?;
        client_cert_file.write_all(client_cert_pem.as_bytes())?;
        let mut client_key_file = File::create(output_dir.join("client.key"))?;
        client_key_file.write_all(client_key_pem.as_bytes())?;

        println!("Generated test certificates in {}", output_dir.display());
        println!("  - ca.crt       : Certificate Authority");
        println!("  - server.crt   : Server certificate");
        println!("  - server.key   : Server private key");
        println!("  - client.crt   : Client certificate");
        println!("  - client.key   : Client private key");

        Ok(())
    }
}

// ============================================================================
// Nonce Validation for Replay Attack Prevention
// ============================================================================

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Configuration for nonce validation
#[derive(Debug, Clone)]
pub struct NonceConfig {
    /// Maximum age for timestamps to be considered valid (default: 5 minutes)
    pub max_timestamp_drift: Duration,
    /// How long to keep nonces in cache before expiry (default: 10 minutes)
    pub nonce_cache_ttl: Duration,
    /// Maximum number of nonces to track (prevents memory exhaustion)
    pub max_cached_nonces: usize,
}

impl Default for NonceConfig {
    fn default() -> Self {
        Self {
            max_timestamp_drift: Duration::from_secs(5 * 60),  // 5 minutes
            nonce_cache_ttl: Duration::from_secs(10 * 60),     // 10 minutes
            max_cached_nonces: 100_000,
        }
    }
}

/// Nonce entry with timestamp for expiry
#[derive(Debug)]
struct NonceEntry {
    seen_at: SystemTime,
}

/// Validates nonces to prevent replay attacks
///
/// Keeps track of recently seen nonces and rejects duplicates.
/// Also validates client timestamps aren't too far from server time.
pub struct NonceValidator {
    config: NonceConfig,
    /// Seen nonces with their timestamps (for cleanup)
    seen_nonces: Mutex<HashMap<String, NonceEntry>>,
}

impl NonceValidator {
    /// Create a new nonce validator with default configuration
    pub fn new() -> Self {
        Self::with_config(NonceConfig::default())
    }

    /// Create a nonce validator with custom configuration
    pub fn with_config(config: NonceConfig) -> Self {
        Self {
            config,
            seen_nonces: Mutex::new(HashMap::new()),
        }
    }

    /// Validate a nonce and timestamp
    ///
    /// # Arguments
    /// * `nonce` - Unique nonce from the client (should be UUID)
    /// * `client_timestamp_ms` - Client's timestamp in milliseconds since epoch
    ///
    /// # Returns
    /// * `Ok(())` if the nonce is valid and not a replay
    /// * `Err(NonceError)` if validation fails
    pub fn validate(&self, nonce: &str, client_timestamp_ms: u64) -> Result<(), NonceError> {
        // Skip validation if nonce is empty (for backward compatibility)
        if nonce.is_empty() {
            tracing::debug!("Empty nonce - skipping replay validation (legacy client?)");
            return Ok(());
        }

        // Validate timestamp drift
        if client_timestamp_ms > 0 {
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);

            let drift_ms = if client_timestamp_ms > now_ms {
                client_timestamp_ms - now_ms
            } else {
                now_ms - client_timestamp_ms
            };

            let max_drift_ms = self.config.max_timestamp_drift.as_millis() as u64;
            if drift_ms > max_drift_ms {
                tracing::warn!(
                    nonce = %nonce,
                    client_ts = client_timestamp_ms,
                    server_ts = now_ms,
                    drift_ms = drift_ms,
                    "Timestamp drift too large"
                );
                return Err(NonceError::TimestampDrift {
                    drift_ms,
                    max_drift_ms,
                });
            }
        }

        // Check for replay
        let mut nonces = self.seen_nonces.lock().map_err(|_| NonceError::InternalError)?;

        // Clean up expired entries periodically (every validation for simplicity)
        self.cleanup_expired(&mut nonces);

        // Check if we've seen this nonce before
        if nonces.contains_key(nonce) {
            tracing::warn!(nonce = %nonce, "Replay attack detected - duplicate nonce");
            return Err(NonceError::ReplayDetected(nonce.to_string()));
        }

        // Enforce max cache size
        if nonces.len() >= self.config.max_cached_nonces {
            tracing::warn!("Nonce cache full ({} entries) - removing oldest", nonces.len());
            // Remove oldest entries (simple strategy: remove first 10%)
            let to_remove = nonces.len() / 10;
            let keys_to_remove: Vec<_> = nonces.keys().take(to_remove).cloned().collect();
            for key in keys_to_remove {
                nonces.remove(&key);
            }
        }

        // Record the nonce
        nonces.insert(
            nonce.to_string(),
            NonceEntry {
                seen_at: SystemTime::now(),
            },
        );

        tracing::debug!(nonce = %nonce, "Nonce validated successfully");
        Ok(())
    }

    /// Clean up expired nonces from the cache
    fn cleanup_expired(&self, nonces: &mut HashMap<String, NonceEntry>) {
        let now = SystemTime::now();
        let ttl = self.config.nonce_cache_ttl;

        nonces.retain(|_, entry| {
            now.duration_since(entry.seen_at)
                .map(|age| age < ttl)
                .unwrap_or(true)
        });
    }
}

impl Default for NonceValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors from nonce validation
#[derive(Debug, thiserror::Error)]
pub enum NonceError {
    #[error("Replay attack detected: nonce '{0}' has been seen before")]
    ReplayDetected(String),

    #[error("Timestamp drift too large: {drift_ms}ms exceeds maximum {max_drift_ms}ms")]
    TimestampDrift { drift_ms: u64, max_drift_ms: u64 },

    #[error("Internal validation error")]
    InternalError,
}

/// Generate a nonce for use in requests (call from agent side)
pub fn generate_nonce() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Get current timestamp in milliseconds for client_timestamp_ms field
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_certs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_certs")
    }

    #[test]
    fn test_generate_and_load_certs() {
        let dir = test_certs_dir();

        // Generate test certs
        test_certs::generate_test_certs(&dir).expect("Failed to generate test certs");

        // Test loading CA
        let ca_certs = load_ca_certs(dir.join("ca.crt")).expect("Failed to load CA");
        assert!(!ca_certs.is_empty());

        // Test loading server certs
        let server_certs = load_certs(dir.join("server.crt")).expect("Failed to load server cert");
        assert!(!server_certs.is_empty());

        let _server_key =
            load_private_key(dir.join("server.key")).expect("Failed to load server key");

        // Test loading client certs
        let client_certs = load_certs(dir.join("client.crt")).expect("Failed to load client cert");
        assert!(!client_certs.is_empty());

        let _client_key =
            load_private_key(dir.join("client.key")).expect("Failed to load client key");

        // Clean up
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_hub_tls_config() {
        let dir = test_certs_dir().join("hub_test");
        test_certs::generate_test_certs(&dir).expect("Failed to generate test certs");

        let config = HubTlsConfig::new(
            dir.join("server.crt"),
            dir.join("server.key"),
            dir.join("ca.crt"),
        );

        assert!(config.is_ok(), "Failed to create Hub TLS config: {:?}", config.err());

        // Clean up
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_agent_tls_config() {
        let dir = test_certs_dir().join("agent_test");
        test_certs::generate_test_certs(&dir).expect("Failed to generate test certs");

        let config = AgentTlsConfig::new(
            dir.join("client.crt"),
            dir.join("client.key"),
            dir.join("ca.crt"),
        );

        assert!(config.is_ok(), "Failed to create Agent TLS config: {:?}", config.err());

        // Clean up
        std::fs::remove_dir_all(&dir).ok();
    }
}
