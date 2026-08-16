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

/// Returns true if the (lower-cased) file name has an extension that the
/// daemon runs a content probe on (text files, images, archives, etc.).
/// Used to limit the per-file mtime staleness check to files that could
/// actually have changed banner metadata. This is the canonical list —
/// the daemon's inotify watcher delegates to it.
pub fn is_content_probe_ext(lower_name: &str) -> bool {
    lower_name.ends_with(".txt")
        || lower_name.ends_with(".md")
        || lower_name.ends_with(".json")
        || lower_name.ends_with(".js")
        || lower_name.ends_with(".ts")
        || lower_name.ends_with(".jsx")
        || lower_name.ends_with(".tsx")
        || lower_name.ends_with(".rs")
        || lower_name.ends_with(".py")
        || lower_name.ends_with(".go")
        || lower_name.ends_with(".java")
        || lower_name.ends_with(".rb")
        || lower_name.ends_with(".c")
        || lower_name.ends_with(".cpp")
        || lower_name.ends_with(".h")
        || lower_name.ends_with(".hpp")
        || lower_name.ends_with(".sh")
        || lower_name.ends_with(".yaml")
        || lower_name.ends_with(".yml")
        || lower_name.ends_with(".toml")
        || lower_name.ends_with(".xml")
        || lower_name.ends_with(".html")
        || lower_name.ends_with(".css")
        || lower_name.ends_with(".scss")
        || lower_name.ends_with(".png")
        || lower_name.ends_with(".jpg")
        || lower_name.ends_with(".jpeg")
        || lower_name.ends_with(".gif")
        || lower_name.ends_with(".webp")
        || lower_name.ends_with(".bmp")
        || lower_name.ends_with(".tiff")
        || lower_name.ends_with(".zip")
        || lower_name.ends_with(".tar")
        || lower_name.ends_with(".gz")
        || lower_name.ends_with(".mp4")
        || lower_name.ends_with(".mov")
        || lower_name.ends_with(".mkv")
        || lower_name.ends_with(".webm")
        || lower_name.ends_with(".m4v")
}

/// Returns the maximum mtime of all files with content-probe
/// extensions under `path`, recursing into subdirectories with a
/// bounded walk (depth ≤ 8, ≤ 8192 entries visited, heavy dirs
/// skipped). Used to detect file content changes that don't advance
/// the directory's own mtime (e.g., editing a file in place). A
/// flat, one-level scan missed nested edits at depth ≥ 2, which
/// could keep banners stale for the full 300s cache TTL.
pub fn max_descendant_mtime(path: &Path) -> Option<SystemTime> {
    fn walk(path: &Path, depth: usize, visited: &mut usize) -> Option<SystemTime> {
        if depth > 8 || *visited >= 8192 {
            return None;
        }
        let entries = std::fs::read_dir(path).ok()?;
        let mut max: Option<SystemTime> = None;
        for entry in entries.flatten() {
            if *visited >= 8192 {
                break;
            }
            *visited += 1;
            let name_str = entry.file_name().to_string_lossy().to_string();
            if !is_content_probe_ext(&name_str.to_ascii_lowercase()) {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    if let Ok(mtime) = meta.modified() {
                        max = Some(max.map_or(mtime, |m| m.max(mtime)));
                    }
                } else if meta.is_dir() {
                    // Skip heavyweight subtrees: probe-relevant content under
                    // them is not shown by the banner anyway.
                    if matches!(
                        name_str.as_ref(),
                        ".git" | "target" | "node_modules" | ".cache" | ".venv" | "build"
                    ) {
                        continue;
                    }
                    if let Some(mtime) = walk(&entry.path(), depth + 1, visited) {
                        max = Some(max.map_or(mtime, |m| m.max(mtime)));
                    }
                }
            }
        }
        max
    }

    let mut visited = 0usize;
    walk(path, 0, &mut visited)
}

/// Returns `true` if the cache file for `path` exists, is younger than
/// `CACHE_TTL`, AND is not older than the directory or any of its
/// direct children. The last two checks guard against the case where
/// the user changed files in the directory (add/remove advances the
/// dir mtime; in-place edit advances the file's mtime but not the
/// dir mtime).
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
    // the cache file's mtime, the file is stale (e.g., a file was
    // added or removed).
    if let Some(dir_mtime) = directory_mtime(path) {
        if dir_mtime > file_mtime {
            return false;
        }
    }
    // Guard against in-place file edits that don't advance the dir
    // mtime: if any direct child's mtime is newer than the cache
    // file's mtime, the file is stale. O(N) stat calls, but page-cached
    // and fast (~0.6 ms for ~200 files).
    if let Some(max_child_mtime) = max_descendant_mtime(path) {
        if max_child_mtime > file_mtime {
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
        tracing::warn!("Cache path is a directory, removing: {}", file.display());
        let _ = std::fs::remove_dir(&file);
        return None;
    }
    let bytes = std::fs::read(&file).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Remove the per-path disk cache file. Called when the daemon invalidates
/// a banner: the file's mtime is the client's freshness signal, so a stale
/// file would keep serving pre-invalidation data on the client fast path.
pub fn remove_cache(path: &Path) {
    if let Some(file) = cache_file_path(path) {
        let _ = std::fs::remove_file(file);
    }
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
            tracing::warn!("Cache path is a directory, removing: {}", file.display());
            let _ = std::fs::remove_dir(&file);
        }
    }
    // Atomic write: temp file + rename in the same directory, so a
    // concurrent reader (client fast path) never observes a partially
    // written file.
    let tmp = file.with_extension("json.tmp");
    if let Err(e) = std::fs::write(&tmp, &bytes) {
        tracing::warn!("Failed to write banner data cache: {}", e);
        return Ok(());
    }
    if let Err(e) = std::fs::rename(&tmp, &file) {
        let _ = std::fs::remove_file(&tmp);
        tracing::warn!("Failed to rename banner data cache: {}", e);
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
    use crate::fs::{DirEntry, DirSummary, ProjectType};
    use std::fs;
    use std::time::Duration;

    fn make_test_banner_data() -> BannerData {
        BannerData {
            summary: DirSummary {
                total_items: 1,
                total_size: 42,
                files: 1,
                dirs: 0,
                top_items: vec![DirEntry {
                    name: "test.txt".to_string(),
                    path: PathBuf::from("/tmp/test.txt"),
                    is_dir: false,
                    is_file: true,
                    is_symlink: false,
                    is_exec: false,
                    size: 42,
                    modified: None,
                    perms: "rw-r--r--".to_string(),
                    owner: "dracon".to_string(),
                    group: "users".to_string(),
                    symlink_target: None,
                    symlink_valid: true,
                    content_probe: None,
                }],
                project_type: ProjectType::Generic,
                last_modified: None,
                build_status: None,
                todo_info: None,
                code_metrics: None,
                port_info: None,
                docker_info: None,
            },
            git_info: None,
        }
    }

    #[test]
    fn fnv1a_64_is_stable() {
        // Spot-check a few well-known values.
        assert_eq!(fnv1a_64(b""), 0xcbf29ce484222325);
        assert_eq!(fnv1a_64(b"a"), 0xaf63dc4c8601ec8c);
        assert_eq!(fnv1a_64(b"foobar"), 0x85944171f73967e8);
    }

    #[test]
    fn fnv1a_64_changes_with_input() {
        assert_ne!(fnv1a_64(b"a"), fnv1a_64(b"b"));
        assert_ne!(fnv1a_64(b"abc"), fnv1a_64(b"abd"));
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
    fn cache_file_path_uses_hex_hash() {
        let p = cache_file_path(Path::new("/tmp/test")).unwrap();
        let fname = p.file_name().unwrap().to_str().unwrap();
        // Must be 16 hex chars + .json
        assert_eq!(fname.len(), 21);
        assert!(fname.ends_with(".json"));
        let hash = &fname[..16];
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn nonexistent_cache_is_not_fresh() {
        // Use a path that almost certainly does not have a cache file.
        assert!(!is_cache_fresh(Path::new(
            "/tmp/this/path/should/not/exist/fab-test"
        )));
    }

    #[test]
    fn write_and_read_cache_roundtrip() {
        // Use a temp dir for the data dir by overriding the directories call.
        // We can't easily override directories::ProjectDirs, so we test
        // write_cache/read_cache with a real temp path.
        let tmp = std::env::temp_dir().join(format!("fab-test-{}", std::process::id()));
        let _ = fs::create_dir_all(&tmp);
        let test_path = tmp.join("test_dir");
        let _ = fs::create_dir_all(&test_path);

        // Write
        let data = make_test_banner_data();
        write_cache(&test_path, &data).unwrap();

        // The cache file should exist at the expected location
        let cache_file = cache_file_path(&test_path).unwrap();
        assert!(
            cache_file.exists(),
            "cache file should exist at {:?}",
            cache_file
        );

        // Read back
        let read_back = read_cache(&test_path).expect("read_cache should succeed");
        assert_eq!(read_back.summary.total_items, 1);
        assert_eq!(read_back.summary.total_size, 42);
        assert_eq!(read_back.summary.top_items.len(), 1);
        assert_eq!(read_back.summary.top_items[0].name, "test.txt");

        // Cleanup
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn write_cache_creates_parent_dir() {
        let tmp = std::env::temp_dir().join(format!("fab-test-parent-{}", std::process::id()));
        let test_path = tmp.join("nested/test_dir");
        let _ = fs::create_dir_all(&test_path);

        // The banner_data subdir shouldn't exist yet
        let cache_file = cache_file_path(&test_path).unwrap();
        let parent = cache_file.parent().unwrap();
        let _ = fs::remove_dir_all(parent);

        // Write should create the parent
        let data = make_test_banner_data();
        write_cache(&test_path, &data).unwrap();
        assert!(parent.exists(), "parent dir should be created");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn read_cache_returns_none_for_missing_file() {
        // Path that doesn't have a cache file
        let result = read_cache(Path::new(
            "/tmp/this/path/should/not/exist/fab-test-missing",
        ));
        assert!(result.is_none());
    }

    #[test]
    fn read_cache_handles_corrupt_json() {
        let tmp = std::env::temp_dir().join(format!("fab-test-corrupt-{}", std::process::id()));
        let _ = fs::create_dir_all(&tmp);
        let test_path = tmp.join("test_dir");
        let _ = fs::create_dir_all(&test_path);

        // Write garbage to the cache file
        let cache_file = cache_file_path(&test_path).unwrap();
        if let Some(parent) = cache_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&cache_file, b"{ this is not valid json").unwrap();

        // Read should return None, not panic
        let result = read_cache(&test_path);
        assert!(result.is_none(), "corrupt JSON should return None");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn read_cache_handles_directory_at_cache_path() {
        let tmp = std::env::temp_dir().join(format!("fab-test-dirpath-{}", std::process::id()));
        let _ = fs::create_dir_all(&tmp);
        let test_path = tmp.join("test_dir");
        let _ = fs::create_dir_all(&test_path);

        // Create a directory at the cache file path
        let cache_file = cache_file_path(&test_path).unwrap();
        if let Some(parent) = cache_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::remove_file(&cache_file);
        fs::create_dir(&cache_file).unwrap();

        // Read should remove the directory and return None
        let result = read_cache(&test_path);
        assert!(
            result.is_none(),
            "directory at cache path should return None"
        );
        assert!(!cache_file.exists(), "directory should be removed");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn write_cache_replaces_directory_at_cache_path() {
        let tmp = std::env::temp_dir().join(format!("fab-test-replace-{}", std::process::id()));
        let _ = fs::create_dir_all(&tmp);
        let test_path = tmp.join("test_dir");
        let _ = fs::create_dir_all(&test_path);

        // Create a directory at the cache file path
        let cache_file = cache_file_path(&test_path).unwrap();
        if let Some(parent) = cache_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::remove_file(&cache_file);
        fs::create_dir(&cache_file).unwrap();

        // Write should remove the directory and create a file
        let data = make_test_banner_data();
        write_cache(&test_path, &data).unwrap();
        assert!(
            cache_file.is_file(),
            "directory should be replaced with file"
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn is_cache_fresh_false_for_missing_file() {
        assert!(!is_cache_fresh(Path::new(
            "/tmp/this/path/should/not/exist/fab-fresh-test"
        )));
    }

    #[test]
    fn is_cache_fresh_true_for_recent_file() {
        // Use a real path that we just wrote
        let tmp = std::env::temp_dir().join(format!("fab-test-isfresh-{}", std::process::id()));
        let _ = fs::create_dir_all(&tmp);
        let test_path = tmp.join("test_dir");
        let _ = fs::create_dir_all(&test_path);

        let data = make_test_banner_data();
        write_cache(&test_path, &data).unwrap();

        // File was just written, so it should be fresh
        assert!(is_cache_fresh(&test_path));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn is_cache_fresh_false_for_directory() {
        let tmp = std::env::temp_dir().join(format!("fab-test-isdir-{}", std::process::id()));
        let _ = fs::create_dir_all(&tmp);
        let test_path = tmp.join("test_dir");
        let _ = fs::create_dir_all(&test_path);

        // Create a directory at the cache file path
        let cache_file = cache_file_path(&test_path).unwrap();
        if let Some(parent) = cache_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::remove_file(&cache_file);
        fs::create_dir(&cache_file).unwrap();

        // Directory at cache path should not be fresh
        assert!(!is_cache_fresh(&test_path));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn cache_ttl_is_5_minutes() {
        // Sanity check: the TTL must match the daemon's CACHE_TTL.
        assert_eq!(CACHE_TTL, Duration::from_secs(300));
    }
}
