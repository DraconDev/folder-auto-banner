//! Git integration — branch, status, ahead/behind, dirty state
//!
//! Uses git2 crate for fast, native Git operations.

use anyhow::Result;
use git2::Repository;
use serde::{Deserialize, Serialize};
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
    pub stash_count: usize,
    pub merge_state: Option<String>,
    pub tag: Option<String>,
    pub lines_added: usize,
    pub lines_deleted: usize,
    pub is_dirty: bool,
    pub file_statuses: std::collections::HashMap<String, FileStatus>,
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
            FileStatus::Added => "\u{25cf}",      // ●
            FileStatus::Deleted => "\u{25cf}",    // ●
            FileStatus::Renamed => "\u{25cf}",    // ●
            FileStatus::Untracked => "\u{25cf}",  // ●
            FileStatus::Conflict => "\u{25cf}",   // ●
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
pub fn get_git_info(path: &Path) -> Result<GitInfo> {
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
        .and_then(|h| h.shorthand().map(|s| s.to_string()));

    // Get status
    let mut staged = 0;
    let mut modified = 0;
    let mut untracked = 0;
    let mut file_statuses = std::collections::HashMap::new();

    let statuses = repo.statuses(None).ok();
    if let Some(statuses) = statuses {
        for entry in statuses.iter() {
            let status = entry.status();
            let file_path = entry.path().unwrap_or("").to_string();

            if status.contains(git2::Status::INDEX_NEW)
                || status.contains(git2::Status::INDEX_MODIFIED)
                || status.contains(git2::Status::INDEX_DELETED)
                || status.contains(git2::Status::INDEX_RENAMED)
            {
                staged += 1;
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
            } else if status.contains(git2::Status::WT_MODIFIED)
                || status.contains(git2::Status::WT_DELETED)
                || status.contains(git2::Status::WT_RENAMED)
            {
                modified += 1;
                let fs = if status.contains(git2::Status::WT_DELETED) {
                    FileStatus::Deleted
                } else if status.contains(git2::Status::WT_RENAMED) {
                    FileStatus::Renamed
                } else {
                    FileStatus::Modified
                };
                file_statuses.insert(file_path, fs);
            } else if status.contains(git2::Status::WT_NEW) {
                untracked += 1;
                file_statuses.insert(file_path, FileStatus::Untracked);
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

    // Get last commit message and hash
    let (last_commit_msg, last_commit_hash) = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|commit| {
            let msg = commit
                .message()
                .map(|m| m.lines().next().unwrap_or("").to_string());
            let hash = commit
                .as_object()
                .short_id()
                .ok()
                .map(|id| id.as_str().unwrap_or("").to_string());
            (msg, hash)
        })
        .unwrap_or((None, None));

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
        for tag_name in tags.iter().flatten() {
            if let Ok(tag_ref) = repo.find_reference(&format!("refs/tags/{}", tag_name)) {
                if tag_ref.target() == Some(head_oid) {
                    return Some(tag_name.to_string());
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
        stash_count,
        merge_state,
        tag,
        lines_added,
        lines_deleted,
        is_dirty,
        file_statuses,
    })
}

/// Format git status as compact string (e.g., "[main ↑2 ↓0]")
#[allow(dead_code)]
pub fn format_git_status(info: &GitInfo) -> String {
    if !info.is_repo {
        return String::new();
    }

    let branch = info.branch.as_deref().unwrap_or("?");

    let mut status = String::new();
    status.push_str(&format!("[{}", branch));

    if info.ahead > 0 {
        status.push_str(&format!(" ↑{}", info.ahead));
    }
    if info.behind > 0 {
        status.push_str(&format!(" ↓{}", info.behind));
    }
    if info.modified > 0 {
        status.push_str(&format!(" ✚{}", info.modified));
    }
    if info.untracked > 0 {
        status.push_str(&format!(" ?{}", info.untracked));
    }
    if info.staged > 0 {
        status.push_str(&format!(" ●{}", info.staged));
    }

    status.push(']');
    status
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
