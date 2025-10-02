# Chapter 10: State Management Deep Dive

State management in pforge provides persistent and in-memory storage for your MCP tools. This chapter explores the state management system architecture, backends, and best practices.

## State Management Architecture

pforge provides a `StateManager` trait that abstracts different storage backends:

```rust
#[async_trait]
pub trait StateManager: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}
```

## State Backends

### 1. Sled (Persistent Storage)

**Use case:** Production servers requiring persistence across restarts

```yaml
state:
  backend: sled
  path: /var/lib/my-server/state
  cache_size: 10000  # Number of keys to cache in memory
```

**Implementation:**
```rust
pub struct SledStateManager {
    db: sled::Db,
}

impl SledStateManager {
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }
}
```

**Characteristics:**
- **Persistence:** All data survives process restarts
- **Performance:** O(log n) read/write (B-tree)
- **Durability:** ACID guarantees with fsync
- **Size:** Can handle billions of keys
- **Concurrency:** Thread-safe with internal locking

**Best practices:**
```rust
// Efficient batch operations
async fn batch_update(&self, updates: Vec<(String, Vec<u8>)>) -> Result<()> {
    let mut batch = Batch::default();
    for (key, value) in updates {
        batch.insert(key.as_bytes(), value);
    }
    self.db.apply_batch(batch)?;
    Ok(())
}
```

### 2. Memory (In-Memory Storage)

**Use case:** Testing, caching, ephemeral data

```yaml
state:
  backend: memory
```

**Implementation:**
```rust
pub struct MemoryStateManager {
    store: dashmap::DashMap<String, Vec<u8>>,
}
```

**Characteristics:**
- **Performance:** O(1) read/write (hash map)
- **Concurrency:** Lock-free with DashMap
- **Durability:** None - data lost on restart
- **Size:** Limited by RAM

**Best practices:**
```rust
// Use for caching expensive computations
async fn get_or_compute(&self, key: &str, compute: impl Fn() -> Vec<u8>) -> Result<Vec<u8>> {
    if let Some(cached) = self.get(key).await? {
        return Ok(cached);
    }

    let value = compute();
    self.set(key, value.clone(), Some(Duration::from_secs(300))).await?;
    Ok(value)
}
```

## Using State in Handlers

### Basic Usage

```rust
use pforge_runtime::{Handler, Result, StateManager};
use serde::{Deserialize, Serialize};

pub struct CounterHandler {
    state: Arc<dyn StateManager>,
}

#[derive(Deserialize)]
pub struct CounterInput {
    operation: String,  // "increment" or "get"
}

#[derive(Serialize)]
pub struct CounterOutput {
    value: u64,
}

#[async_trait::async_trait]
impl Handler for CounterHandler {
    type Input = CounterInput;
    type Output = CounterOutput;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        match input.operation.as_str() {
            "increment" => {
                let current = self.get_counter().await?;
                let new_value = current + 1;
                self.set_counter(new_value).await?;
                Ok(CounterOutput { value: new_value })
            }
            "get" => {
                let value = self.get_counter().await?;
                Ok(CounterOutput { value })
            }
            _ => Err(Error::Handler("Unknown operation".into()))
        }
    }
}

impl CounterHandler {
    async fn get_counter(&self) -> Result<u64> {
        let bytes = self.state.get("counter").await?;
        match bytes {
            Some(b) => Ok(u64::from_le_bytes(b.try_into().unwrap())),
            None => Ok(0),
        }
    }

    async fn set_counter(&self, value: u64) -> Result<()> {
        self.state.set("counter", value.to_le_bytes().to_vec(), None).await
    }
}
```

### Advanced: Serialization Helpers

```rust
use serde::{Deserialize, Serialize};

pub trait StateExt {
    async fn get_json<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>>;
    async fn set_json<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>;
}

impl<S: StateManager> StateExt for S {
    async fn get_json<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        match self.get(key).await? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)
                    .map_err(|e| Error::Handler(format!("JSON deserialize error: {}", e)))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set_json<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()> {
        let bytes = serde_json::to_vec(value)
            .map_err(|e| Error::Handler(format!("JSON serialize error: {}", e)))?;
        self.set(key, bytes, ttl).await
    }
}

// Usage
#[derive(Serialize, Deserialize)]
struct UserProfile {
    name: String,
    email: String,
}

async fn store_user(&self, user: &UserProfile) -> Result<()> {
    self.state.set_json(&format!("user:{}", user.email), user, None).await
}
```

## State Patterns

### 1. Counter Pattern

```rust
async fn atomic_increment(&self, key: &str) -> Result<u64> {
    loop {
        let current = self.get_json::<u64>(key).await?.unwrap_or(0);
        let new_value = current + 1;

        // In production, use compare-and-swap
        self.set_json(key, &new_value, None).await?;

        // Verify (simplified - use CAS in production)
        if self.get_json::<u64>(key).await? == Some(new_value) {
            return Ok(new_value);
        }
        // Retry on conflict
    }
}
```

### 2. Cache Pattern

```rust
async fn cached_api_call(&self, endpoint: &str) -> Result<Value> {
    let cache_key = format!("api_cache:{}", endpoint);

    // Check cache
    if let Some(cached) = self.state.get_json(&cache_key).await? {
        return Ok(cached);
    }

    // Call API
    let response = reqwest::get(endpoint).await?.json().await?;

    // Cache for 5 minutes
    self.state.set_json(&cache_key, &response, Some(Duration::from_secs(300))).await?;

    Ok(response)
}
```

### 3. Session Pattern

```rust
#[derive(Serialize, Deserialize)]
struct Session {
    user_id: String,
    created_at: DateTime<Utc>,
    data: HashMap<String, Value>,
}

async fn create_session(&self, user_id: String) -> Result<String> {
    let session_id = Uuid::new_v4().to_string();
    let session = Session {
        user_id,
        created_at: Utc::now(),
        data: HashMap::new(),
    };

    // Store with 1 hour TTL
    self.state.set_json(
        &format!("session:{}", session_id),
        &session,
        Some(Duration::from_secs(3600))
    ).await?;

    Ok(session_id)
}
```

### 4. Rate Limiting Pattern

```rust
async fn check_rate_limit(&self, user_id: &str, max_requests: u64, window: Duration) -> Result<bool> {
    let key = format!("rate_limit:{}:{}", user_id, Utc::now().timestamp() / window.as_secs() as i64);

    let count = self.state.get_json::<u64>(&key).await?.unwrap_or(0);

    if count >= max_requests {
        return Ok(false);  // Rate limit exceeded
    }

    self.state.set_json(&key, &(count + 1), Some(window)).await?;
    Ok(true)
}
```

## Performance Optimization

### 1. Batch Operations

```rust
async fn batch_get(&self, keys: Vec<String>) -> Result<HashMap<String, Vec<u8>>> {
    let mut results = HashMap::new();

    // Execute in parallel
    let futures: Vec<_> = keys.iter()
        .map(|key| self.state.get(key))
        .collect();

    let values = futures::future::join_all(futures).await;

    for (key, value) in keys.into_iter().zip(values) {
        if let Some(v) = value? {
            results.insert(key, v);
        }
    }

    Ok(results)
}
```

### 2. Connection Pooling

For Sled, use a shared instance:
```rust
lazy_static! {
    static ref STATE: Arc<SledStateManager> = Arc::new(
        SledStateManager::new("/var/lib/state").unwrap()
    );
}
```

### 3. Caching Layer

```rust
pub struct CachedStateManager {
    backend: Arc<dyn StateManager>,
    cache: Arc<DashMap<String, (Vec<u8>, Instant)>>,
    ttl: Duration,
}

impl CachedStateManager {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Check cache first
        if let Some((value, timestamp)) = self.cache.get(key) {
            if timestamp.elapsed() < self.ttl {
                return Ok(Some(value.clone()));
            }
        }

        // Fetch from backend
        let value = self.backend.get(key).await?;

        // Update cache
        if let Some(v) = &value {
            self.cache.insert(key.to_string(), (v.clone(), Instant::now()));
        }

        Ok(value)
    }
}
```

## Error Handling

```rust
async fn safe_state_operation(&self, key: &str) -> Result<Vec<u8>> {
    match self.state.get(key).await {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(Error::Handler(format!("Key not found: {}", key))),
        Err(e) => {
            // Log error
            eprintln!("State error: {}", e);

            // Return default value or propagate error
            Err(Error::Handler(format!("State backend error: {}", e)))
        }
    }
}
```

## Testing State

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pforge_runtime::MemoryStateManager;

    #[tokio::test]
    async fn test_counter_handler() {
        let state = Arc::new(MemoryStateManager::new());
        let handler = CounterHandler { state };

        // Increment
        let result = handler.handle(CounterInput {
            operation: "increment".into()
        }).await.unwrap();
        assert_eq!(result.value, 1);

        // Increment again
        let result = handler.handle(CounterInput {
            operation: "increment".into()
        }).await.unwrap();
        assert_eq!(result.value, 2);

        // Get
        let result = handler.handle(CounterInput {
            operation: "get".into()
        }).await.unwrap();
        assert_eq!(result.value, 2);
    }
}
```

## Best Practices

1. **Use appropriate backend**
   - Sled for persistence
   - Memory for caching and testing

2. **Serialize consistently**
   - Use JSON for complex types
   - Use binary for performance-critical data

3. **Handle missing keys gracefully**
   - Always check for None
   - Provide sensible defaults

4. **Use TTL for ephemeral data**
   - Sessions, caches, rate limits

5. **Batch when possible**
   - Reduce roundtrips
   - Use parallel execution

6. **Monitor state size**
   - Implement cleanup routines
   - Use TTL to prevent unbounded growth

7. **Test with real backends**
   - Use temporary directories for Sled in tests

## Future: Redis Backend

Future versions will support distributed state:

```yaml
state:
  backend: redis
  url: redis://localhost:6379
  pool_size: 10
```

---

**Next:** [Fault Tolerance](ch11-00-fault-tolerance.md)
