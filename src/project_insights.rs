//! Combined project insight scanning.
//!
//! TODO counts and code metrics both need a bounded text-file scan. This module
//! performs that scan once and returns both results, avoiding a second tree walk
//! and a second round of file reads when both insights are enabled.

use anyhow::Result;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use crate::{code_metrics::CodeMetrics, todo_scanner::TodoInfo, utils};

const INSIGHT_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_FILES: usize = 1000;

const TODO_PATTERNS: &[&str] = &[
    "- [ ]", "TODO:", "TODO ", "FIXME:", "FIXME ", "HACK:", "HACK ", "XXX:", "XXX ",
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectInsights {
    pub todos: TodoInfo,
    pub metrics: CodeMetrics,
}

/// Scan TODO markers and code metrics in a single bounded pass.
///
/// `extra_skip_dirs` allows callers (e.g. from config) to add additional
/// directory names to skip beyond the built-in [`utils::SKIP_DIRS`].
pub fn scan_insights(path: &Path, extra_skip_dirs: &[&str]) -> Result<ProjectInsights> {
    let start = std::time::Instant::now();
    let mut todo_count = 0;
    let mut todo_by_pattern: HashMap<String, usize> = HashMap::new();
    let mut total_loc = 0;
    let mut loc_by_extension: HashMap<String, usize> = HashMap::new();
    let mut file_count = 0;
    let mut files_scanned = 0;

    // Merge built-in skip dirs with any caller-provided extras.
    // Use owned Strings so the set can be moved into the WalkBuilder closure
    // (which requires 'static) and reused for the component check below.
    let skip_set: std::collections::HashSet<String> = utils::SKIP_DIRS
        .iter()
        .map(|s| s.to_string())
        .chain(extra_skip_dirs.iter().map(|s| s.to_string()))
        .collect();

    // Clone for the filter_entry closure (which requires 'static ownership)
    let filter_set = skip_set.clone();

    let walker = WalkBuilder::new(path)
        .hidden(false)
        .ignore(true)
        .git_ignore(true)
        .filter_entry(move |entry| {
            // Skip heavy directories that aren't covered by gitignore when
            // the project root is not itself a git repo. This makes the
            // scan behave the same as projects that have a .gitignore.
            let name = entry.file_name().to_string_lossy();
            !filter_set.contains(&name.to_string())
        })
        .build();

    for entry in walker.flatten() {
        if files_scanned >= MAX_FILES {
            break;
        }

        if start.elapsed() > INSIGHT_TIMEOUT {
            break;
        }

        let file_type = match entry.file_type() {
            Some(ft) => ft,
            None => continue,
        };

        if !file_type.is_file() {
            continue;
        }

        let rel_path = entry.path().strip_prefix(path).unwrap_or(entry.path());
        let components: Vec<_> = rel_path.components().collect();
        if components
            .iter()
            .any(|c| skip_set.contains(&c.as_os_str().to_string_lossy().to_string()))
        {
            continue;
        }

        let Some(ext_os) = entry.path().extension() else {
            continue;
        };
        let ext = ext_os.to_string_lossy().to_lowercase();
        if utils::BINARY_EXTS.contains(&ext.as_ref()) {
            continue;
        }

        // Read the file. Cap the read at 256 KiB so a single huge file (e.g.
        // a vendored bundle, a minified asset, or a generated source file)
        // cannot dominate the scan cost. Files larger than this still get
        // counted as a file, but their line/TODO counts are bounded.
        const MAX_INSIGHT_FILE_BYTES: u64 = 256 * 1024;
        let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        if file_size > MAX_INSIGHT_FILE_BYTES {
            files_scanned += 1;
            file_count += 1;
            // Use a conservative per-file line estimate based on file size.
            // We don't have actual line boundaries, so just skip the
            // accurate count and the TODO scan for this file.
            continue;
        }
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };

        files_scanned += 1;
        file_count += 1;
        // Count lines by counting newlines instead of materializing a
        // Vec<&str> from content.lines(), which is a significant allocation
        // for large files like Cargo.lock and large Rust source trees.
        let bytes = content.as_bytes();
        let newline_count = bytes.iter().filter(|&&b| b == b'\n').count();
        let lines = if content.is_empty() {
            0
        } else if content.ends_with('\n') {
            newline_count
        } else {
            // If the file does not end with a newline, the implicit trailing
            // line still counts. This matches what `str::lines().count()` does
            // (each `\n` ends a line; a non-newline-terminated final line also
            // counts as a line).
            newline_count + 1
        };
        total_loc += lines;

        for line in content.lines() {
            for pattern in TODO_PATTERNS {
                if line.contains(pattern) {
                    todo_count += 1;
                    *todo_by_pattern.entry((*pattern).to_string()).or_insert(0) += 1;
                    break;
                }
            }
        }

        let ext_key = if ext.is_empty() {
            "no-ext".to_string()
        } else {
            ext.to_string()
        };
        *loc_by_extension.entry(ext_key).or_insert(0) += lines;
    }

    let todo_by_pattern: Vec<(String, usize)> = todo_by_pattern.into_iter().collect();
    let mut by_extension: Vec<(String, usize)> = loc_by_extension.into_iter().collect();
    by_extension.sort_by_key(|b| std::cmp::Reverse(b.1));
    by_extension.truncate(5);

    Ok(ProjectInsights {
        todos: TodoInfo {
            count: todo_count,
            by_pattern: todo_by_pattern,
        },
        metrics: CodeMetrics {
            total_loc,
            by_extension,
            file_count,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_scan_insights_counts_todos_and_loc() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("main.rs"),
            "fn main() {}\n// TODO: finish\n",
        )
        .unwrap();
        fs::write(tmp.path().join("note.md"), "# Title\n- [ ] task\n").unwrap();
        fs::create_dir(tmp.path().join("target")).unwrap();
        fs::write(tmp.path().join("target/skip.rs"), "TODO: skipped\n").unwrap();

        let insights = scan_insights(tmp.path(), &[]).unwrap();
        assert_eq!(insights.metrics.file_count, 2);
        assert_eq!(insights.metrics.total_loc, 4);
        assert_eq!(insights.todos.count, 2);
    }
}
