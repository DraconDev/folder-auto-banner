//! Per-directory on-disk cache of `BannerData`.
//!
//! The daemon's IPC `read4` (4-byte length prefix) roundtrip has a
//! kernel-scheduling floor of ~1–10 ms on Linux, even for a tiny
//! 4-byte read on a Unix socket. For the `f <path>` use case (a fresh
//! process on every invocation, often called several times per second
//! when browsing), that floor dominates the wall-clock cost of an
//! otherwise-fast call.
//!
//! To skip the IPC for cache hits, the daemon writes the serialized
//! `BannerData` to a per-path file under `banner_data/` after every
//! successful banner compute (both cache miss and cache hit), with the
//! file's mtime set to the compute time. The client, before opening a
//! Unix-socket connection, checks whether the file exists and whether
//! its mtime is within the daemon's `CACHE_TTL`. If so, it reads and
//! deserializes the file directly and skips the IPC entirely.
//!
//! A 4-byte read of a Unix-socket response is dominated by kernel
//! scheduling (1–10 ms); a stat + read of a 70 KB file is dominated by
//! page-cache hits (<0.1 ms). The disk path is therefore typically
//! 5–50× faster than the IPC path for a warm cache hit.
//!
//! The cache file is owned by the daemon, not the client, so cache
//! invalidation (TTL expiry, root-mtime change) is handled by the
//! daemon's existing banner-cache logic. The file's mtime is the
//! freshness signal; if the daemon's in-memory cache is invalidated,
//! the next banner compute re-writes the file with a new mtime.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::daemon_types::BannerData;

/// Maximum age of a cache file before the client considers it stale.
/// Must match the daemon's `CACHE_TTL` (`Duration::from_secs(300)` in
/// `src/daemon.rs`). Exposed here as a constant so the client doesn't
/// need to read it from the daemon at startup.
pub const CACHE_TTL: Duration = Duration::from_secs(300);

/// Subdirectory under the data dir where per-path cache files live.
const CACHE_SUBDIR: &str = "banner_data";

/// Returns the cache file path for the given directory, e.g.
/// `~/.local/share/fab/banner_data/3f8a2b1c...d4e5.json`.
///
/// The hash is a stable FNV-1a 64-bit digest of the canonicalized path
/// (lowercased on Windows would be needed for cross-platform stability,
/// but Unix paths are case-sensitive so we hash as-is).
pub fn cache_file_path(path: &Path) -> Option<PathBuf> {
    let data_dir = data_dir()?;
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let hash = fnv1a_64(canonical.to_string_lossy().as_bytes());
    Some(
        data_dir
            .join(CACHE_SUBDIR)
            .join(format!("{:016x}.json", hash)),
    )
}

/// Returns the cache directory, creating it if necessary.
fn data_dir() -> Option<PathBuf> {
    let dir = directories::ProjectDirs::from("com", "fab", "fab")?
        .data_dir()
        .to_path_buf();
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    Some(dir)
}

/// FNV-1a 64-bit hash. Stable across Rust versions and platforms
/// (unlike `DefaultHasher`).
fn fnv1a_64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut h = FNV_OFFSET;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// Returns the mtime of the cache file for `path`, or `None` if the
/// file does not exist, is a directory, or its mtime cannot be read.
pub fn cache_file_mtime(path: &Path) -> Option<SystemTime> {
    let file = cache_file_path(path)?;
    let meta = std::fs::metadata(&file).ok()?;
    if meta.is_dir() {
        return None;
    }
    meta.modified().ok()
}

/// Returns the mtime of the directory at `path`. Used to validate that
/// a cache file's mtime is not older than the directory it describes
/// (if a user changes files in the directory, the directory's mtime
/// advances; the cache file's mtime should track that).
pub fn directory_mtime(path: &Path) -> Option<SystemTime> {
    let meta = std::fs::metadata(path).ok()?;
    meta.modified().ok()
}

/// Returns `true` if the cache file for `path` exists, is younger than
/// `CACHE_TTL`, AND is not older than the directory it describes. The
/// last check guards against the case where the user changed files
/// in the directory while the daemon was down or the cache file was
/// otherwise not refreshed.
pub fn is_cache_fresh(path: &Path) -> bool {
    let Some(file_mtime) = cache_file_mtime(path) else {
        return false;
    };
    let Ok(age) = SystemTime::now().duration_since(file_mtime) else {
        return false;
    };
    if age >= CACHE_TTL {
        return false;
    }
    // Guard against stale data: if the directory's mtime is newer than
    // the cache file's mtime, the file is stale.
    if let Some(dir_mtime) = directory_mtime(path) {
        if dir_mtime > file_mtime {
            return false;
        }
    }
    true
}

/// Read and deserialize the cache file for `path`. Returns `None` if
/// the file is missing, unreadable, is a directory, or fails to
/// deserialize. If the path is a directory (corruption from a previous
/// bug, manual intervention, etc.), the directory is removed so the
/// daemon can write a fresh file on the next IPC call.
pub fn read_cache(path: &Path) -> Option<BannerData> {
    let file = cache_file_path(path)?;
    let meta = std::fs::metadata(&file).ok()?;
    if meta.is_dir() {
        tracing::warn!(
            "Cache path is a directory, removing: {}",
            file.display()
        );
        let _ = std::fs::remove_dir(&file);
        return None;
    }
    let bytes = std::fs::read(&file).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Serialize `data` and write it to the cache file for `path`. Sets
/// the file's mtime to `now`. Returns `Ok(())` on success.
///
/// This is a best-effort write: any I/O error is logged but does not
/// fail the caller's banner render, because the disk cache is a pure
/// performance optimization — the IPC path is still the fallback.
pub fn write_cache(path: &Path, data: &BannerData) -> std::io::Result<()> {
    let Some(file) = cache_file_path(path) else {
        return Ok(());
    };
    if let Some(parent) = file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let bytes = match serde_json::to_vec(data) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("Failed to serialize banner data cache: {}", e);
            return Ok(());
        }
    };
    // If the path exists and is a directory (corruption from a previous
    // bug, manual intervention, or filesystem weirdness), remove it so
    // we can write a regular file. We only remove it if it's a directory
    // — never remove a regular file.
    if let Ok(meta) = std::fs::metadata(&file) {
        if meta.is_dir() {
            tracing::warn!(
                "Cache path is a directory, removing: {}",
                file.display()
            );
            let _ = std::fs::remove_dir(&file);
        }
    }
    if let Err(e) = std::fs::write(&file, &bytes) {
        tracing::warn!("Failed to write banner data cache: {}", e);
        return Ok(());
    }
    // The mtime is implicitly set to "now" by the write. No need to
    // call `utimes`/`filetime` — `std::fs::write` does the right thing.
    let _ = SystemTime::now().duration_since(UNIX_EPOCH);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_64_is_stable() {
        // Spot-check a few well-known values.
        assert_eq!(fnv1a_64(b""), 0xcbf29ce484222325);
        assert_eq!(fnv1a_64(b"a"), 0xaf63dc4c8601ec8c);
        assert_eq!(fnv1a_64(b"foobar"), 0x85944171f73967e8);
    }

    #[test]
    fn cache_file_path_is_deterministic() {
        let p = Path::new("/tmp/some/path");
        let a = cache_file_path(p).unwrap();
        let b = cache_file_path(p).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn cache_file_path_differs_per_input() {
        let a = cache_file_path(Path::new("/tmp/a")).unwrap();
        let b = cache_file_path(Path::new("/tmp/b")).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn nonexistent_cache_is_not_fresh() {
        // Use a path that almost certainly does not have a cache file.
        assert!(!is_cache_fresh(Path::new(
            "/tmp/this/path/should/not/exist/fab-test"
        )));
    }
}
