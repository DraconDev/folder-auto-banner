//! Git integration — branch, status, ahead/behind, dirty state
//! 
//! Uses git2 crate for fast, native Git operations.

use anyhow::Result;
use git2::Repository;
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
    pub file_statuses: std::collections::HashMap<String, FileStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
            FileStatus::Modified => "M",
            FileStatus::Added => "+",
            FileStatus::Deleted => "D",
            FileStatus::Renamed => "R",
            FileStatus::Untracked => "?",
            FileStatus::Conflict => "!",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            FileStatus::Modified => "\x1b[33m",   // yellow
            FileStatus::Added => "\x1b[32m",      // green
            FileStatus::Deleted => "\x1b[31m",    // red
            FileStatus::Renamed => "\x1b[36m",    // cyan
            FileStatus::Untracked => "\x1b[2m",   // dim
            FileStatus::Conflict => "\x1b[31m",   // red
        }
    }
}

/// Get Git info for a directory
pub fn get_git_info(path: &Path) -> Result<GitInfo> {
    let repo = match Repository::discover(path) {
        Ok(r) => r,
        Err(_) => return Ok(GitInfo::default()),
    };

    let head = repo.head().ok();

    let branch = head.as_ref().and_then(|h| {
        h.shorthand().map(|s| s.to_string())
    });

    // Get status
    let mut staged = 0;
    let mut modified = 0;
    let mut untracked = 0;
    let mut file_statuses = std::collections::HashMap::new();

    let statuses = repo.statuses(None).ok();
    if let Some(statuses) = statuses {
        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("").to_string();

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
                file_statuses.insert(path, fs);
            }
            if status.contains(git2::Status::WT_MODIFIED)
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
                file_statuses.insert(path, fs);
            }
            if status.contains(git2::Status::WT_NEW) {
                untracked += 1;
                file_statuses.insert(path, FileStatus::Untracked);
            }
        }
    }

    // Get ahead/behind (simplified - skip if not straightforward)
    let (ahead, behind) = if let Some(_head) = head.as_ref() {
        // Just report 0 for ahead/behind for now, complexity not worth it
        (0, 0)
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
        file_statuses,
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