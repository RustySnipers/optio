//! Agent Update Server (Phoenix Hub Component)
//!
//! This module provides the Hub-side implementation of the agent update service.
//! It serves agent binaries and version information for the self-update mechanism.
//!
//! # Directory Structure
//! The update server expects binaries to be placed in the `updates/` directory:
//! ```
//! updates/
//! ├── manifest.json          # Version manifest with hashes
//! ├── windows/
//! │   └── x86_64/
//! │       └── optio-agent.exe
//! ├── linux/
//! │   └── x86_64/
//! │       └── optio-agent
//! └── macos/
//!     ├── x86_64/
//!     │   └── optio-agent
//!     └── aarch64/
//!         └── optio-agent
//! ```

use crate::db::Database;
use optio_core::proto::update_service_server::{UpdateService, UpdateServiceServer};
use optio_core::proto::{
    BinaryChunk, CheckVersionRequest, CheckVersionResponse,
    GetAgentBinaryRequest,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, warn};

/// Default chunk size for binary streaming (64KB)
const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Update manifest containing version info for all platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateManifest {
    /// Latest available version
    pub version: String,
    /// Release notes (markdown)
    pub release_notes: Option<String>,
    /// Whether this is a mandatory update
    pub mandatory: bool,
    /// Minimum version required (for breaking changes)
    pub minimum_version: Option<String>,
    /// Platform-specific binaries
    pub binaries: HashMap<String, PlatformBinary>,
}

/// Platform-specific binary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformBinary {
    /// Path to the binary (relative to updates directory)
    pub path: String,
    /// SHA256 hash of the binary
    pub sha256: String,
    /// Size in bytes
    pub size_bytes: u64,
}

impl UpdateManifest {
    /// Get binary info for a specific platform and architecture
    pub fn get_binary(&self, platform: &str, arch: &str) -> Option<&PlatformBinary> {
        let key = format!("{}-{}", platform, arch);
        self.binaries.get(&key)
    }
}

/// State for the update service
pub struct UpdateState {
    /// Directory containing update binaries
    pub updates_dir: PathBuf,
    /// Cached manifest
    pub manifest: Option<UpdateManifest>,
    /// Database for logging
    pub db: Arc<Database>,
}

impl UpdateState {
    /// Load or reload the manifest
    pub fn load_manifest(&mut self) -> Result<(), String> {
        let manifest_path = self.updates_dir.join("manifest.json");

        if !manifest_path.exists() {
            debug!("No update manifest found at {:?}", manifest_path);
            self.manifest = None;
            return Ok(());
        }

        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        let manifest: UpdateManifest = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

        info!(
            "Loaded update manifest: version {} with {} binaries",
            manifest.version,
            manifest.binaries.len()
        );

        self.manifest = Some(manifest);
        Ok(())
    }

    /// Get the binary path for a platform/arch
    pub fn get_binary_path(&self, platform: &str, arch: &str) -> Option<PathBuf> {
        let manifest = self.manifest.as_ref()?;
        let binary = manifest.get_binary(platform, arch)?;
        Some(self.updates_dir.join(&binary.path))
    }
}

/// Update service implementation
pub struct UpdateServiceImpl {
    state: Arc<RwLock<UpdateState>>,
}

impl UpdateServiceImpl {
    /// Create a new update service
    pub fn new(updates_dir: PathBuf, db: Arc<Database>) -> Self {
        let mut state = UpdateState {
            updates_dir,
            manifest: None,
            db,
        };

        // Load manifest on startup
        if let Err(e) = state.load_manifest() {
            warn!("Failed to load initial manifest: {}", e);
        }

        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }

    /// Reload the manifest (call after adding new updates)
    pub async fn reload_manifest(&self) -> Result<(), String> {
        let mut state = self.state.write().await;
        state.load_manifest()
    }

    /// Create the gRPC service
    pub fn into_service(self) -> UpdateServiceServer<Self> {
        UpdateServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl UpdateService for UpdateServiceImpl {
    async fn check_version(
        &self,
        request: Request<CheckVersionRequest>,
    ) -> Result<Response<CheckVersionResponse>, Status> {
        let req = request.into_inner();

        info!(
            machine_id = %req.machine_id,
            current_version = %req.current_version,
            platform = %req.platform,
            arch = %req.architecture,
            "Version check request"
        );

        let state = self.state.read().await;

        // Check if we have a manifest
        let manifest = match &state.manifest {
            Some(m) => m,
            None => {
                debug!("No update manifest available");
                return Ok(Response::new(CheckVersionResponse {
                    update_available: false,
                    latest_version: req.current_version.clone(),
                    sha256: String::new(),
                    size_bytes: 0,
                    release_notes: String::new(),
                    mandatory: false,
                    minimum_version: String::new(),
                }));
            }
        };

        // Check if platform/arch is supported
        let binary = match manifest.get_binary(&req.platform, &req.architecture) {
            Some(b) => b,
            None => {
                warn!(
                    "No binary available for {}-{}",
                    req.platform, req.architecture
                );
                return Ok(Response::new(CheckVersionResponse {
                    update_available: false,
                    latest_version: manifest.version.clone(),
                    sha256: String::new(),
                    size_bytes: 0,
                    release_notes: manifest.release_notes.clone().unwrap_or_default(),
                    mandatory: false,
                    minimum_version: String::new(),
                }));
            }
        };

        // Compare versions
        let update_available = match (
            semver::Version::parse(&req.current_version),
            semver::Version::parse(&manifest.version),
        ) {
            (Ok(current), Ok(latest)) => latest > current,
            _ => {
                // If we can't parse versions, fall back to string comparison
                manifest.version != req.current_version
            }
        };

        let response = CheckVersionResponse {
            update_available,
            latest_version: manifest.version.clone(),
            sha256: binary.sha256.clone(),
            size_bytes: binary.size_bytes,
            release_notes: manifest.release_notes.clone().unwrap_or_default(),
            mandatory: manifest.mandatory,
            minimum_version: manifest.minimum_version.clone().unwrap_or_default(),
        };

        if update_available {
            info!(
                machine_id = %req.machine_id,
                current = %req.current_version,
                latest = %manifest.version,
                "Update available for agent"
            );
        }

        Ok(Response::new(response))
    }

    type GetAgentBinaryStream =
        Pin<Box<dyn Stream<Item = Result<BinaryChunk, Status>> + Send + 'static>>;

    async fn get_agent_binary(
        &self,
        request: Request<GetAgentBinaryRequest>,
    ) -> Result<Response<Self::GetAgentBinaryStream>, Status> {
        let req = request.into_inner();

        info!(
            machine_id = %req.machine_id,
            platform = %req.platform,
            arch = %req.architecture,
            "Binary download request"
        );

        let state = self.state.read().await;

        // Get the binary path
        let binary_path = state
            .get_binary_path(&req.platform, &req.architecture)
            .ok_or_else(|| {
                Status::not_found(format!(
                    "No binary available for {}-{}",
                    req.platform, req.architecture
                ))
            })?;

        if !binary_path.exists() {
            error!("Binary file not found at {:?}", binary_path);
            return Err(Status::not_found("Binary file not found"));
        }

        // Read the entire file (for smaller binaries)
        // For production, consider streaming directly from disk
        let binary_data = fs::read(&binary_path).map_err(|e| {
            error!("Failed to read binary: {}", e);
            Status::internal("Failed to read binary file")
        })?;

        let total_size = binary_data.len() as u64;
        let total_chunks = (binary_data.len() + DEFAULT_CHUNK_SIZE - 1) / DEFAULT_CHUNK_SIZE;

        info!(
            machine_id = %req.machine_id,
            size = total_size,
            chunks = total_chunks,
            "Streaming binary to agent"
        );

        // Create a stream of chunks
        let (tx, rx) = tokio::sync::mpsc::channel(16);

        tokio::spawn(async move {
            for (i, chunk) in binary_data.chunks(DEFAULT_CHUNK_SIZE).enumerate() {
                // Calculate chunk hash
                let mut hasher = Sha256::new();
                hasher.update(chunk);
                let chunk_hash = hex::encode(hasher.finalize());

                let binary_chunk = BinaryChunk {
                    chunk_number: i as u32,
                    total_chunks: total_chunks as u32,
                    data: chunk.to_vec(),
                    chunk_hash,
                    total_size,
                };

                if tx.send(Ok(binary_chunk)).await.is_err() {
                    debug!("Client disconnected during download");
                    break;
                }
            }
        });

        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream)))
    }
}

/// HTTP endpoint handlers for REST API fallback
pub mod http {
    use super::*;
    use std::sync::Arc;

    /// JSON response for version check
    #[derive(Serialize)]
    pub struct VersionResponse {
        pub version: String,
        pub sha256: String,
        pub size_bytes: Option<u64>,
        pub release_notes: Option<String>,
    }

    /// Get version info as JSON (REST endpoint)
    pub async fn get_version(
        state: Arc<RwLock<UpdateState>>,
        platform: &str,
        arch: &str,
    ) -> Option<VersionResponse> {
        let state = state.read().await;
        let manifest = state.manifest.as_ref()?;
        let binary = manifest.get_binary(platform, arch)?;

        Some(VersionResponse {
            version: manifest.version.clone(),
            sha256: binary.sha256.clone(),
            size_bytes: Some(binary.size_bytes),
            release_notes: manifest.release_notes.clone(),
        })
    }

    /// Get binary file (REST endpoint)
    pub async fn get_binary(
        state: Arc<RwLock<UpdateState>>,
        platform: &str,
        arch: &str,
    ) -> Option<Vec<u8>> {
        let state = state.read().await;
        let path = state.get_binary_path(platform, arch)?;
        fs::read(&path).ok()
    }
}

/// Helper function to generate a manifest from existing binaries
pub fn generate_manifest(updates_dir: &Path, version: &str) -> Result<UpdateManifest, String> {
    let mut binaries = HashMap::new();

    let platforms = ["windows", "linux", "macos"];
    let architectures = ["x86_64", "aarch64"];

    for platform in &platforms {
        for arch in &architectures {
            let binary_name = if *platform == "windows" {
                "optio-agent.exe"
            } else {
                "optio-agent"
            };

            let binary_path = updates_dir.join(platform).join(arch).join(binary_name);

            if binary_path.exists() {
                let mut file = File::open(&binary_path)
                    .map_err(|e| format!("Failed to open {}: {}", binary_path.display(), e))?;

                let mut hasher = Sha256::new();
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)
                    .map_err(|e| format!("Failed to read {}: {}", binary_path.display(), e))?;

                hasher.update(&buffer);
                let sha256 = hex::encode(hasher.finalize());

                let relative_path = format!("{}/{}/{}", platform, arch, binary_name);
                let key = format!("{}-{}", platform, arch);

                binaries.insert(
                    key,
                    PlatformBinary {
                        path: relative_path,
                        sha256,
                        size_bytes: buffer.len() as u64,
                    },
                );

                info!("Found binary: {}/{}/{}", platform, arch, binary_name);
            }
        }
    }

    Ok(UpdateManifest {
        version: version.to_string(),
        release_notes: None,
        mandatory: false,
        minimum_version: None,
        binaries,
    })
}

/// Save manifest to file
pub fn save_manifest(manifest: &UpdateManifest, updates_dir: &Path) -> Result<(), String> {
    let manifest_path = updates_dir.join("manifest.json");
    let content = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

    fs::write(&manifest_path, content)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    info!("Saved manifest to {:?}", manifest_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_get_binary() {
        let mut binaries = HashMap::new();
        binaries.insert(
            "windows-x86_64".to_string(),
            PlatformBinary {
                path: "windows/x86_64/optio-agent.exe".to_string(),
                sha256: "abc123".to_string(),
                size_bytes: 1024,
            },
        );

        let manifest = UpdateManifest {
            version: "1.0.0".to_string(),
            release_notes: None,
            mandatory: false,
            minimum_version: None,
            binaries,
        };

        assert!(manifest.get_binary("windows", "x86_64").is_some());
        assert!(manifest.get_binary("linux", "x86_64").is_none());
    }

    #[test]
    fn test_manifest_serialization() {
        let mut binaries = HashMap::new();
        binaries.insert(
            "windows-x86_64".to_string(),
            PlatformBinary {
                path: "windows/x86_64/optio-agent.exe".to_string(),
                sha256: "abc123".to_string(),
                size_bytes: 1024,
            },
        );

        let manifest = UpdateManifest {
            version: "1.0.0".to_string(),
            release_notes: Some("Bug fixes".to_string()),
            mandatory: false,
            minimum_version: None,
            binaries,
        };

        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("1.0.0"));
        assert!(json.contains("windows-x86_64"));
    }
}
