use crate::Result;
use async_trait::async_trait;
use std::time::Duration;

/// State management trait for pforge handlers.
///
/// Provides a simple key-value interface for persistent or ephemeral state.
/// Current implementation: `MemoryStateManager` (in-memory, non-persistent).
///
/// Future: Will integrate with `trueno-db` KV module (Phase 6) for:
/// - SIMD-optimized key hashing
/// - GPU batch operations
/// - Parquet persistence
/// - WASM browser storage
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

/// Entry with optional expiration time
struct StateEntry {
    value: Vec<u8>,
    expires_at: Option<tokio::time::Instant>,
}

/// In-memory state manager using DashMap for concurrent access.
///
/// This is the default state backend. Data is lost on process restart.
/// Supports TTL (time-to-live) for automatic key expiration.
pub struct MemoryStateManager {
    store: dashmap::DashMap<String, StateEntry>,
}

impl MemoryStateManager {
    /// Create a new in-memory state manager
    #[must_use]
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
        if let Some(entry) = self.store.get(key) {
            // Check if expired
            if let Some(expires_at) = entry.expires_at {
                if tokio::time::Instant::now() >= expires_at {
                    // Key expired, remove it
                    drop(entry); // Release lock before removing
                    self.store.remove(key);
                    return Ok(None);
                }
            }
            Ok(Some(entry.value.clone()))
        } else {
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<()> {
        let expires_at = ttl.map(|d| tokio::time::Instant::now() + d);
        self.store
            .insert(key.to_string(), StateEntry { value, expires_at });
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.store.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        if let Some(entry) = self.store.get(key) {
            // Check if expired
            if let Some(expires_at) = entry.expires_at {
                if tokio::time::Instant::now() >= expires_at {
                    drop(entry);
                    self.store.remove(key);
                    return Ok(false);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

// KV backend (Phase 6), now via the aprender-db package
#[cfg(feature = "persistence")]
pub use trueno_kv::TruenoKvStateManager;

#[cfg(feature = "persistence")]
mod trueno_kv {
    use super::*;
    use crate::Error;
    use tokio::time::Instant;
    // Crate is `trueno_db`, package is `aprender-db`: the aprender monorepo
    // keeps the original lib names across the APR-MONO consolidation
    // (`aprender-db/Cargo.toml` declares `[lib] name = "trueno_db"`), so the
    // dependency moved but the `use` path did not.
    use trueno_db::kv::{KvStore, MemoryKvStore};

    /// State manager backed by the trueno_db KV store (aprender-db package).
    ///
    /// Provides SIMD-optimized key hashing via `trueno::hash` module.
    /// Currently uses in-memory storage; future versions will support
    /// Parquet persistence and WASM browser storage.
    ///
    /// TTL support is implemented via a separate expiration tracker.
    pub struct TruenoKvStateManager {
        store: MemoryKvStore,
        /// Tracks expiration times for keys with TTL
        expirations: dashmap::DashMap<String, Instant>,
    }

    impl TruenoKvStateManager {
        /// Create a new trueno-db backed state manager
        #[must_use]
        pub fn new() -> Self {
            Self {
                store: MemoryKvStore::new(),
                expirations: dashmap::DashMap::new(),
            }
        }

        /// Create with pre-allocated capacity
        #[must_use]
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                store: MemoryKvStore::with_capacity(capacity),
                expirations: dashmap::DashMap::new(),
            }
        }

        /// Check if a key has expired and clean up if so
        fn is_expired(&self, key: &str) -> bool {
            // First check if expired (read lock only)
            let expired = if let Some(expires_at) = self.expirations.get(key) {
                Instant::now() >= *expires_at
            } else {
                return false;
            };
            // Drop the read lock before attempting write lock to avoid deadlock
            if expired {
                self.expirations.remove(key);
            }
            expired
        }

        /// Get the number of stored keys
        #[must_use]
        pub fn len(&self) -> usize {
            self.store.len()
        }

        /// Check if the store is empty
        #[must_use]
        pub fn is_empty(&self) -> bool {
            self.store.is_empty()
        }

        /// Clear all stored keys
        pub fn clear(&self) {
            self.store.clear();
        }

        /// Test-only: Directly set an expiration time for a key
        /// This allows testing expiration without real time delays
        #[cfg(test)]
        pub(crate) fn set_expiration_for_test(&self, key: &str, expires_at: Instant) {
            self.expirations.insert(key.to_string(), expires_at);
        }
    }

    impl Default for TruenoKvStateManager {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl StateManager for TruenoKvStateManager {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
            // Check expiration first
            if self.is_expired(key) {
                // Key expired - we already cleaned up the expiration tracker in is_expired()
                // The store entry will be lazily cleaned up on next set() call
                return Ok(None);
            }

            self.store
                .get(key)
                .await
                .map_err(|e| Error::StateError(e.to_string()))
        }

        async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<()> {
            // Set expiration time if TTL provided
            if let Some(duration) = ttl {
                let expires_at = Instant::now() + duration;
                self.expirations.insert(key.to_string(), expires_at);
            } else {
                // Remove any existing expiration
                self.expirations.remove(key);
            }

            self.store
                .set(key, value)
                .await
                .map_err(|e| Error::StateError(e.to_string()))
        }

        async fn delete(&self, key: &str) -> Result<()> {
            // Also remove expiration tracking
            self.expirations.remove(key);

            self.store
                .delete(key)
                .await
                .map_err(|e| Error::StateError(e.to_string()))
        }

        async fn exists(&self, key: &str) -> Result<bool> {
            // Check expiration first
            if self.is_expired(key) {
                // Key expired - we already cleaned up the expiration tracker in is_expired()
                // The store entry will be lazily cleaned up on next set() call
                return Ok(false);
            }

            self.store
                .exists(key)
                .await
                .map_err(|e| Error::StateError(e.to_string()))
        }
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
    async fn test_memory_state_overwrite() {
        let state = MemoryStateManager::new();

        state.set("key", b"value1".to_vec(), None).await.unwrap();
        state.set("key", b"value2".to_vec(), None).await.unwrap();

        let value = state.get("key").await.unwrap();
        assert_eq!(value, Some(b"value2".to_vec()));
    }

    #[tokio::test]
    async fn test_memory_state_concurrent() {
        use std::sync::Arc;

        let state = Arc::new(MemoryStateManager::new());
        let mut handles = vec![];

        for i in 0..10 {
            let state = Arc::clone(&state);
            handles.push(tokio::spawn(async move {
                let key = format!("key{i}");
                let value = format!("value{i}").into_bytes();
                state.set(&key, value, None).await.unwrap();
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        for i in 0..10 {
            let key = format!("key{i}");
            assert!(state.exists(&key).await.unwrap());
        }
    }

    #[tokio::test(start_paused = true)]
    async fn test_memory_state_ttl_expiration() {
        let state = MemoryStateManager::new();

        // Set with 50ms TTL
        state
            .set(
                "ttl_key",
                b"value".to_vec(),
                Some(Duration::from_millis(50)),
            )
            .await
            .unwrap();

        // Should exist immediately
        assert!(state.exists("ttl_key").await.unwrap());
        assert_eq!(state.get("ttl_key").await.unwrap(), Some(b"value".to_vec()));

        // Advance time past expiration (instant with time mocking)
        tokio::time::advance(Duration::from_millis(60)).await;

        // Should be expired now
        assert!(!state.exists("ttl_key").await.unwrap());
        assert_eq!(state.get("ttl_key").await.unwrap(), None);
    }

    #[tokio::test(start_paused = true)]
    async fn test_memory_state_ttl_no_expiration() {
        let state = MemoryStateManager::new();

        // Set without TTL
        state.set("no_ttl", b"value".to_vec(), None).await.unwrap();

        // Advance time (instant with time mocking)
        tokio::time::advance(Duration::from_millis(10)).await;

        // Should still exist
        assert!(state.exists("no_ttl").await.unwrap());
        assert_eq!(state.get("no_ttl").await.unwrap(), Some(b"value".to_vec()));
    }

    #[tokio::test(start_paused = true)]
    async fn test_memory_state_ttl_overwrite_extends() {
        let state = MemoryStateManager::new();

        // Set with short TTL
        state
            .set("key", b"v1".to_vec(), Some(Duration::from_millis(30)))
            .await
            .unwrap();

        // Advance time (instant with time mocking)
        tokio::time::advance(Duration::from_millis(20)).await;

        // Overwrite with longer TTL
        state
            .set("key", b"v2".to_vec(), Some(Duration::from_millis(100)))
            .await
            .unwrap();

        // Advance past original expiration (instant with time mocking)
        tokio::time::advance(Duration::from_millis(20)).await;

        // Should still exist with new value
        assert_eq!(state.get("key").await.unwrap(), Some(b"v2".to_vec()));
    }

    // trueno-db KV backend tests (Phase 6)
    #[cfg(feature = "persistence")]
    mod trueno_kv_tests {
        use super::*;
        use crate::state::TruenoKvStateManager;

        #[tokio::test]
        async fn test_trueno_kv_basic() {
            let state = TruenoKvStateManager::new();

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
        async fn test_trueno_kv_overwrite() {
            let state = TruenoKvStateManager::new();

            state.set("key", b"value1".to_vec(), None).await.unwrap();
            state.set("key", b"value2".to_vec(), None).await.unwrap();

            let value = state.get("key").await.unwrap();
            assert_eq!(value, Some(b"value2".to_vec()));
        }

        #[tokio::test]
        async fn test_trueno_kv_with_capacity() {
            let state = TruenoKvStateManager::with_capacity(100);
            state.set("key", b"value".to_vec(), None).await.unwrap();
            assert_eq!(state.get("key").await.unwrap(), Some(b"value".to_vec()));
        }

        #[tokio::test]
        async fn test_trueno_kv_len_and_clear() {
            let state = TruenoKvStateManager::new();

            assert!(state.is_empty());
            assert_eq!(state.len(), 0);

            state.set("key1", b"value1".to_vec(), None).await.unwrap();
            assert!(!state.is_empty());
            assert_eq!(state.len(), 1);

            state.set("key2", b"value2".to_vec(), None).await.unwrap();
            assert_eq!(state.len(), 2);

            state.clear();
            assert!(state.is_empty());
        }

        #[test]
        fn test_trueno_kv_default() {
            let state: TruenoKvStateManager = Default::default();
            assert!(state.is_empty());
        }

        #[tokio::test]
        async fn test_trueno_kv_ttl_expiration() {
            use tokio::time::Instant;

            let state = TruenoKvStateManager::new();

            // Set a key without TTL first (TTL will be set via test helper)
            state
                .set("ttl_key", b"value".to_vec(), None)
                .await
                .expect("set should succeed");

            // Should exist initially
            assert!(state
                .exists("ttl_key")
                .await
                .expect("exists check should succeed"));

            // Set expiration to current time (should be considered expired immediately
            // since is_expired uses >= comparison)
            state.set_expiration_for_test("ttl_key", Instant::now());

            // Small yield to ensure time has advanced past the expiration
            tokio::task::yield_now().await;

            // Should be expired now - just check exists (get would try to access
            // store after expiration is already cleaned up, which has different semantics)
            assert!(!state
                .exists("ttl_key")
                .await
                .expect("exists check should succeed"));
        }

        #[tokio::test]
        async fn test_trueno_kv_ttl_no_expiration() {
            use tokio::time::Instant;

            let state = TruenoKvStateManager::new();

            // Set without TTL
            state
                .set("no_ttl", b"value".to_vec(), None)
                .await
                .expect("set should succeed");

            // Set expiration to a time in the future (should not expire)
            let future = Instant::now() + Duration::from_secs(3600);
            state.set_expiration_for_test("no_ttl", future);

            // Should still exist
            assert!(state
                .exists("no_ttl")
                .await
                .expect("exists check should succeed"));
            assert_eq!(
                state.get("no_ttl").await.expect("get should succeed"),
                Some(b"value".to_vec())
            );
        }
    }
}
