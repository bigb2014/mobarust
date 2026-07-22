//! Credential vault: master-password encrypted secret store.
//!
//! Uses Argon2id for key derivation and AES-256-GCM for encryption.
//! Secrets are zeroized on drop.

use std::path::Path;

use aes_gcm::aead::{Aead, KeyInit};
#[allow(deprecated)]
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::Argon2;
use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroize;

/// Errors that can occur in the vault.
#[derive(Debug, Error)]
pub enum VaultError {
    /// Key derivation failed.
    #[error("kdf error: {0}")]
    KdfError(String),
    /// Encryption failed.
    #[error("encrypt error: {0}")]
    EncryptError(String),
    /// Decryption failed (wrong password or corrupted data).
    #[error("decrypt error: {0}")]
    DecryptError(String),
    /// Wrong master password.
    #[error("wrong master password")]
    WrongPassword,
    /// IO error.
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

/// A secret that is zeroized on drop.
#[derive(Clone, Zeroize)]
pub struct Secret {
    data: Vec<u8>,
}

impl Secret {
    /// Creates a new secret from raw bytes.
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Returns the secret bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Consumes the secret and returns the raw bytes.
    #[must_use]
    pub fn into_bytes(mut self) -> Vec<u8> {
        let data = std::mem::take(&mut self.data);
        // Don't zeroize since we're returning the data.
        data
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

/// A vault entry storing an encrypted secret.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VaultEntry {
    /// Unique identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Secret type (password/key/token).
    pub secret_type: String,
    /// Target host or service (optional).
    pub target: Option<String>,
    /// Username (optional).
    pub username: Option<String>,
    /// AES-GCM ciphertext (nonce prepended).
    pub encrypted_value: Vec<u8>,
}

/// The credential vault.
pub struct Vault {
    /// Derived encryption key (zeroized on drop).
    key: Secret,
    /// All stored entries.
    entries: Vec<VaultEntry>,
    /// Salt used for key derivation.
    salt: Vec<u8>,
}

impl Vault {
    /// Creates a new vault with the given master password.
    ///
    /// # Errors
    /// Returns `VaultError::KdfError` if key derivation fails.
    pub fn new(master_password: &str) -> Result<Self, VaultError> {
        let salt = generate_salt();
        let key = derive_key(master_password, &salt)?;
        Ok(Self {
            key,
            entries: Vec::new(),
            salt,
        })
    }

    /// Creates a vault with an existing salt (for loading).
    fn with_salt(master_password: &str, salt: Vec<u8>) -> Result<Self, VaultError> {
        let key = derive_key(master_password, &salt)?;
        Ok(Self {
            key,
            entries: Vec::new(),
            salt,
        })
    }

    /// Adds an entry with an encrypted secret.
    ///
    /// # Errors
    /// Returns `VaultError::EncryptError` if encryption fails.
    pub fn add_entry(
        &mut self,
        entry_id: &str,
        name: &str,
        secret_type: &str,
        plaintext: &[u8],
        target: Option<&str>,
        username: Option<&str>,
    ) -> Result<(), VaultError> {
        let encrypted = encrypt(&self.key, plaintext)?;
        let entry = VaultEntry {
            id: entry_id.to_string(),
            name: name.to_string(),
            secret_type: secret_type.to_string(),
            target: target.map(String::from),
            username: username.map(String::from),
            encrypted_value: encrypted,
        };
        self.entries.push(entry);
        Ok(())
    }

    /// Returns an entry by id.
    #[must_use]
    pub fn get_entry(&self, entry_id: &str) -> Option<&VaultEntry> {
        self.entries.iter().find(|e| e.id == entry_id)
    }

    /// Decrypts an entry's secret value.
    ///
    /// # Errors
    /// Returns `VaultError::DecryptError` if decryption fails (wrong password).
    pub fn decrypt_entry(&self, entry_id: &str) -> Result<Secret, VaultError> {
        let entry = self
            .get_entry(entry_id)
            .ok_or_else(|| VaultError::DecryptError("entry not found".to_string()))?;
        decrypt(&self.key, &entry.encrypted_value)
    }

    /// Removes an entry by id.
    pub fn remove_entry(&mut self, entry_id: &str) -> bool {
        let len = self.entries.len();
        self.entries.retain(|e| e.id != entry_id);
        self.entries.len() < len
    }

    /// Returns all entry IDs.
    #[must_use]
    pub fn list_entries(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.id.as_str()).collect()
    }

    /// Saves the vault (entries + salt, no key) to a file.
    ///
    /// # Errors
    /// Returns `VaultError::IoError` on write failure.
    pub fn save_to_path(&self, path: &Path) -> Result<(), VaultError> {
        #[derive(Serialize)]
        struct VaultFile {
            salt: Vec<u8>,
            entries: Vec<VaultEntry>,
        }
        let file = VaultFile {
            salt: self.salt.clone(),
            entries: self.entries.clone(),
        };
        let json = serde_json::to_string_pretty(&file)
            .map_err(|e| VaultError::EncryptError(e.to_string()))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Loads a vault from a file using the master password.
    ///
    /// # Errors
    /// Returns `VaultError::WrongPassword` if the password is wrong,
    /// or `VaultError::IoError` on read failure.
    pub fn load_from_path(path: &Path, master_password: &str) -> Result<Self, VaultError> {
        #[derive(Deserialize)]
        struct VaultFile {
            salt: Vec<u8>,
            entries: Vec<VaultEntry>,
        }
        let contents = std::fs::read_to_string(path)?;
        let file: VaultFile =
            serde_json::from_str(&contents).map_err(|e| VaultError::DecryptError(e.to_string()))?;
        let mut vault = Self::with_salt(master_password, file.salt)?;
        // Verify password by trying to decrypt the first entry (if any).
        if let Some(first) = file.entries.first() {
            let _ = decrypt(&vault.key, &first.encrypted_value)
                .map_err(|_| VaultError::WrongPassword)?;
        }
        vault.entries = file.entries;
        Ok(vault)
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        // Key is a Secret, which zeroizes on drop automatically.
    }
}

/// Generates a random 16-byte salt.
fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; 16];
    rand::rng().fill_bytes(&mut salt);
    salt
}

/// Derives a 32-byte key from a password and salt using Argon2id.
fn derive_key(password: &str, salt: &[u8]) -> Result<Secret, VaultError> {
    let argon2 = Argon2::default();
    let mut key = vec![0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| VaultError::KdfError(e.to_string()))?;
    Ok(Secret::new(key))
}

#[allow(deprecated)]
/// Encrypts plaintext with AES-256-GCM. Returns nonce + ciphertext.
fn encrypt(key: &Secret, plaintext: &[u8]) -> Result<Vec<u8>, VaultError> {
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|e| VaultError::EncryptError(e.to_string()))?;
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), plaintext)
        .map_err(|e| VaultError::EncryptError(e.to_string()))?;
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

#[allow(deprecated)]
/// Decrypts AES-256-GCM ciphertext (nonce prepended).
fn decrypt(key: &Secret, data: &[u8]) -> Result<Secret, VaultError> {
    if data.len() < 13 {
        return Err(VaultError::DecryptError("ciphertext too short".to_string()));
    }
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|e| VaultError::DecryptError(e.to_string()))?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&data[..12]), &data[12..])
        .map_err(|e| VaultError::DecryptError(e.to_string()))?;
    Ok(Secret::new(plaintext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_vault_works() {
        let vault = Vault::new("masterpass").unwrap();
        assert!(vault.list_entries().is_empty());
    }

    #[test]
    fn add_and_decrypt_entry() {
        let mut vault = Vault::new("masterpass").unwrap();
        vault
            .add_entry("e1", "My Password", "password", b"secret123", None, None)
            .unwrap();
        let decrypted = vault.decrypt_entry("e1").unwrap();
        assert_eq!(decrypted.as_bytes(), b"secret123");
    }

    #[test]
    fn wrong_password_fails_to_decrypt() {
        let mut vault = Vault::new("passwordA").unwrap();
        vault
            .add_entry("e1", "test", "password", b"secret", None, None)
            .unwrap();
        // Save with passwordA
        let tmp = std::env::temp_dir().join("mobarust_vault_test.json");
        vault.save_to_path(&tmp).unwrap();
        // Try to load with passwordB
        let result = Vault::load_from_path(&tmp, "passwordB");
        assert!(result.is_err());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn remove_entry_works() {
        let mut vault = Vault::new("masterpass").unwrap();
        vault
            .add_entry("e1", "test", "password", b"secret", None, None)
            .unwrap();
        assert!(vault.remove_entry("e1"));
        assert!(vault.get_entry("e1").is_none());
        assert!(!vault.remove_entry("nonexistent"));
    }

    #[test]
    fn list_entries_works() {
        let mut vault = Vault::new("masterpass").unwrap();
        vault
            .add_entry("e1", "a", "password", b"s1", None, None)
            .unwrap();
        vault
            .add_entry("e2", "b", "key", b"s2", None, None)
            .unwrap();
        let ids = vault.list_entries();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"e1"));
        assert!(ids.contains(&"e2"));
    }

    #[test]
    fn save_load_round_trip() {
        let tmp = std::env::temp_dir().join("mobarust_vault_rt.json");
        let mut vault = Vault::new("masterpass").unwrap();
        vault
            .add_entry("e1", "test", "password", b"mysecret", None, None)
            .unwrap();
        vault.save_to_path(&tmp).unwrap();
        let loaded = Vault::load_from_path(&tmp, "masterpass").unwrap();
        let decrypted = loaded.decrypt_entry("e1").unwrap();
        assert_eq!(decrypted.as_bytes(), b"mysecret");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn at_rest_ciphertext_has_no_plaintext() {
        let mut vault = Vault::new("masterpass").unwrap();
        let plaintext = b"sensitive_data_12345";
        vault
            .add_entry("e1", "test", "password", plaintext, None, None)
            .unwrap();
        let entry = vault.get_entry("e1").unwrap();
        let ct = &entry.encrypted_value;
        assert!(!ct.windows(plaintext.len()).any(|w| w == plaintext));
    }
}
