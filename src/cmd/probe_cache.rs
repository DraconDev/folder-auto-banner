//! Per-process content-probe cache.
//!
//! `get_file_contents` reads up to 64 KiB of each file to extract metadata
//! (PNG/JPG resolution, ZIP entry count, MP4/MOV/M4V/WebM/MKV duration,
//! SQLite table count, text line count). The daemon cache short-circuits
//! directory scans, but the content probe is a client-side per-file I/O
//! that runs on every `f` invocation.
//!
//! For a warm, recently-browsed directory the probed files almost never
//! change between calls, so we cache the probe result keyed by
//! `(path, size, mtime_nanos)`. If the size or mtime differs, the file
//! has been modified and we re-probe.
//!
//! The cache is bounded by an LRU-style eviction (drop the oldest entry
//! when the cache is full). It's process-local and not shared across
//! `f` invocations; each new process starts fresh, so memory is bounded
//! by the size of the working set in a single session.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

const MAX_ENTRIES: usize = 4096;

struct CacheEntry {
    /// Monotonic insert-order index used as a simple LRU timestamp.
    insert_order: u64,
    value: String,
}

pub struct ProbeCache {
    entries: HashMap<CacheKey, CacheEntry>,
    next_order: u64,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CacheKey {
    pub path: PathBuf,
    pub size: u64,
    /// Nanoseconds since the Unix epoch, or `None` if the file's mtime
    /// is not available. Two entries with the same path but different
    /// mtime/size are considered distinct (the file changed).
    pub mtime_nanos: Option<i128>,
    /// Tag distinguishing probe kinds so a file's contents-probe result
    /// can never collide with a directory's count result for the same
    /// (path, size, mtime). Kept tiny (1 byte) so it doesn't bloat the
    /// key's hash cost.
    pub kind: u8,
}

impl CacheKey {
    /// Build a cache key for a single file's content probe
    /// (PNG/JPG resolution, ZIP entry count, MP4/MOV/M4V/WebM/MKV
    /// duration, SQLite table count, text line count).
    pub fn for_file(
        path: &std::path::Path,
        size: u64,
        modified: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self::new(path, size, modified, PROBE_KIND_FILE)
    }

    /// Build a cache key for a directory's child count.
    pub fn for_dir(
        path: &std::path::Path,
        size: u64,
        modified: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self::new(path, size, modified, PROBE_KIND_DIR_COUNT)
    }

    fn new(
        path: &std::path::Path,
        size: u64,
        modified: Option<chrono::DateTime<chrono::Utc>>,
        kind: u8,
    ) -> Self {
        let mtime_nanos = modified.map(|dt| {
            let nanos = dt.timestamp_nanos_opt().unwrap_or(0);
            nanos as i128
        });
        Self {
            path: path.to_path_buf(),
            size,
            mtime_nanos,
            kind,
        }
    }
}

const PROBE_KIND_FILE: u8 = 1;
const PROBE_KIND_DIR_COUNT: u8 = 2;

static CACHE: OnceLock<std::sync::Mutex<ProbeCache>> = OnceLock::new();

fn cache() -> &'static std::sync::Mutex<ProbeCache> {
    CACHE.get_or_init(|| {
        std::sync::Mutex::new(ProbeCache {
            entries: HashMap::with_capacity(MAX_ENTRIES),
            next_order: 0,
        })
    })
}

impl ProbeCache {
    /// Look up a cached probe result for the given key. Returns `None` if
    /// the key is not present (or if the cache lock is poisoned).
    pub fn get(key: &CacheKey) -> Option<String> {
        let guard = cache().lock().ok()?;
        guard.entries.get(key).map(|e| e.value.clone())
    }

    /// Insert a probe result. If the cache is full, evict the oldest
    /// entry (by insertion order) to make room.
    pub fn put(key: CacheKey, value: String) {
        let Ok(mut guard) = cache().lock() else {
            return;
        };
        if !guard.entries.contains_key(&key) && guard.entries.len() >= MAX_ENTRIES {
            // Batch eviction: collect the oldest 10% entries and remove them.
            // This amortizes the O(N) sort across hundreds of subsequent insertions.
            let evict_count = (MAX_ENTRIES / 10).max(1);
            let mut entries_with_order: Vec<(CacheKey, u64)> = guard
                .entries
                .iter()
                .map(|(k, v)| (k.clone(), v.insert_order))
                .collect();
            entries_with_order.sort_by_key(|(_, order)| *order);
            for (k, _) in entries_with_order.into_iter().take(evict_count) {
                guard.entries.remove(&k);
            }
        }
        let order = guard.next_order;
        guard.next_order = guard.next_order.wrapping_add(1);
        guard.entries.insert(
            key,
            CacheEntry {
                insert_order: order,
                value,
            },
        );
    }

    /// Number of entries currently in the cache. Used for tests / metrics.
    pub fn len() -> usize {
        cache().lock().map(|g| g.entries.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(p: &str, size: u64, nanos: i128) -> CacheKey {
        CacheKey {
            path: PathBuf::from(p),
            size,
            mtime_nanos: Some(nanos),
            kind: PROBE_KIND_FILE,
        }
    }

    #[test]
    fn put_and_get() {
        let k = key("/tmp/a.png", 100, 1);
        ProbeCache::put(k.clone(), "10x10".to_string());
        assert_eq!(ProbeCache::get(&k).as_deref(), Some("10x10"));
    }

    #[test]
    fn missing_returns_none() {
        let k = key("/tmp/missing.png", 0, 0);
        assert_eq!(ProbeCache::get(&k), None);
    }

    #[test]
    fn different_size_same_path_is_distinct() {
        let k1 = key("/tmp/x.png", 100, 1);
        let k2 = key("/tmp/x.png", 200, 1);
        ProbeCache::put(k1.clone(), "old".to_string());
        ProbeCache::put(k2.clone(), "new".to_string());
        assert_eq!(ProbeCache::get(&k1).as_deref(), Some("old"));
        assert_eq!(ProbeCache::get(&k2).as_deref(), Some("new"));
    }

    #[test]
    fn different_kinds_for_same_path_are_distinct() {
        let file_key = CacheKey::for_file(&PathBuf::from("/tmp/x"), 100, None);
        let dir_key = CacheKey::for_dir(&PathBuf::from("/tmp/x"), 100, None);
        ProbeCache::put(file_key.clone(), "1920x1080".to_string());
        ProbeCache::put(dir_key.clone(), "5".to_string());
        assert_eq!(ProbeCache::get(&file_key).as_deref(), Some("1920x1080"));
        assert_eq!(ProbeCache::get(&dir_key).as_deref(), Some("5"));
    }

    #[test]
    fn evicts_oldest_when_full() {
        // Fill the cache. We don't pre-clear (process-shared), so just
        // exercise the eviction path by inserting > MAX_ENTRIES unique keys.
        for i in 0..(MAX_ENTRIES + 100) {
            let k = CacheKey {
                path: PathBuf::from(format!("/tmp/evict-{}.png", i)),
                size: i as u64,
                mtime_nanos: Some(i as i128),
                kind: PROBE_KIND_FILE,
            };
            ProbeCache::put(k, format!("{}", i));
        }
        // Cache should be bounded at MAX_ENTRIES.
        assert!(ProbeCache::len() <= MAX_ENTRIES);
    }
}
