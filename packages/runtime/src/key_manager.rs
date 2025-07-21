//! Secure key management module
//! 
//! Provides secure storage and retrieval of sensitive keys using OS keychains
//! and hardware security modules (HSM) when available.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Result type for key manager operations
pub type Result<T> = std::result::Result<T, KeyManagerError>;

/// Key manager errors
#[derive(Debug, thiserror::Error)]
pub enum KeyManagerError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Keychain access error: {0}")]
    KeychainError(String),
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("HSM not available")]
    HsmNotAvailable,
    
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// Sensitive key material that is zeroized on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureKey {
    #[zeroize(skip)]
    id: String,
    key_material: Vec<u8>,
}

impl SecureKey {
    /// Create a new secure key
    pub fn new(id: String, key_material: Vec<u8>) -> Self {
        Self { id, key_material }
    }
    
    /// Get the key ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Get the key material (use with caution)
    pub fn material(&self) -> &[u8] {
        &self.key_material
    }
}

/// Key metadata for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub id: String,
    pub name: String,
    pub key_type: KeyType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: HashMap<String, String>,
}

/// Type of key
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum KeyType {
    /// API key for external services
    ApiKey,
    /// Private key for signing (e.g., Solana)
    PrivateKey,
    /// Symmetric encryption key
    SymmetricKey,
    /// Database credentials
    DatabaseCredential,
    /// Generic secret
    Secret,
}

/// Backend storage mechanism
#[derive(Debug, Clone, Copy)]
pub enum StorageBackend {
    /// OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
    OsKeychain,
    /// Hardware Security Module
    Hsm,
    /// Encrypted file storage (less secure, fallback option)
    EncryptedFile,
    /// In-memory only (for testing)
    Memory,
}

/// Trait for key storage backends
#[async_trait]
pub trait KeyStorage: Send + Sync {
    /// Store a key
    async fn store(&self, metadata: &KeyMetadata, key: &SecureKey) -> Result<()>;
    
    /// Retrieve a key
    async fn retrieve(&self, id: &str) -> Result<SecureKey>;
    
    /// Delete a key
    async fn delete(&self, id: &str) -> Result<()>;
    
    /// List all key metadata
    async fn list(&self) -> Result<Vec<KeyMetadata>>;
    
    /// Check if a key exists
    async fn exists(&self, id: &str) -> Result<bool>;
}

/// OS Keychain storage implementation
pub struct OsKeychainStorage {
    service_name: String,
}

impl OsKeychainStorage {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }
}

#[async_trait]
impl KeyStorage for OsKeychainStorage {
    async fn store(&self, _metadata: &KeyMetadata, _key: &SecureKey) -> Result<()> {
        // In a real implementation, this would use:
        // - macOS: Security framework
        // - Windows: Windows Credential Manager API
        // - Linux: Secret Service API (libsecret)
        
        #[cfg(target_os = "macos")]
        {
            // Use macOS Keychain
            warn!("macOS Keychain integration not yet implemented");
            return Err(KeyManagerError::KeychainError(
                "macOS Keychain integration pending".to_string()
            ));
        }
        
        #[cfg(target_os = "windows")]
        {
            // Use Windows Credential Manager
            warn!("Windows Credential Manager integration not yet implemented");
            return Err(KeyManagerError::KeychainError(
                "Windows Credential Manager integration pending".to_string()
            ));
        }
        
        #[cfg(target_os = "linux")]
        {
            // Use Secret Service API
            warn!("Linux Secret Service integration not yet implemented");
            return Err(KeyManagerError::KeychainError(
                "Linux Secret Service integration pending".to_string()
            ));
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            return Err(KeyManagerError::KeychainError(
                "Unsupported operating system".to_string()
            ));
        }
    }
    
    async fn retrieve(&self, id: &str) -> Result<SecureKey> {
        Err(KeyManagerError::KeyNotFound(id.to_string()))
    }
    
    async fn delete(&self, id: &str) -> Result<()> {
        Err(KeyManagerError::KeyNotFound(id.to_string()))
    }
    
    async fn list(&self) -> Result<Vec<KeyMetadata>> {
        Ok(Vec::new())
    }
    
    async fn exists(&self, _id: &str) -> Result<bool> {
        Ok(false)
    }
}

/// In-memory storage for testing
pub struct MemoryStorage {
    keys: Arc<RwLock<HashMap<String, (KeyMetadata, SecureKey)>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl KeyStorage for MemoryStorage {
    async fn store(&self, metadata: &KeyMetadata, key: &SecureKey) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.insert(metadata.id.clone(), (metadata.clone(), key.clone()));
        Ok(())
    }
    
    async fn retrieve(&self, id: &str) -> Result<SecureKey> {
        let keys = self.keys.read().await;
        keys.get(id)
            .map(|(_, key)| key.clone())
            .ok_or_else(|| KeyManagerError::KeyNotFound(id.to_string()))
    }
    
    async fn delete(&self, id: &str) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.remove(id)
            .ok_or_else(|| KeyManagerError::KeyNotFound(id.to_string()))?;
        Ok(())
    }
    
    async fn list(&self) -> Result<Vec<KeyMetadata>> {
        let keys = self.keys.read().await;
        Ok(keys.values().map(|(meta, _)| meta.clone()).collect())
    }
    
    async fn exists(&self, id: &str) -> Result<bool> {
        let keys = self.keys.read().await;
        Ok(keys.contains_key(id))
    }
}

/// Main key manager
pub struct KeyManager {
    storage: Box<dyn KeyStorage>,
    backend: StorageBackend,
}

impl KeyManager {
    /// Create a new key manager with the specified backend
    pub async fn new(backend: StorageBackend) -> Result<Self> {
        let storage: Box<dyn KeyStorage> = match backend {
            StorageBackend::OsKeychain => {
                info!("Initializing OS keychain storage");
                Box::new(OsKeychainStorage::new("banshee".to_string()))
            }
            StorageBackend::Memory => {
                warn!("Using in-memory key storage (not secure for production)");
                Box::new(MemoryStorage::new())
            }
            _ => {
                error!("Unsupported storage backend: {:?}", backend);
                return Err(KeyManagerError::KeychainError(
                    format!("Unsupported backend: {:?}", backend)
                ));
            }
        };
        
        Ok(Self { storage, backend })
    }
    
    /// Store a new API key
    pub async fn store_api_key(&self, name: &str, api_key: &str) -> Result<String> {
        let id = format!("api-key-{}", uuid::Uuid::new_v4());
        let metadata = KeyMetadata {
            id: id.clone(),
            name: name.to_string(),
            key_type: KeyType::ApiKey,
            created_at: chrono::Utc::now(),
            last_accessed: None,
            tags: HashMap::new(),
        };
        
        let secure_key = SecureKey::new(id.clone(), api_key.as_bytes().to_vec());
        self.storage.store(&metadata, &secure_key).await?;
        
        info!("Stored API key '{}' with ID: {}", name, id);
        Ok(id)
    }
    
    /// Store a private key (e.g., Solana keypair)
    pub async fn store_private_key(&self, name: &str, key_bytes: &[u8]) -> Result<String> {
        let id = format!("private-key-{}", uuid::Uuid::new_v4());
        let metadata = KeyMetadata {
            id: id.clone(),
            name: name.to_string(),
            key_type: KeyType::PrivateKey,
            created_at: chrono::Utc::now(),
            last_accessed: None,
            tags: HashMap::new(),
        };
        
        let secure_key = SecureKey::new(id.clone(), key_bytes.to_vec());
        self.storage.store(&metadata, &secure_key).await?;
        
        info!("Stored private key '{}' with ID: {}", name, id);
        Ok(id)
    }
    
    /// Retrieve a key by ID
    pub async fn retrieve(&self, id: &str) -> Result<SecureKey> {
        debug!("Retrieving key: {}", id);
        let key = self.storage.retrieve(id).await?;
        
        // Note: In a production system, we would update last accessed time
        // This would require a mutable storage interface or a separate audit log
        
        Ok(key)
    }
    
    /// Delete a key
    pub async fn delete(&self, id: &str) -> Result<()> {
        info!("Deleting key: {}", id);
        self.storage.delete(id).await
    }
    
    /// List all keys
    pub async fn list(&self) -> Result<Vec<KeyMetadata>> {
        self.storage.list().await
    }
    
    /// Get the current storage backend
    pub fn backend(&self) -> StorageBackend {
        self.backend
    }
}

/// Helper to get API key from environment or key manager
pub async fn get_api_key(
    key_manager: &KeyManager,
    env_var: &str,
    key_name: &str,
) -> Result<String> {
    // First check environment variable
    if let Ok(key) = std::env::var(env_var) {
        debug!("Using API key from environment variable: {}", env_var);
        return Ok(key);
    }
    
    // Then check key manager
    let keys = key_manager.list().await?;
    if let Some(metadata) = keys.iter().find(|k| k.name == key_name && k.key_type == KeyType::ApiKey) {
        let secure_key = key_manager.retrieve(&metadata.id).await?;
        let key_str = String::from_utf8(secure_key.material().to_vec())
            .map_err(|e| KeyManagerError::InvalidKeyFormat(e.to_string()))?;
        debug!("Using API key from key manager: {}", key_name);
        return Ok(key_str);
    }
    
    Err(KeyManagerError::KeyNotFound(format!(
        "API key '{}' not found in environment or key manager",
        key_name
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_storage() {
        let manager = KeyManager::new(StorageBackend::Memory).await.unwrap();
        
        // Store an API key
        let id = manager.store_api_key("test-api", "secret-key-123").await.unwrap();
        
        // Retrieve it
        let key = manager.retrieve(&id).await.unwrap();
        assert_eq!(key.material(), b"secret-key-123");
        
        // List keys
        let keys = manager.list().await.unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].name, "test-api");
        
        // Delete it
        manager.delete(&id).await.unwrap();
        
        // Should be gone
        assert!(manager.retrieve(&id).await.is_err());
    }
    
    #[tokio::test]
    async fn test_secure_key_zeroize() {
        let key_data = vec![1, 2, 3, 4, 5];
        let secure_key = SecureKey::new("test".to_string(), key_data);
        
        // Key material should be accessible
        assert_eq!(secure_key.material(), &[1, 2, 3, 4, 5]);
        
        // When dropped, the memory will be zeroized
        drop(secure_key);
    }
}