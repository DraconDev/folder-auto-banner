//! Code metrics — counts lines of code and files by extension
//!
//! Scans source files for LOC counts.
//! Skips: node_modules, target, .git, dist, build, vendor, .next
//! Limit: first 1000 files, 1 second timeout
//! Cache: 60 seconds

use anyhow::Result;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use crate::utils;

const METRICS_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_FILES: usize = 1000;

/// Code metrics result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub total_loc: usize,
    pub by_extension: Vec<(String, usize)>,
    pub file_count: usize,
}

/// Scan for code metrics
pub fn scan_metrics(path: &Path) -> Result<CodeMetrics> {
    let start = std::time::Instant::now();
    let mut total_loc = 0;
    let mut by_extension: HashMap<String, usize> = HashMap::new();
    let mut file_count = 0;

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

        if start.elapsed() > METRICS_TIMEOUT {
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
            utils::SKIP_DIRS.contains(&s.as_ref())
        });
        if skip {
            continue;
        }

        // Skip binary files by extension
        let ext = entry
            .path()
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if utils::BINARY_EXTS.contains(&ext.as_ref()) {
            continue;
        }

        // Read and count lines
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            files_scanned += 1;
            file_count += 1;
            let lines = content.lines().count();
            total_loc += lines;

            let ext_key = if ext.is_empty() {
                "no-ext".to_string()
            } else {
                ext.to_string()
            };
            *by_extension.entry(ext_key).or_insert(0) += lines;
        }
    }

    // Sort by LOC descending, take top 5
    let mut by_extension: Vec<(String, usize)> = by_extension.into_iter().collect();
    by_extension.sort_by_key(|b| std::cmp::Reverse(b.1));
    by_extension.truncate(5);

    Ok(CodeMetrics {
        total_loc,
        by_extension,
        file_count,
    })
}
