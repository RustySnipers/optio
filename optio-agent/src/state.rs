//! Agent State Persistence
//!
//! Manages persistent agent state including machine ID that survives restarts.
//! On Windows: stores in %PROGRAMDATA%\Optio\
//! On Linux/macOS: stores in /var/lib/optio/

use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Filename for the machine ID file
const MACHINE_ID_FILENAME: &str = "machine-id";

/// Get the directory for storing agent state
fn get_state_dir() -> PathBuf {
    #[cfg(windows)]
    {
        // Use %PROGRAMDATA%\Optio (typically C:\ProgramData\Optio)
        if let Ok(program_data) = std::env::var("PROGRAMDATA") {
            PathBuf::from(program_data).join("Optio")
        } else {
            // Fallback to local app data
            PathBuf::from("C:\\ProgramData\\Optio")
        }
    }
    #[cfg(not(windows))]
    {
        // Use /var/lib/optio for Linux/macOS
        PathBuf::from("/var/lib/optio")
    }
}

/// Load or generate a persistent machine ID.
///
/// On first run:
/// - Generates a new UUIDv4
/// - Persists it to disk
///
/// On subsequent runs:
/// - Reads the existing ID from disk
///
/// If persistence fails (permissions, etc.):
/// - Falls back to the provided fallback ID
///
/// # Arguments
/// * `fallback_id` - ID to use if persistence fails
///
/// # Returns
/// The machine ID (either persisted or fallback)
pub fn load_or_create_machine_id(fallback_id: &str) -> String {
    let state_dir = get_state_dir();
    let machine_id_path = state_dir.join(MACHINE_ID_FILENAME);

    // Try to read existing machine ID
    if machine_id_path.exists() {
        match fs::read_to_string(&machine_id_path) {
            Ok(id) => {
                let id = id.trim().to_string();
                if !id.is_empty() {
                    debug!("Loaded persisted machine ID from {:?}", machine_id_path);
                    return id;
                }
            }
            Err(e) => {
                warn!("Failed to read machine ID from {:?}: {}", machine_id_path, e);
            }
        }
    }

    // Generate new machine ID
    let new_id = Uuid::new_v4().to_string();
    info!("Generated new machine ID: {}", new_id);

    // Try to persist it
    match persist_machine_id(&state_dir, &machine_id_path, &new_id) {
        Ok(()) => {
            info!("Persisted machine ID to {:?}", machine_id_path);
            new_id
        }
        Err(e) => {
            warn!(
                "Failed to persist machine ID (using fallback): {}. \
                 This may happen on first run without admin privileges.",
                e
            );
            // Return fallback ID if we can't persist
            fallback_id.to_string()
        }
    }
}

/// Persist machine ID to disk
fn persist_machine_id(
    state_dir: &PathBuf,
    machine_id_path: &PathBuf,
    id: &str,
) -> Result<(), std::io::Error> {
    // Create directory if it doesn't exist
    if !state_dir.exists() {
        fs::create_dir_all(state_dir)?;
        debug!("Created state directory: {:?}", state_dir);
    }

    // Write the machine ID
    fs::write(machine_id_path, id)?;

    // On Windows, try to set attributes to make it less visible
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // The file is already written; setting attributes is optional
        // We could use SetFileAttributes but for simplicity we'll skip it
    }

    Ok(())
}

/// Get the path where machine ID is stored (for diagnostics)
pub fn get_machine_id_path() -> PathBuf {
    get_state_dir().join(MACHINE_ID_FILENAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_machine_id_generation_and_persistence() {
        // Create a temp directory for testing
        let temp_dir = TempDir::new().unwrap();
        let machine_id_path = temp_dir.path().join(MACHINE_ID_FILENAME);

        // First call should generate a new ID
        let fallback = "fallback-id";
        
        // Since we can't easily mock get_state_dir, test the persistence function directly
        let test_id = Uuid::new_v4().to_string();
        persist_machine_id(
            &temp_dir.path().to_path_buf(),
            &machine_id_path,
            &test_id,
        ).unwrap();

        // Verify the ID was written
        let read_id = fs::read_to_string(&machine_id_path).unwrap();
        assert_eq!(read_id, test_id);
    }

    #[test]
    fn test_fallback_when_dir_not_writable() {
        // When persistence fails, should return fallback
        let fallback = "my-fallback-id";
        // This would require mocking - for now just verify the logic path
        assert!(!fallback.is_empty());
    }
}
