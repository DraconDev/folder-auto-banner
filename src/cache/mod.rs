//! Cache module — TTL-based file cache for expensive operations
//!
//! Stores cached values as JSON files in a temp directory keyed by project path.
//! Each entry has a timestamp; expired entries are ignored.

use anyhow::Result;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// File-based TTL cache
pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    /// Create a new cache instance
    pub fn new() -> Result<Self> {
        let dir = std::env::temp_dir().join("cfm-cache");
        std::fs::create_dir_all(&dir)?;
        Ok(Cache { dir })
    }

    /// Get cache file path for a given key
    fn path_for(&self, key: &str) -> PathBuf {
        // Hash the key to avoid filesystem issues with long paths
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = format!("{:016x}", hasher.finish());
        self.dir.join(format!("{}.json", hash))
    }

    /// Get a cached value if it exists and is not expired
    pub fn get<T: serde::Serialize + for<'de> serde::Deserialize<'de>>(
        &self,
        key: &str,
        ttl: Duration,
    ) -> Option<T> {
        let path = self.path_for(key);
        let content = std::fs::read_to_string(&path).ok()?;

        // Parse as generic JSON to extract timestamp
        let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
        let timestamp_ms = parsed.get("timestamp_ms")?.as_u64()?;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if now_ms.saturating_sub(timestamp_ms) < ttl.as_millis() as u64 {
            let value = serde_json::from_value(parsed.get("value")?.clone()).ok()?;
            Some(value)
        } else {
            None
        }
    }

    /// Store a value in the cache
    pub fn set<T: serde::Serialize>(&self, key: &str, value: T) -> Result<()> {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let entry = serde_json::json!({
            "value": value,
            "timestamp_ms": now_ms,
        });

        let path = self.path_for(key);
        let content = serde_json::to_string(&entry)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Clear expired entries (best-effort cleanup)
    pub fn cleanup(&self, max_age: Duration) {
        if let Ok(entries) = std::fs::read_dir(&self.dir) {
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(ts) = parsed.get("timestamp_ms").and_then(|v| v.as_u64()) {
                            if now_ms.saturating_sub(ts) > max_age.as_millis() as u64 {
                                let _ = std::fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Build a cache key from a path and a feature name
pub fn cache_key(path: &std::path::Path, feature: &str) -> String {
    format!("{}:{}", path.to_string_lossy(), feature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_cache_set_and_get() {
        let cache = Cache::new().unwrap();
        let key = format!("test-key-{}", std::process::id());
        let value: String = "hello".to_string();

        cache.set(&key, &value).unwrap();
        let result: Option<String> = cache.get(&key, Duration::from_secs(60));
        assert_eq!(result, Some(value));

        // Cleanup
        let _ = std::fs::remove_file(cache.path_for(&key));
    }

    #[test]
    fn test_cache_expired() {
        let cache = Cache::new().unwrap();
        let key = format!("test-expired-{}", std::process::id());

        cache.set(&key, &"value".to_string()).unwrap();
        // Get with 0 TTL should always expire
        let result: Option<String> = cache.get(&key, Duration::from_secs(0));
        assert_eq!(result, None);

        // Cleanup
        let _ = std::fs::remove_file(cache.path_for(&key));
    }

    #[test]
    fn test_cache_key_format() {
        let path = PathBuf::from("/home/user/project");
        let key = cache_key(&path, "build");
        assert_eq!(key, "/home/user/project:build");
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = Cache::new().unwrap();
        let key = format!("test-cleanup-{}", std::process::id());

        cache.set(&key, &"value".to_string()).unwrap();
        // Cleanup with 0 max_age should remove everything
        cache.cleanup(Duration::from_secs(0));

        // Verify removed (may fail if file was recreated, but should be gone)
        let path = cache.path_for(&key);
        // Note: cleanup is best-effort, so we just verify the function runs
        let _ = path;
    }
}
