//! Git integration — branch, status, ahead/behind, dirty state
//!
//! Uses git2 crate for fast, native Git operations.

use anyhow::Result;
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

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

fn get_git_info_inner(
    path: &Path,
    collect_file_statuses: bool,
    filter_paths: &[String],
) -> Result<GitInfo> {
    let mut repo = match Repository::discover(path) {
        Ok(r) => r,
        Err(_) => return Ok(GitInfo::default()),
    };

    // Get stash count first (needs mutable borrow)
    let stash_count = {
        let mut count = 0;
        let _ = repo.stash_foreach(|_, _, _| {
            count += 1;
            true
        });
        count
    };

    let head = repo.head().ok();

    let branch = head
        .as_ref()
        .and_then(|h| h.shorthand().ok().map(|s| s.to_string()));

    // Get status
    let mut staged = 0;
    let mut modified = 0;
    let mut untracked = 0;
    let mut file_statuses = std::collections::HashMap::new();

    let statuses = if !filter_paths.is_empty() {
        // Use pathspec filtering to only collect statuses for paths the banner
        // displays or aggregates. Directories use `dir/*` to limit the walk to
        // immediate children while preserving the existing depth-0/depth-1
        // aggregation behavior.
        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true).recurse_untracked_dirs(false);
        for path in filter_paths {
            opts.pathspec(path);
        }
        repo.statuses(Some(&mut opts)).ok()
    } else {
        // No filter: still limit untracked dir recursion for performance
        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true).recurse_untracked_dirs(false);
        repo.statuses(Some(&mut opts)).ok()
    };

    if let Some(statuses) = statuses {
        for entry in statuses.iter() {
            let status = entry.status();
            let file_path = Path::new(entry.path().unwrap_or(""))
                .to_string_lossy()
                .to_string();

            if status.contains(git2::Status::INDEX_NEW)
                || status.contains(git2::Status::INDEX_MODIFIED)
                || status.contains(git2::Status::INDEX_DELETED)
                || status.contains(git2::Status::INDEX_RENAMED)
            {
                staged += 1;
                if collect_file_statuses {
                    let fs = if status.contains(git2::Status::INDEX_NEW) {
                        FileStatus::Added
                    } else if status.contains(git2::Status::INDEX_DELETED) {
                        FileStatus::Deleted
                    } else if status.contains(git2::Status::INDEX_RENAMED) {
                        FileStatus::Renamed
                    } else {
                        FileStatus::Added
                    };
                    file_statuses.insert(file_path, fs);
                }
            } else if status.contains(git2::Status::WT_MODIFIED)
                || status.contains(git2::Status::WT_DELETED)
                || status.contains(git2::Status::WT_RENAMED)
            {
                modified += 1;
                if collect_file_statuses {
                    let fs = if status.contains(git2::Status::WT_DELETED) {
                        FileStatus::Deleted
                    } else if status.contains(git2::Status::WT_RENAMED) {
                        FileStatus::Renamed
                    } else {
                        FileStatus::Modified
                    };
                    file_statuses.insert(file_path, fs);
                }
            } else if status.contains(git2::Status::WT_NEW) {
                untracked += 1;
                if collect_file_statuses {
                    file_statuses.insert(file_path, FileStatus::Untracked);
                }
            }
        }
    }

    // Get ahead/behind using git2
    let (ahead, behind) = if let Some(head) = head.as_ref() {
        if let Some(head_oid) = head.target() {
            // Try to find upstream branch
            let branch_name = head.shorthand().unwrap_or("");
            let upstream_ref_name = format!("refs/remotes/origin/{}", branch_name);

            if let Ok(upstream_ref) = repo.find_reference(&upstream_ref_name) {
                if let Some(upstream_oid) = upstream_ref.target() {
                    repo.graph_ahead_behind(head_oid, upstream_oid)
                        .unwrap_or((0, 0))
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

    // Get last commit message, hash, and time
    let (last_commit_msg, last_commit_hash, last_commit_time) = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|commit| {
            let msg = commit.message().ok().map(|m| {
                let first_line = m.lines().next().unwrap_or("");
                if first_line.len() > 80 {
                    first_line[..80].to_string()
                } else {
                    first_line.to_string()
                }
            });
            let hash = commit
                .as_object()
                .short_id()
                .ok()
                .map(|id| id.as_str().unwrap_or("").to_string());
            let time = Some(commit.time().seconds());
            (msg, hash, time)
        })
        .unwrap_or((None, None, None));

    // Count commits today
    let commits_today = {
        let mut count = 0;
        let today_start = {
            let now = chrono::Utc::now();
            let today = now.date_naive();
            today
                .and_hms_opt(0, 0, 0)
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or(0)
        };

        let mut revwalk = repo.revwalk().ok();
        if let Some(ref mut walk) = revwalk {
            let _ = walk.push_head();
            for (i, oid) in walk.enumerate() {
                if i >= 1000 {
                    break;
                } // Limit to avoid slow repos
                if let Ok(oid) = oid {
                    if let Ok(commit) = repo.find_commit(oid) {
                        if commit.time().seconds() >= today_start {
                            count += 1;
                        } else {
                            break; // Commits are chronological, stop when we hit yesterday
                        }
                    }
                }
            }
        }
        count
    };

    // Count branches
    let branch_count = {
        let mut count = 0;
        if let Ok(branches) = repo.branches(Some(git2::BranchType::Local)) {
            for _ in branches.flatten() {
                count += 1;
            }
        }
        count
    };

    // Check merge/rebase state
    let merge_state = if repo.state() == git2::RepositoryState::Merge {
        Some("MERGING".to_string())
    } else if repo.state() == git2::RepositoryState::Rebase
        || repo.state() == git2::RepositoryState::RebaseInteractive
        || repo.state() == git2::RepositoryState::RebaseMerge
    {
        Some("REBASING".to_string())
    } else if repo.state() == git2::RepositoryState::CherryPick {
        Some("CHERRY-PICKING".to_string())
    } else if repo.state() == git2::RepositoryState::Revert {
        Some("REVERTING".to_string())
    } else {
        None
    };

    // Get tag at HEAD
    let tag = repo.head().ok().and_then(|h| {
        let head_oid = h.target()?;
        // Check all tags
        let tags = repo.tag_names(None).ok()?;
        for tag_result in tags.iter() {
            let Some(name) = tag_result.ok().flatten() else {
                continue;
            };
            if let Ok(tag_ref) = repo.find_reference(&format!("refs/tags/{}", name)) {
                if tag_ref.target() == Some(head_oid) {
                    return Some(name.to_string());
                }
            }
        }
        None
    });

    // Get diff stats (lines added/deleted)
    let (lines_added, lines_deleted) = if let Ok(head) = repo.head() {
        if let Ok(commit) = head.peel_to_commit() {
            if let Ok(tree) = commit.tree() {
                if let Ok(diff) = repo.diff_tree_to_workdir(Some(&tree), None) {
                    let stats = diff.stats().ok();
                    let added = stats.as_ref().map(|s| s.insertions()).unwrap_or(0);
                    let deleted = stats.as_ref().map(|s| s.deletions()).unwrap_or(0);
                    (added, deleted)
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

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
}
