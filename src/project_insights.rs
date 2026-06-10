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

const INSIGHT_TIMEOUT: Duration = Duration::from_millis(500);
const MAX_FILES: usize = 500;

const TODO_PATTERNS: &[&str] = &[
    "- [ ]", "TODO:", "TODO ", "FIXME:", "FIXME ", "HACK:", "HACK ", "XXX:", "XXX ",
];

#[derive(Debug, Clone)]
pub struct ProjectInsights {
    pub todos: TodoInfo,
    pub metrics: CodeMetrics,
}

/// Scan TODO markers and code metrics in a single bounded pass.
pub fn scan_insights(path: &Path) -> Result<ProjectInsights> {
    let start = std::time::Instant::now();
    let mut todo_count = 0;
    let mut todo_by_pattern: HashMap<String, usize> = HashMap::new();
    let mut total_loc = 0;
    let mut loc_by_extension: HashMap<String, usize> = HashMap::new();
    let mut file_count = 0;
    let mut files_scanned = 0;

    let walker = WalkBuilder::new(path)
        .hidden(false)
        .ignore(true)
        .git_ignore(true)
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
            .any(|c| utils::SKIP_DIRS.contains(&c.as_os_str().to_string_lossy().as_ref()))
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

        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };

        files_scanned += 1;
        file_count += 1;
        let lines = content.lines().count();
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

        let insights = scan_insights(tmp.path()).unwrap();
        assert_eq!(insights.metrics.file_count, 2);
        assert_eq!(insights.metrics.total_loc, 4);
        assert_eq!(insights.todos.count, 2);
    }
}
