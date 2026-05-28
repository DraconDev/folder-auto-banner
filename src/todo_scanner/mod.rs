//! TODO/FIXME counter — scans source files for task markers
//!
//! Patterns: `- [ ]`, `TODO:`, `FIXME:`, `HACK:`, `XXX:`
//! Skips: node_modules, target, .git, dist, build, vendor, .next
//! Skips binary files
//! Limit: first 1000 files, 1 second timeout
//! Cache: 60 seconds

use anyhow::Result;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

const TODO_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_FILES: usize = 1000;

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "dist",
    "build",
    "vendor",
    ".next",
    "__pycache__",
    ".venv",
    "venv",
];

const BINARY_EXTS: &[&str] = &[
    "exe", "bin", "o", "so", "dll", "dylib", "a", "lib", "obj", "pdb",
    "png", "jpg", "jpeg", "gif", "webp", "ico", "svg", "bmp", "tiff",
    "mp3", "mp4", "avi", "mkv", "mov", "webm", "flac", "wav", "ogg",
    "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "tgz",
    "woff", "woff2", "ttf", "eot",
    "pdf", "doc", "docx", "xls", "xlsx",
    "sqlite", "sqlite3", "db",
];

const TODO_PATTERNS: &[&str] = &[
    "- [ ]",
    "TODO:",
    "TODO ",
    "FIXME:",
    "FIXME ",
    "HACK:",
    "HACK ",
    "XXX:",
    "XXX ",
];

/// TODO scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoInfo {
    pub count: usize,
    pub by_pattern: Vec<(String, usize)>,
}

/// Scan for TODO/FIXME markers in source files
pub fn scan_todos(path: &Path) -> Result<TodoInfo> {
    let start = std::time::Instant::now();
    let mut count = 0;
    let mut by_pattern: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    let walker = WalkBuilder::new(path)
        .hidden(false)
        .ignore(true)
        .git_ignore(true)
        .build();

    let mut files_scanned = 0;

    for entry in walker.flatten() {
        if files_scanned >= MAX_FILES {
            break;
        }

        if start.elapsed() > TODO_TIMEOUT {
            break;
        }

        let file_type = match entry.file_type() {
            Some(ft) => ft,
            None => continue,
        };

        if !file_type.is_file() {
            continue;
        }

        // Skip directories we don't want to scan
        let rel_path = entry.path().strip_prefix(path).unwrap_or(entry.path());
        let components: Vec<_> = rel_path.components().collect();
        let skip = components.iter().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            SKIP_DIRS.contains(&s.as_ref())
        });
        if skip {
            continue;
        }

        // Skip binary files by extension
        if let Some(ext) = entry.path().extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if BINARY_EXTS.contains(&ext_str.as_ref()) {
                continue;
            }
        }

        // Read and scan
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            files_scanned += 1;
            for line in content.lines() {
                for pattern in TODO_PATTERNS {
                    if line.contains(pattern) {
                        count += 1;
                        *by_pattern.entry(pattern.to_string()).or_insert(0) += 1;
                        break; // Count each line only once
                    }
                }
            }
        }
    }

    let by_pattern: Vec<(String, usize)> = by_pattern.into_iter().collect();

    Ok(TodoInfo { count, by_pattern })
}
