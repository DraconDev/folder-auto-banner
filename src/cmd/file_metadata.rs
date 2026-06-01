//! File metadata extraction — image resolution, ZIP entries, SQLite tables, video duration
//!
//! These functions read file contents to extract metadata for display in the banner.
//! Extracted from banner.rs to decouple I/O from rendering.

use std::path::Path;

/// Get contents description for a file — line count for text, resolution for image, etc.
/// Returns plain text (no ANSI codes) — coloring is applied by the renderer.
#[allow(dead_code)]
pub fn get_file_contents(entry: &crate::fs::DirEntry) -> String {
    let name = &entry.name;
    let lower = name.to_lowercase();

    // Image files: try to get resolution from header
    if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        if let Ok(bytes) = std::fs::read(&entry.path) {
            if let Some(res) = extract_image_resolution(&bytes, &lower) {
                return res;
            }
        }
    }

    // ZIP files: count entries
    if lower.ends_with(".zip") {
        if let Ok(bytes) = std::fs::read(&entry.path) {
            if let Some(count) = count_zip_entries(&bytes) {
                return count.to_string();
            }
        }
    }

    // SQLite DB: show table count
    if lower.ends_with(".db") || lower.ends_with(".sqlite") || lower.ends_with(".sqlite3") {
        if let Some(count) = count_sqlite_tables(&entry.path) {
            return format!("{}t", count);
        }
    }

    // Video files: extract duration from container headers
    if lower.ends_with(".mp4") || lower.ends_with(".mov") || lower.ends_with(".m4v") {
        if let Some(dur) = extract_video_duration(&entry.path) {
            return dur;
        }
    }

    // Text files under 1MB: count lines
    if entry.size < 1024 * 1024 {
        if let Ok(content) = std::fs::read_to_string(&entry.path) {
            let lines = content.lines().count();
            return lines.to_string();
        }
    }

    // WebM/MKV: extract duration from EBML headers
    if lower.ends_with(".webm") || lower.ends_with(".mkv") {
        if let Some(dur) = extract_video_duration(&entry.path) {
            return dur;
        }
    }

    String::new()
}

/// Count items in a directory
pub fn count_items_in_dir(entry: &crate::fs::DirEntry) -> usize {
    std::fs::read_dir(&entry.path)
        .map(|d| d.count())
        .unwrap_or(0)
}

/// Extract image resolution from PNG or JPEG header bytes
fn extract_image_resolution(bytes: &[u8], ext: &str) -> Option<String> {
    if ext.ends_with(".png") && bytes.len() >= 24 {
        // PNG: width at offset 16-19 (big endian), height at 20-23
        let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) as usize;
        let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]) as usize;
        if w > 0 && h > 0 {
            return Some(format!("{}x{}", w, h));
        }
    } else if ext.ends_with(".jpg") || ext.ends_with(".jpeg") {
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
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path).ok()?;
    let file_len = file.metadata().ok()?.len();

    // For files under 100MB, just read the whole thing - fast and reliable
    if file_len <= 100 * 1024 * 1024 {
        let mut buf = Vec::with_capacity(file_len as usize);
        file.read_to_end(&mut buf).ok()?;
        return parse_mp4_duration(&buf);
    }

    // For very large files, read 50MB from start and 50MB from end
    let chunk_size = 50 * 1024 * 1024;

    let mut buf = vec![0u8; chunk_size];
    let bytes_read = file.read(&mut buf).ok()?;
    buf.truncate(bytes_read);

    if let Some(dur) = parse_mp4_duration(&buf) {
        return Some(dur);
    }

    file.seek(SeekFrom::Start(file_len - chunk_size as u64))
        .ok()?;
    let mut buf = vec![0u8; chunk_size];
    let bytes_read = file.read(&mut buf).ok()?;
    buf.truncate(bytes_read);

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
        };

        let contents = get_file_contents(&entry);
        assert_eq!(contents, "");
    }
}
