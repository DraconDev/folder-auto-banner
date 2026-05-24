//! Git integration — branch, status, ahead/behind, dirty state
//! 
//! Uses git2 crate for fast, native Git operations.

use anyhow::{Context, Result};
use git2::{Repository, BranchType};
use std::path::Path;

/// Git status for a directory
#[derive(Debug, Clone, Default)]
pub struct GitInfo {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub last_commit_msg: Option<String>,
    pub is_dirty: bool,
}

/// Get Git info for a directory
pub fn get_git_info(path: &Path) -> Result<GitInfo> {
    let repo = match Repository::discover(path) {
        Ok(r) => r,
        Err(_) => return Ok(GitInfo::default()),
    };

    let head = match repo.head() {
        Ok(h) => Some(h),
        Err(_) => None,
    };

    let branch = head.as_ref().and_then(|h| {
        h.shorthand().map(|s| s.to_string())
    });

    // Get status
    let mut staged = 0;
    let mut modified = 0;
    let mut untracked = 0;

    let statuses = repo.statuses(None).ok();
    if let Some(statuses) = statuses {
        for entry in statuses.iter() {
            let status = entry.status();
            if status.contains(git2::Status::INDEX_NEW)
                || status.contains(git2::Status::INDEX_MODIFIED)
                || status.contains(git2::Status::INDEX_DELETED)
                || status.contains(git2::Status::INDEX_RENAMED)
            {
                staged += 1;
            }
            if status.contains(git2::Status::WT_MODIFIED)
                || status.contains(git2::Status::WT_DELETED)
                || status.contains(git2::Status::WT_RENAMED)
            {
                modified += 1;
            }
            if status.contains(git2::Status::WT_NEW) {
                untracked += 1;
            }
        }
    }

    // Get ahead/behind
    let (ahead, behind) = if let Some(head) = head.as_ref() {
        let head_oid = head.target().unwrap();
        let upstream = repo.find_branch("HEAD", BranchType::Remote)?;
        
        if let Ok(upstream) = upstream {
            let upstream_oid = upstream.target().unwrap();
            let (ahead, behind) = repo.graph_ahead_behind(head_oid, upstream_oid)
                .unwrap_or((0, 0));
            (ahead, behind)
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

    // Get last commit message
    let last_commit_msg = repo.head().ok().and_then(|h| {
        h.peel_to_commit().ok()
    }).and_then(|commit| {
        commit.message().map(|m| m.lines().next().unwrap_or("").to_string())
    });

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
        is_dirty,
    })
}

/// Format git status as compact string (e.g., "[main ↑2 ↓0]")
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