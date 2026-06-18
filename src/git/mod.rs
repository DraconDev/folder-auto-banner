//! Git integration — branch, status, ahead/behind, dirty state
//!
//! Uses native `git` subprocess calls instead of libgit2. On large
//! repos (15K+ commits, multi-GB .git) the native git binary is
//! 100-500× faster than libgit2's `repo.statuses()` because git has
//! index/untracked-cache/fsmonitor optimizations that libgit2 lacks.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

/// Git status for a directory
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitInfo {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub last_commit_msg: Option<String>,
    pub last_commit_hash: Option<String>,
    pub last_commit_time: Option<i64>, // Unix timestamp
    pub commits_today: usize,
    pub branch_count: usize,
    pub stash_count: usize,
    pub merge_state: Option<String>,
    pub tag: Option<String>,
    pub lines_added: usize,
    pub lines_deleted: usize,
    pub is_dirty: bool,
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty", default)]
    pub file_statuses: std::collections::HashMap<String, FileStatus>,
}

/// Build git status pathspecs for the banner rows.
///
/// Files use their exact top-level name. Directories use `dir/*` so libgit2
/// only walks the immediate children that the banner can display or aggregate,
/// instead of scanning every nested file under large trees.
pub fn status_filter_paths_for_items(items: &[crate::fs::DirEntry]) -> Vec<String> {
    items
        .iter()
        .map(|item| {
            if item.is_dir {
                format!("{}/*", item.name)
            } else {
                item.name.clone()
            }
        })
        .collect()
}

/// Keep only git status entries that the banner can display or aggregate.
pub fn is_displayed_git_status_path(path_str: &str, keep: &HashSet<String>) -> bool {
    let mut components = Path::new(path_str).components();
    let Some(first) = components.next() else {
        return false;
    };
    let first = first.as_os_str().to_string_lossy();
    if !keep.contains(first.as_ref()) {
        return false;
    }
    components.next().is_none() || components.next().is_none()
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Conflict,
}

impl FileStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            FileStatus::Modified => "\u{25cf}",  // ●
            FileStatus::Added => "\u{25cf}",     // ●
            FileStatus::Deleted => "\u{25cf}",   // ●
            FileStatus::Renamed => "\u{25cf}",   // ●
            FileStatus::Untracked => "\u{25cf}", // ●
            FileStatus::Conflict => "\u{25cf}",  // ●
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            FileStatus::Modified => "\x1b[33m", // yellow
            FileStatus::Added => "\x1b[32m",    // green
            FileStatus::Deleted => "\x1b[31m",  // red
            FileStatus::Renamed => "\x1b[36m",  // cyan
            FileStatus::Untracked => "\x1b[2m", // dim
            FileStatus::Conflict => "\x1b[31m", // red
        }
    }
}

/// Get Git info for a directory
///
/// If `collect_file_statuses` is false, skips building the per-file status map
/// (which can be 39K+ entries for large repos). Use false when you only need
/// aggregate counts (staged/modified/untracked) for the banner header.
pub fn get_git_info(path: &Path) -> Result<GitInfo> {
    get_git_info_inner(path, true, &[])
}

/// Get Git info with optional path filtering for performance.
pub fn get_git_info_filtered(path: &Path, filter_paths: &[String]) -> Result<GitInfo> {
    get_git_info_inner(path, true, filter_paths)
}

/// Run a git command in the given directory and return stdout.
/// Returns None on any error (non-git dir, git not installed, etc.).
fn git_cmd(path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

/// Like `git_cmd` but returns the raw bytes (for when we need exact content).
fn git_cmd_raw(path: &Path, args: &[&str]) -> Option<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .ok()?;
    if output.status.success() {
        Some(output.stdout)
    } else {
        None
    }
}

/// Check if a path is inside a git repository at all.
fn is_git_repo(path: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Discover the .git directory for a given path.
fn git_dir(path: &Path) -> Option<std::path::PathBuf> {
    let out = git_cmd(path, &["rev-parse", "--git-dir"])?;
    let raw = out.trim();
    let p = std::path::PathBuf::from(raw);
    if p.is_absolute() {
        Some(p)
    } else {
        // Relative to cwd; resolve against the repo root
        let root_out = git_cmd(path, &["rev-parse", "--show-toplevel"])?;
        Some(std::path::PathBuf::from(root_out.trim()).join(raw))
    }
}

fn get_git_info_inner(
    path: &Path,
    collect_file_statuses: bool,
    filter_paths: &[String],
) -> Result<GitInfo> {
    // Fast bail: not a git repo
    if !is_git_repo(path) {
        return Ok(GitInfo::default());
    }

    // --- Spawn independent git commands in parallel threads ---
    let path_owned = path.to_owned();
    let filter_owned = filter_paths.to_vec();
    let status_handle = std::thread::spawn({
        let p = path_owned.clone();
        let fp = filter_owned.clone();
        move || git_status(&p, collect_file_statuses, &fp)
    });

    let path_clone = path_owned.clone();
    let branch_handle = std::thread::spawn(move || git_cmd(&path_clone, &["rev-parse", "--abbrev-ref", "HEAD"]));

    let path_clone = path_owned.clone();
    let ahead_behind_handle = std::thread::spawn(move || git_ahead_behind(&path_clone));

    let path_clone = path_owned.clone();
    let last_commit_handle = std::thread::spawn(move || git_last_commit(&path_clone));

    let path_clone = path_owned.clone();
    let stash_handle = std::thread::spawn(move || {
        let out = git_cmd(&path_clone, &["stash", "list"]);
        out.map(|s| s.lines().count()).unwrap_or(0)
    });

    let path_clone = path_owned.clone();
    let commits_today_handle = std::thread::spawn(move || git_commits_today(&path_clone));

    let path_clone = path_owned.clone();
    let branch_count_handle = std::thread::spawn(move || {
        let out = git_cmd(&path_clone, &["branch"]);
        out.map(|s| s.lines().count()).unwrap_or(0)
    });

    let path_clone = path_owned.clone();
    let tag_handle = std::thread::spawn(move || {
        let out = git_cmd(&path_clone, &["tag", "--points-at", "HEAD"]);
        out.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
    });

    let path_clone = path_owned.clone();
    let diff_stats_handle = std::thread::spawn(move || git_diff_stats(&path_clone));

    let merge_state_handle = std::thread::spawn(move || git_merge_state(&path_owned));

    // --- Collect results ---
    let StatusResult {
        staged,
        modified,
        untracked,
        file_statuses,
    } = status_handle.join().unwrap_or_default();

    let branch = branch_handle
        .join()
        .unwrap_or_default()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let (ahead, behind) = ahead_behind_handle.join().unwrap_or((0, 0));
    let (last_commit_msg, last_commit_hash, last_commit_time) =
        last_commit_handle.join().unwrap_or((None, None, None));
    let stash_count = stash_handle.join().unwrap_or(0);
    let commits_today = commits_today_handle.join().unwrap_or(0);
    let branch_count = branch_count_handle.join().unwrap_or(0);
    let tag = tag_handle.join().unwrap_or_default();
    let (lines_added, lines_deleted) = diff_stats_handle.join().unwrap_or((0, 0));
    let merge_state = merge_state_handle.join().unwrap_or_default();

    let is_dirty = staged > 0 || modified > 0 || untracked > 0;

    Ok(GitInfo {
        is_repo: true,
        branch,
        ahead,
        behind,
        staged,
        modified,
        untracked,
        last_commit_msg,
        last_commit_hash,
        last_commit_time,
        commits_today,
        branch_count,
        stash_count,
        merge_state,
        tag,
        lines_added,
        lines_deleted,
        is_dirty,
        file_statuses,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_git_dir() {
        let info = get_git_info(Path::new("/tmp")).unwrap();
        assert!(!info.is_repo);
    }

    #[test]
    fn test_status_filter_paths_for_items() {
        let items = vec![
            crate::fs::DirEntry {
                name: "src".to_string(),
                path: Path::new("src").to_path_buf(),
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
            },
            crate::fs::DirEntry {
                name: "Cargo.toml".to_string(),
                path: Path::new("Cargo.toml").to_path_buf(),
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
            },
        ];

        let paths = status_filter_paths_for_items(&items);
        assert_eq!(paths, vec!["src/*".to_string(), "Cargo.toml".to_string()]);
    }

    #[test]
    fn test_is_displayed_git_status_path() {
        let keep: HashSet<_> = ["src", "Cargo.toml"]
            .into_iter()
            .map(str::to_string)
            .collect();
        assert!(is_displayed_git_status_path("src/lib.rs", &keep));
        assert!(is_displayed_git_status_path("Cargo.toml", &keep));
        assert!(!is_displayed_git_status_path("src/deep/lib.rs", &keep));
        assert!(!is_displayed_git_status_path("tests/lib.rs", &keep));
    }
}
