//! Keypair persistence with AES-256-GCM encryption
//!
//! Provides secure storage for Hub signing keypairs. Private keys are
//! encrypted before storage using a derived key from the app's master secret.

use crate::crypto::{ExportedKeyPair, HubKeyPair};
use crate::db::Database;
use crate::error::{OptioError, OptioResult};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chrono::{DateTime, Utc};
use rand::RngCore;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Stored keypair record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredKeyPair {
    pub id: String,
    pub key_id: String,
    pub public_key: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
}

/// Repository for Hub keypair persistence
pub struct KeyPairRepository<'a> {
    db: &'a Database,
    encryption_key: [u8; 32],
}

impl<'a> KeyPairRepository<'a> {
    /// Create a new repository with the given encryption key
    ///
    /// The encryption key should be derived from a master secret
    /// (e.g., user password or hardware-derived key)
    pub fn new(db: &'a Database, master_secret: &[u8]) -> Self {
        // Derive a 256-bit encryption key from the master secret
        let mut hasher = Sha256::new();
        hasher.update(b"OPTIO-KEYPAIR-ENCRYPTION-V1");
        hasher.update(master_secret);
        let hash = hasher.finalize();

        let mut encryption_key = [0u8; 32];
        encryption_key.copy_from_slice(&hash);

        KeyPairRepository { db, encryption_key }
    }

    /// Create with a default key (for development/demo only)
    ///
    /// WARNING: In production, use a proper master secret
    pub fn new_with_default_key(db: &'a Database) -> Self {
        Self::new(db, b"OPTIO-DEFAULT-DEV-KEY-DO-NOT-USE-IN-PROD")
    }

    /// Save a keypair to the database
    ///
    /// The private key is encrypted with AES-256-GCM before storage
    pub fn save(&self, keypair: &HubKeyPair, key_id: &str) -> OptioResult<StoredKeyPair> {
        let conn = self
            .db
            .conn
            .lock()
            .map_err(|e| OptioError::Database(e.to_string()))?;

        // Encrypt the private key
        let private_key_bytes = hex::decode(keypair.private_key_hex())
            .map_err(|e| OptioError::Crypto(format!("Failed to decode private key: {}", e)))?;

        let (encrypted_data, nonce) = self.encrypt(&private_key_bytes)?;

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        // Deactivate any existing active keypairs
        conn.execute("UPDATE hub_keypairs SET is_active = 0", [])?;

        // Insert the new keypair
        conn.execute(
            "INSERT INTO hub_keypairs (id, key_id, public_key, encrypted_private_key, nonce, is_active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
            params![
                id,
                key_id,
                keypair.public_key_hex(),
                encrypted_data,
                nonce.to_vec(),
                now.to_rfc3339(),
            ],
        )?;

        tracing::info!("Saved Hub keypair: {}", key_id);

        Ok(StoredKeyPair {
            id,
            key_id: key_id.to_string(),
            public_key: keypair.public_key_hex(),
            is_active: true,
            created_at: now,
            rotated_at: None,
        })
    }

    /// Load the active keypair from the database
    pub fn load_active(&self) -> OptioResult<Option<(HubKeyPair, StoredKeyPair)>> {
        let conn = self
            .db
            .conn
            .lock()
            .map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, key_id, public_key, encrypted_private_key, nonce, created_at, rotated_at
             FROM hub_keypairs WHERE is_active = 1 LIMIT 1",
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let key_id: String = row.get(1)?;
            let public_key: String = row.get(2)?;
            let encrypted_data: Vec<u8> = row.get(3)?;
            let nonce_bytes: Vec<u8> = row.get(4)?;
            let created_at: String = row.get(5)?;
            let rotated_at: Option<String> = row.get(6)?;

            // Decrypt the private key
            let private_key_bytes = self.decrypt(&encrypted_data, &nonce_bytes)?;
            let private_key_hex = hex::encode(&private_key_bytes);

            // Reconstruct the keypair
            let keypair = HubKeyPair::from_private_key_hex(&private_key_hex)?;

            // Verify the public key matches
            if keypair.public_key_hex() != public_key {
                return Err(OptioError::Crypto(
                    "Public key mismatch after decryption - possible corruption".to_string(),
                ));
            }

            let stored = StoredKeyPair {
                id,
                key_id,
                public_key,
                is_active: true,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| OptioError::Database(e.to_string()))?
                    .with_timezone(&Utc),
                rotated_at: rotated_at
                    .map(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .flatten()
                    .map(|dt| dt.with_timezone(&Utc)),
            };

            tracing::debug!("Loaded active keypair: {}", stored.key_id);

            Ok(Some((keypair, stored)))
        } else {
            Ok(None)
        }
    }

    /// Load a keypair by key_id
    pub fn load_by_key_id(&self, key_id: &str) -> OptioResult<Option<(HubKeyPair, StoredKeyPair)>> {
        let conn = self
            .db
            .conn
            .lock()
            .map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, key_id, public_key, encrypted_private_key, nonce, is_active, created_at, rotated_at
             FROM hub_keypairs WHERE key_id = ?1",
        )?;

        let mut rows = stmt.query(params![key_id])?;

        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let key_id: String = row.get(1)?;
            let public_key: String = row.get(2)?;
            let encrypted_data: Vec<u8> = row.get(3)?;
            let nonce_bytes: Vec<u8> = row.get(4)?;
            let is_active: i32 = row.get(5)?;
            let created_at: String = row.get(6)?;
            let rotated_at: Option<String> = row.get(7)?;

            let private_key_bytes = self.decrypt(&encrypted_data, &nonce_bytes)?;
            let private_key_hex = hex::encode(&private_key_bytes);
            let keypair = HubKeyPair::from_private_key_hex(&private_key_hex)?;

            let stored = StoredKeyPair {
                id,
                key_id,
                public_key,
                is_active: is_active == 1,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| OptioError::Database(e.to_string()))?
                    .with_timezone(&Utc),
                rotated_at: rotated_at
                    .map(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .flatten()
                    .map(|dt| dt.with_timezone(&Utc)),
            };

            Ok(Some((keypair, stored)))
        } else {
            Ok(None)
        }
    }

    /// List all stored keypairs (metadata only, no private keys)
    pub fn list(&self) -> OptioResult<Vec<StoredKeyPair>> {
        let conn = self
            .db
            .conn
            .lock()
            .map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, key_id, public_key, is_active, created_at, rotated_at
             FROM hub_keypairs ORDER BY created_at DESC",
        )?;

        let mut keypairs = Vec::new();
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            let is_active: i32 = row.get(3)?;
            let created_at: String = row.get(4)?;
            let rotated_at: Option<String> = row.get(5)?;

            keypairs.push(StoredKeyPair {
                id: row.get(0)?,
                key_id: row.get(1)?,
                public_key: row.get(2)?,
                is_active: is_active == 1,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                rotated_at: rotated_at
                    .map(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .flatten()
                    .map(|dt| dt.with_timezone(&Utc)),
            });
        }

        Ok(keypairs)
    }

    /// Rotate to a new keypair (deactivates current, activates new)
    pub fn rotate(&self, new_keypair: &HubKeyPair, new_key_id: &str) -> OptioResult<StoredKeyPair> {
        let conn = self
            .db
            .conn
            .lock()
            .map_err(|e| OptioError::Database(e.to_string()))?;

        let now = Utc::now();

        // Mark current active keypair as rotated
        conn.execute(
            "UPDATE hub_keypairs SET is_active = 0, rotated_at = ?1 WHERE is_active = 1",
            params![now.to_rfc3339()],
        )?;

        drop(conn); // Release lock before calling save

        // Save the new keypair as active
        self.save(new_keypair, new_key_id)
    }

    /// Delete a keypair by key_id
    pub fn delete(&self, key_id: &str) -> OptioResult<bool> {
        let conn = self
            .db
            .conn
            .lock()
            .map_err(|e| OptioError::Database(e.to_string()))?;

        let deleted = conn.execute("DELETE FROM hub_keypairs WHERE key_id = ?1", params![key_id])?;

        if deleted > 0 {
            tracing::info!("Deleted keypair: {}", key_id);
        }

        Ok(deleted > 0)
    }

    /// Encrypt data with AES-256-GCM
    fn encrypt(&self, plaintext: &[u8]) -> OptioResult<(Vec<u8>, [u8; 12])> {
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| OptioError::Crypto(format!("Failed to create cipher: {}", e)))?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| OptioError::Crypto(format!("Encryption failed: {}", e)))?;

        Ok((ciphertext, nonce_bytes))
    }

    /// Decrypt data with AES-256-GCM
    fn decrypt(&self, ciphertext: &[u8], nonce_bytes: &[u8]) -> OptioResult<Vec<u8>> {
        if nonce_bytes.len() != 12 {
            return Err(OptioError::Crypto("Invalid nonce length".to_string()));
        }

        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| OptioError::Crypto(format!("Failed to create cipher: {}", e)))?;

        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| OptioError::Crypto(format!("Decryption failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_db() -> Database {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        let db = Database::open(&path).unwrap();
        db.init_schema().unwrap();
        db
    }

    #[test]
    fn test_save_and_load_keypair() {
        let db = create_test_db();
        let repo = KeyPairRepository::new_with_default_key(&db);

        let keypair = HubKeyPair::generate();
        let saved = repo.save(&keypair, "test-key-1").unwrap();

        assert_eq!(saved.key_id, "test-key-1");
        assert!(saved.is_active);

        let (loaded_kp, loaded_meta) = repo.load_active().unwrap().unwrap();
        assert_eq!(loaded_kp.public_key_hex(), keypair.public_key_hex());
        assert_eq!(loaded_meta.key_id, "test-key-1");
    }

    #[test]
    fn test_keypair_rotation() {
        let db = create_test_db();
        let repo = KeyPairRepository::new_with_default_key(&db);

        let keypair1 = HubKeyPair::generate();
        repo.save(&keypair1, "key-v1").unwrap();

        let keypair2 = HubKeyPair::generate();
        repo.rotate(&keypair2, "key-v2").unwrap();

        let (active_kp, active_meta) = repo.load_active().unwrap().unwrap();
        assert_eq!(active_kp.public_key_hex(), keypair2.public_key_hex());
        assert_eq!(active_meta.key_id, "key-v2");

        // Old key should still be loadable
        let (old_kp, old_meta) = repo.load_by_key_id("key-v1").unwrap().unwrap();
        assert_eq!(old_kp.public_key_hex(), keypair1.public_key_hex());
        assert!(!old_meta.is_active);
        assert!(old_meta.rotated_at.is_some());
    }

    #[test]
    fn test_list_keypairs() {
        let db = create_test_db();
        let repo = KeyPairRepository::new_with_default_key(&db);

        repo.save(&HubKeyPair::generate(), "key-1").unwrap();
        repo.save(&HubKeyPair::generate(), "key-2").unwrap();
        repo.save(&HubKeyPair::generate(), "key-3").unwrap();

        let list = repo.list().unwrap();
        assert_eq!(list.len(), 3);

        // Only the last one should be active
        let active_count = list.iter().filter(|k| k.is_active).count();
        assert_eq!(active_count, 1);
    }
}
