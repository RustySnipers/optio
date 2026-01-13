//! Phoenix Module - Agent Self-Update Mechanism
//!
//! This module implements a robust self-update mechanism for the Optio Agent.
//! It handles:
//! - Periodic version checks against the Hub
//! - Binary download with progress tracking
//! - SHA256 hash verification
//! - Atomic binary replacement using the "rename-and-replace" strategy
//! - Graceful restart (relies on Windows Service Manager)
//!
//! # Security Considerations
//! - All downloads are over mTLS (same as heartbeat channel)
//! - Binary integrity is verified via SHA256 hash
//! - Update server endpoint is authenticated via client certificates

use anyhow::{Context, Result};
use reqwest::Client;
use semver::Version;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Current agent version
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default update check interval (1 hour)
const DEFAULT_UPDATE_INTERVAL: Duration = Duration::from_secs(3600);

/// Errors that can occur during update operations
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Version check failed: {0}")]
    VersionCheck(String),

    #[error("Download failed: {0}")]
    Download(String),

    #[error("Hash verification failed: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Binary replacement failed: {0}")]
    Replacement(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Version parse error: {0}")]
    VersionParse(#[from] semver::Error),
}

/// Response from the Hub's CheckVersion endpoint
#[derive(Debug, Clone, serde::Deserialize)]
pub struct VersionInfo {
    /// Latest available version string (semver)
    pub version: String,
    /// SHA256 hash of the binary (hex-encoded)
    pub sha256: String,
    /// Optional release notes
    pub release_notes: Option<String>,
    /// Size of the binary in bytes
    pub size_bytes: Option<u64>,
}

/// Configuration for the update module
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// Base URL for the update server (Hub address)
    pub hub_address: String,
    /// Update check interval
    pub check_interval: Duration,
    /// Path to current executable
    pub current_exe: PathBuf,
    /// Path to CA certificate for HTTPS
    pub ca_cert_path: Option<PathBuf>,
    /// Path to client certificate
    pub client_cert_path: Option<PathBuf>,
    /// Path to client key
    pub client_key_path: Option<PathBuf>,
    /// Whether updates are enabled
    pub enabled: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            hub_address: "http://127.0.0.1:50051".to_string(),
            check_interval: DEFAULT_UPDATE_INTERVAL,
            current_exe: std::env::current_exe().unwrap_or_default(),
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            enabled: true,
        }
    }
}

/// The Phoenix update manager
pub struct UpdateManager {
    config: UpdateConfig,
    client: Option<Client>,
}

impl UpdateManager {
    /// Create a new update manager
    pub fn new(config: UpdateConfig) -> Self {
        Self {
            config,
            client: None,
        }
    }

    /// Initialize the HTTP client with mTLS if configured
    pub fn init(&mut self) -> Result<()> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(30));

        // Configure mTLS if certificates are provided
        if let (Some(ca_path), Some(cert_path), Some(key_path)) = (
            &self.config.ca_cert_path,
            &self.config.client_cert_path,
            &self.config.client_key_path,
        ) {
            let ca_cert = fs::read(ca_path)
                .context("Failed to read CA certificate")?;
            let client_cert = fs::read(cert_path)
                .context("Failed to read client certificate")?;
            let client_key = fs::read(key_path)
                .context("Failed to read client key")?;

            // Load CA certificate
            let ca = reqwest::Certificate::from_pem(&ca_cert)
                .context("Failed to parse CA certificate")?;

            // Create client identity
            let mut identity_pem = client_cert;
            identity_pem.extend_from_slice(&client_key);
            let identity = reqwest::Identity::from_pem(&identity_pem)
                .context("Failed to create client identity")?;

            builder = builder
                .add_root_certificate(ca)
                .identity(identity);

            info!("Update client configured with mTLS");
        } else {
            warn!("Update client running without mTLS - for development only!");
        }

        self.client = Some(builder.build()?);
        Ok(())
    }

    /// Get the HTTP client
    fn client(&self) -> Result<&Client, UpdateError> {
        self.client
            .as_ref()
            .ok_or_else(|| UpdateError::VersionCheck("Client not initialized".to_string()))
    }

    /// Check if a newer version is available
    pub async fn check_version(&self) -> Result<Option<VersionInfo>, UpdateError> {
        let client = self.client()?;
        let url = format!("{}/api/v1/agent/version", self.config.hub_address);

        debug!("Checking for updates at: {}", url);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| UpdateError::VersionCheck(e.to_string()))?;

        if !response.status().is_success() {
            return Err(UpdateError::VersionCheck(format!(
                "Server returned status: {}",
                response.status()
            )));
        }

        let version_info: VersionInfo = response
            .json()
            .await
            .map_err(|e| UpdateError::VersionCheck(e.to_string()))?;

        // Compare versions
        let current = Version::parse(CURRENT_VERSION)?;
        let latest = Version::parse(&version_info.version)?;

        if latest > current {
            info!(
                "Update available: {} -> {} (current: {})",
                CURRENT_VERSION, version_info.version, CURRENT_VERSION
            );
            Ok(Some(version_info))
        } else {
            debug!(
                "Agent is up to date (current: {}, latest: {})",
                CURRENT_VERSION, version_info.version
            );
            Ok(None)
        }
    }

    /// Download the new binary to a temporary location
    pub async fn download_binary(&self, version_info: &VersionInfo) -> Result<PathBuf, UpdateError> {
        let client = self.client()?;
        let url = format!("{}/api/v1/agent/binary", self.config.hub_address);

        info!(
            "Downloading update v{} from {}",
            version_info.version, url
        );

        let response = client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(UpdateError::Download(format!(
                "Server returned status: {}",
                response.status()
            )));
        }

        // Create temporary file path
        let temp_path = self.config.current_exe.with_extension("new");

        // Download and write to file
        let bytes = response.bytes().await?;
        let mut file = File::create(&temp_path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;

        info!(
            "Downloaded {} bytes to {:?}",
            bytes.len(),
            temp_path
        );

        Ok(temp_path)
    }

    /// Verify the SHA256 hash of the downloaded binary
    pub fn verify_hash(&self, file_path: &PathBuf, expected_hash: &str) -> Result<(), UpdateError> {
        info!("Verifying binary hash...");

        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)?;

        let actual_hash = hex::encode(hasher.finalize());

        if actual_hash.to_lowercase() != expected_hash.to_lowercase() {
            // Delete the corrupt file
            let _ = fs::remove_file(file_path);

            return Err(UpdateError::HashMismatch {
                expected: expected_hash.to_string(),
                actual: actual_hash,
            });
        }

        info!("Hash verification successful");
        Ok(())
    }

    /// Replace the current binary with the new one and restart
    ///
    /// This uses the `self-replace` crate which handles Windows file locking
    /// by using a "rename-and-replace" strategy:
    /// 1. Rename current executable to `.old`
    /// 2. Move new executable to current path
    /// 3. Exit process (Windows Service Manager will restart)
    pub fn apply_update(&self, new_binary_path: &PathBuf) -> Result<(), UpdateError> {
        info!("Applying update...");

        // On Windows, we can't directly replace a running executable
        // The self-replace crate handles this by:
        // 1. Renaming the running exe to a temp name
        // 2. Moving the new exe to the original path
        // 3. Scheduling the old exe for deletion on reboot
        self_replace::self_replace(new_binary_path)
            .map_err(|e| UpdateError::Replacement(e.to_string()))?;

        info!("Binary replaced successfully. Agent will restart.");

        // Clean up the .new file if it still exists
        let _ = fs::remove_file(new_binary_path);

        Ok(())
    }

    /// Perform a full update cycle: check, download, verify, apply
    pub async fn perform_update(&self) -> Result<bool, UpdateError> {
        // Check for new version
        let version_info = match self.check_version().await? {
            Some(info) => info,
            None => return Ok(false), // No update available
        };

        // Download the new binary
        let new_binary_path = self.download_binary(&version_info).await?;

        // Verify the hash
        if let Err(e) = self.verify_hash(&new_binary_path, &version_info.sha256) {
            error!("Hash verification failed: {}", e);
            return Err(e);
        }

        // Apply the update
        self.apply_update(&new_binary_path)?;

        Ok(true)
    }

    /// Run the update loop (should be spawned as a background task)
    pub async fn run_update_loop(&self) {
        if !self.config.enabled {
            info!("Updates are disabled");
            return;
        }

        info!(
            "Starting update loop (interval: {:?})",
            self.config.check_interval
        );

        let mut check_interval = interval(self.config.check_interval);

        loop {
            check_interval.tick().await;

            match self.perform_update().await {
                Ok(true) => {
                    info!("Update applied successfully. Exiting for restart...");
                    // Give a moment for logs to flush
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    // Exit gracefully - Windows Service Manager will restart us
                    std::process::exit(0);
                }
                Ok(false) => {
                    debug!("No updates available");
                }
                Err(e) => {
                    error!("Update check failed: {}", e);
                    // Continue running, will retry next interval
                }
            }
        }
    }
}

/// Start the update manager as a background task
pub fn spawn_update_loop(config: UpdateConfig) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut manager = UpdateManager::new(config);

        if let Err(e) = manager.init() {
            error!("Failed to initialize update manager: {}", e);
            return;
        }

        manager.run_update_loop().await;
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let current = Version::parse("0.1.0").unwrap();
        let newer = Version::parse("0.2.0").unwrap();
        let older = Version::parse("0.0.9").unwrap();
        let same = Version::parse("0.1.0").unwrap();

        assert!(newer > current);
        assert!(older < current);
        assert!(same == current);
    }

    #[test]
    fn test_update_config_default() {
        let config = UpdateConfig::default();
        assert_eq!(config.check_interval, Duration::from_secs(3600));
        assert!(config.enabled);
    }

    #[test]
    fn test_hash_verification() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        let content = b"test content for hashing";
        temp_file.write_all(content).unwrap();
        temp_file.flush().unwrap();

        // Calculate expected hash
        let mut hasher = Sha256::new();
        hasher.update(content);
        let expected = hex::encode(hasher.finalize());

        let manager = UpdateManager::new(UpdateConfig::default());
        let result = manager.verify_hash(&temp_file.path().to_path_buf(), &expected);
        assert!(result.is_ok());
    }
}
