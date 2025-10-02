use crate::{Error, Result};
use async_trait::async_trait;
use std::time::Duration;

/// State management trait
#[async_trait]
pub trait StateManager: Send + Sync {
    /// Get a value by key
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Set a value with optional TTL
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<()>;

    /// Delete a key
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool>;
}

/// Sled-backed state manager
pub struct SledStateManager {
    db: sled::Db,
}

impl SledStateManager {
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path).map_err(|e| Error::Handler(format!("Sled open failed: {}", e)))?;
        Ok(Self { db })
    }
}

#[async_trait]
impl StateManager for SledStateManager {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let value = self
            .db
            .get(key)
            .map_err(|e| Error::Handler(format!("Sled get failed: {}", e)))?;

        Ok(value.map(|v| v.to_vec()))
    }

    async fn set(&self, key: &str, value: Vec<u8>, _ttl: Option<Duration>) -> Result<()> {
        self.db
            .insert(key, value)
            .map_err(|e| Error::Handler(format!("Sled insert failed: {}", e)))?;

        self.db
            .flush()
            .map_err(|e| Error::Handler(format!("Sled flush failed: {}", e)))?;

        // TODO: Implement TTL with background task
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.db
            .remove(key)
            .map_err(|e| Error::Handler(format!("Sled remove failed: {}", e)))?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let exists = self
            .db
            .contains_key(key)
            .map_err(|e| Error::Handler(format!("Sled contains_key failed: {}", e)))?;
        Ok(exists)
    }
}

/// In-memory state manager for testing
pub struct MemoryStateManager {
    store: dashmap::DashMap<String, Vec<u8>>,
}

impl MemoryStateManager {
    pub fn new() -> Self {
        Self {
            store: dashmap::DashMap::new(),
        }
    }
}

impl Default for MemoryStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StateManager for MemoryStateManager {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.store.get(key).map(|v| v.clone()))
    }

    async fn set(&self, key: &str, value: Vec<u8>, _ttl: Option<Duration>) -> Result<()> {
        self.store.insert(key.to_string(), value);
        // TODO: Implement TTL with tokio::time
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.store.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.store.contains_key(key))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_state_basic() {
        let state = MemoryStateManager::new();

        // Set and get
        state.set("key1", b"value1".to_vec(), None).await.unwrap();
        let value = state.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        // Exists
        assert!(state.exists("key1").await.unwrap());
        assert!(!state.exists("key2").await.unwrap());

        // Delete
        state.delete("key1").await.unwrap();
        assert!(!state.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_sled_state_basic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let state = SledStateManager::new(temp_dir.path().to_str().unwrap()).unwrap();

        // Set and get
        state.set("key1", b"value1".to_vec(), None).await.unwrap();
        let value = state.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        // Persistence test - reopen
        drop(state);
        let state = SledStateManager::new(temp_dir.path().to_str().unwrap()).unwrap();
        let value = state.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));
    }
}
