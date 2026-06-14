//! File metadata extraction — image resolution, ZIP entries, SQLite tables, video duration
//!
//! These functions read small headers from each file to extract metadata for the
//! banner. We deliberately read only the first few KiB of binary files (PNG, JPG,
//! ZIP, MP4, MKV) — the dimensions, entry count, and duration are all stored near
//! the start of the file, so a full `read()` is wasteful and made `f` very slow
//! in directories with many images, archives, or videos.

use std::io::Read;
use std::path::Path;

/// How many bytes to read for binary file-header probes. PNG / JPEG / ZIP / MP4
/// / MKV all carry their metadata in the first few KiB, so 64 KiB is more than
/// enough while keeping the per-file cost in the microsecond range.
const FILE_HEADER_PROBE_BYTES: usize = 64 * 1024;

fn read_file_header(path: &Path) -> Option<Vec<u8>> {
    // Open the file and read up to FILE_HEADER_PROBE_BYTES. We don't pre-stat
    // the file (the caller has already done so via `entry.size` for the size
    // column) — we just take(64KB) which reads at most that many bytes and
    // stops at EOF. This saves one stat() syscall per probed file, which
    // matters in directories with hundreds of images / archives / videos.
    let file = std::fs::File::open(path).ok()?;
    let mut buf = Vec::with_capacity(FILE_HEADER_PROBE_BYTES.min(8 * 1024));
    file.take(FILE_HEADER_PROBE_BYTES as u64)
        .read_to_end(&mut buf)
        .ok()?;
    Some(buf)
}

/// Get contents description for a file — line count for text, resolution for image, etc.
/// Returns plain text (no ANSI codes) — coloring is applied by the renderer.
#[allow(dead_code)]
pub fn get_file_contents(entry: &crate::fs::DirEntry) -> String {
    // Per-process cache: identical (path, size, mtime) lookups are served
    // from memory, so a warm `f` on the same directory doesn't re-read
    // headers we already know about. The cache is bounded by an LRU-style
    // eviction; see `probe_cache.rs` for the full design.
    let cache_key = crate::cmd::probe_cache::CacheKey::for_file(
        &entry.path,
        entry.size,
        entry.modified,
    );
    if let Some(cached) = crate::cmd::probe_cache::ProbeCache::get(&cache_key) {
        return cached;
    }

    // Use Path::extension() to avoid an allocation per probed file. The
    // returned extension is already ASCII-lower (per std::path docs), so we
    // don't need to lowercase the name to compare. This drops one String
    // allocation per file in directories with many different extensions.
    let ext = std::path::Path::new(&entry.name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let result = match ext {
        "png" | "jpg" | "jpeg" => read_file_header(&entry.path)
            .and_then(|bytes| extract_image_resolution(&bytes, ext))
            .unwrap_or_default(),
        "zip" => read_file_header(&entry.path)
            .and_then(|bytes| count_zip_entries(&bytes))
            .map(|c| c.to_string())
            .unwrap_or_default(),
        "db" | "sqlite" | "sqlite3" => count_sqlite_tables(&entry.path)
            .map(|c| format!("{}t", c))
            .unwrap_or_default(),
        "mp4" | "mov" | "m4v" | "webm" | "mkv" => {
            extract_video_duration(&entry.path).unwrap_or_default()
        }
        _ => {
            // Text files under 1 MiB: count lines. We deliberately do NOT
            // cache this result, because text files are commonly edited in
            // place and the size/mtime signal isn't a reliable change
            // detector for in-place line-count changes. The cost of a fresh
            // `read_to_string` on a small text file is small (a few hundred
            // microseconds at most), so re-reading is the right tradeoff.
            if entry.size < 1024 * 1024 {
                if let Ok(content) = std::fs::read_to_string(&entry.path) {
                    return content.lines().count().to_string();
                }
            }
            String::new()
        }
    };

    // Cache the result. We only cache binary-file probes (PNG/JPG/ZIP/
    // MP4/MOV/M4V/WebM/MKV/SQLite) above; the text-file branch returns
    // early without caching.
    crate::cmd::probe_cache::ProbeCache::put(cache_key, result.clone());
    result
}

/// Count items in a directory
pub fn count_items_in_dir(entry: &crate::fs::DirEntry) -> usize {
    // Same per-process cache pattern as `get_file_contents`. Directory
    // contents are cheap (just `readdir`) but the call still costs a
    // syscall and path resolution; on a warm `f` invocation we can serve
    // repeated counts from the cache as long as the directory's mtime
    // and size don't change. (For directories we treat `size` as a
    // rough hint; the mtime is the real signal that children changed.)
    let cache_key = crate::cmd::probe_cache::CacheKey::for_dir(
        &entry.path,
        entry.size,
        entry.modified,
    );
    if let Some(cached) = crate::cmd::probe_cache::ProbeCache::get(&cache_key) {
        if let Ok(n) = cached.parse::<usize>() {
            return n;
        }
    }

    let count = std::fs::read_dir(&entry.path)
        .map(|d| d.count())
        .unwrap_or(0);
    crate::cmd::probe_cache::ProbeCache::put(cache_key, count.to_string());
    count
}

/// Extract image resolution from PNG or JPEG header bytes.
/// `ext` is the lowercased extension with or without the leading dot, e.g.
/// "png" or ".png". The dispatch is by ext so we can return early without
/// scanning.
fn extract_image_resolution(bytes: &[u8], ext: &str) -> Option<String> {
    let ext = ext.strip_prefix('.').unwrap_or(ext);
    if ext == "png" && bytes.len() >= 24 {
        // PNG: width at offset 16-19 (big endian), height at 20-23
        let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) as usize;
        let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]) as usize;
        if w > 0 && h > 0 {
            return Some(format!("{}x{}", w, h));
        }
    } else if ext == "jpg" || ext == "jpeg" {
        // JPEG: find SOF marker and read dimensions
        let mut i = 2;
        while i < bytes.len().saturating_sub(9) {
            if bytes[i] == 0xFF
                && bytes[i + 1] >= 0xC0
                && bytes[i + 1] <= 0xCF
                && bytes[i + 1] != 0xC4
                && bytes[i + 1] != 0xC8
                && bytes[i + 1] != 0xCC
            {
                let h = ((bytes[i + 5] as usize) << 8) | (bytes[i + 6] as usize);
                let w = ((bytes[i + 7] as usize) << 8) | (bytes[i + 8] as usize);
                if w > 0 && h > 0 {
                    return Some(format!("{}x{}", w, h));
                }
            }
            i += 1;
        }
    }
    None
}

/// Count ZIP file entries by scanning local file headers
fn count_zip_entries(bytes: &[u8]) -> Option<usize> {
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len().saturating_sub(4) {
        if bytes[i] == 0x50 && bytes[i + 1] == 0x4B && bytes[i + 2] == 0x03 && bytes[i + 3] == 0x04
        {
            count += 1;
            i += 4;
        } else {
            i += 1;
        }
    }
    if count > 0 {
        Some(count)
    } else {
        None
    }
}

/// Count SQLite tables by reading schema
fn count_sqlite_tables(path: &Path) -> Option<usize> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() < 16 {
        return None;
    }
    let header = std::str::from_utf8(&bytes[..16]).ok()?;
    if !header.starts_with("SQLite format 3") {
        return None;
    }

    use std::process::Command;
    let output = Command::new("sqlite3")
        .arg(path)
        .arg("SELECT COUNT(*) FROM sqlite_master WHERE type='table';")
        .output()
        .ok()?;

    let count = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .ok()?;
    Some(count)
}

/// Extract video duration from MP4/MOV container headers
fn extract_video_duration(path: &Path) -> Option<String> {
    // The `moov` atom can appear at the end of an MP4 file, but a 64 KiB probe
    // is enough for fast-start MP4s and avoids the multi-megabyte read of small
    // files. Real-world downloads are typically fast-start, and the banner
    // display is a best-effort metadata hint, not a full parser.
    let buf = read_file_header(path)?;
    parse_mp4_duration(&buf)
}

/// Parse MP4 buffer for moov > mvhd and extract duration
fn parse_mp4_duration(buf: &[u8]) -> Option<String> {
    let mut i = 0;
    while i < buf.len().saturating_sub(8) {
        let size = u32::from_be_bytes([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]) as usize;
        if size < 8 {
            break;
        }

        // Check for "moov" atom
        if buf[i + 4] == 0x6D && buf[i + 5] == 0x6F && buf[i + 6] == 0x6F && buf[i + 7] == 0x76 {
            // Found moov, scan inside for mvhd
            let mut j = i + 8;
            let moov_end = i + size;

            while j < moov_end.saturating_sub(8) && j < buf.len().saturating_sub(8) {
                let atom_size =
                    u32::from_be_bytes([buf[j], buf[j + 1], buf[j + 2], buf[j + 3]]) as usize;
                if atom_size < 8 || atom_size > size {
                    break;
                }

                // Check for "mvhd" atom
                if buf[j + 4] == 0x6D
                    && buf[j + 5] == 0x76
                    && buf[j + 6] == 0x68
                    && buf[j + 7] == 0x64
                {
                    let version = buf[j + 8];

                    let (timescale, duration) = if version == 0 {
                        let ts = u32::from_be_bytes([
                            buf[j + 20],
                            buf[j + 21],
                            buf[j + 22],
                            buf[j + 23],
                        ]);
                        let dur = u32::from_be_bytes([
                            buf[j + 24],
                            buf[j + 25],
                            buf[j + 26],
                            buf[j + 27],
                        ]);
                        (ts as u64, dur as u64)
                    } else {
                        let ts = u32::from_be_bytes([
                            buf[j + 28],
                            buf[j + 29],
                            buf[j + 30],
                            buf[j + 31],
                        ]);
                        let dur = u64::from_be_bytes([
                            buf[j + 32],
                            buf[j + 33],
                            buf[j + 34],
                            buf[j + 35],
                            buf[j + 36],
                            buf[j + 37],
                            buf[j + 38],
                            buf[j + 39],
                        ]);
                        (ts as u64, dur)
                    };

                    if timescale > 0 && duration > 0 {
                        let seconds = duration / timescale;
                        let mins = seconds / 60;
                        let secs = seconds % 60;
                        if mins >= 60 {
                            let hours = mins / 60;
                            let mins = mins % 60;
                            return Some(format!("{}:{:02}:{:02}", hours, mins, secs));
                        } else if mins > 0 {
                            return Some(format!("{}:{:02}", mins, secs));
                        }
                        return Some(format!("{}s", seconds));
                    }
                }
                j += atom_size;
            }
        }
        i += size;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_image_resolution_png() {
        // Minimal PNG header: 8-byte signature + IHDR chunk
        // Width at bytes 16-19, height at bytes 20-23 (big endian)
        let mut png = vec![0u8; 24];
        // PNG signature
        png[0] = 0x89;
        png[1] = 0x50; // P
        png[2] = 0x4E; // N
        png[3] = 0x47; // G
        png[4] = 0x0D;
        png[5] = 0x0A;
        png[6] = 0x1A;
        png[7] = 0x0A;
        // Width = 1920 (0x00000780)
        png[16] = 0x00;
        png[17] = 0x00;
        png[18] = 0x07;
        png[19] = 0x80;
        // Height = 1080 (0x00000438)
        png[20] = 0x00;
        png[21] = 0x00;
        png[22] = 0x04;
        png[23] = 0x38;

        let result = extract_image_resolution(&png, ".png");
        assert_eq!(result, Some("1920x1080".to_string()));
    }

    #[test]
    fn test_extract_image_resolution_too_short() {
        let png = vec![0u8; 10];
        let result = extract_image_resolution(&png, ".png");
        assert_eq!(result, None);
    }

    #[test]
    fn test_count_zip_entries() {
        // Create a minimal ZIP with 3 local file headers
        let mut zip = Vec::new();
        for _ in 0..3 {
            zip.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]); // Local file header signature
            zip.extend_from_slice(&[0u8; 26]); // Rest of header
        }
        let result = count_zip_entries(&zip);
        assert_eq!(result, Some(3));
    }

    #[test]
    fn test_count_zip_entries_empty() {
        let zip = vec![0u8; 100];
        let result = count_zip_entries(&zip);
        assert_eq!(result, None);
    }

    #[test]
    fn test_count_items_in_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let dir_path = tmp.path();
        std::fs::write(dir_path.join("file1.txt"), "a").unwrap();
        std::fs::write(dir_path.join("file2.txt"), "b").unwrap();
        std::fs::create_dir(dir_path.join("subdir")).unwrap();

        let entry = crate::fs::DirEntry {
            name: "test".to_string(),
            path: dir_path.to_path_buf(),
            is_dir: true,
            is_file: false,
            is_symlink: false,
            is_exec: false,
            size: 0,
            modified: None,
            perms: String::new(),
            owner: String::new(),
            group: String::new(),
            symlink_target: None,
            symlink_valid: true,
            content_probe: None,
        };

        let count = count_items_in_dir(&entry);
        assert_eq!(count, 3); // 2 files + 1 dir
    }

    #[test]
    fn test_count_items_in_nonexistent_dir() {
        let entry = crate::fs::DirEntry {
            name: "nonexistent".to_string(),
            path: "/tmp/nonexistent_dir_12345".into(),
            is_dir: true,
            is_file: false,
            is_symlink: false,
            is_exec: false,
            size: 0,
            modified: None,
            perms: String::new(),
            owner: String::new(),
            group: String::new(),
            symlink_target: None,
            symlink_valid: true,
            content_probe: None,
        };

        let count = count_items_in_dir(&entry);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_file_contents_text() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("test.txt");
        std::fs::write(&file_path, "line1\nline2\nline3\n").unwrap();

        let entry = crate::fs::DirEntry {
            name: "test.txt".to_string(),
            path: file_path,
            is_dir: false,
            is_file: true,
            is_symlink: false,
            is_exec: false,
            size: 18,
            modified: None,
            perms: String::new(),
            owner: String::new(),
            group: String::new(),
            symlink_target: None,
            symlink_valid: true,
            content_probe: None,
        };

        let contents = get_file_contents(&entry);
        assert_eq!(contents, "3"); // 3 lines
    }

    #[test]
    fn test_get_file_contents_empty() {
        let entry = crate::fs::DirEntry {
            name: "unknown.xyz".to_string(),
            path: "/tmp/nonexistent".into(),
            is_dir: false,
            is_file: true,
            is_symlink: false,
            is_exec: false,
            size: 0,
            modified: None,
            perms: String::new(),
            owner: String::new(),
            group: String::new(),
            symlink_target: None,
            symlink_valid: true,
            content_probe: None,
        };

        let contents = get_file_contents(&entry);
        assert_eq!(contents, "");
    }
}
