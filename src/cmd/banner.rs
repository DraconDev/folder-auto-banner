//! Banner command — the crown jewel
//!
//! Prints a rich, context-aware directory dashboard and exits.
//! This is the main feature that makes cfm magical.

use anyhow::Result;
use std::path::Path;

use crate::fs::{DirSummary, format_size_compact, format_exact_time};
use crate::git::GitInfo;
use crate::icon;

// ANSI color codes
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const BLUE: &str = "\x1b[34m";
const BLUE_BOLD: &str = "\x1b[1;34m";
const GREEN: &str = "\x1b[32m";
const GREEN_BOLD: &str = "\x1b[1;32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";
const GRAY: &str = "\x1b[90m";

/// Run the banner command
pub fn run_banner(
    path: Option<&Path>,
    raw: bool,
    json: bool,
    compact: bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let path = path.unwrap_or(cwd.as_path()).canonicalize().unwrap_or_else(|_| path.unwrap_or(cwd.as_path()).to_path_buf());

    let summary = DirSummary::scan(&path)?;
    let git_info = crate::git::get_git_info(&path)?;

    if json {
        output_json(&path, &summary, &git_info);
    } else if raw {
        output_raw(&summary);
    } else {
        output_rich(&path, &summary, &git_info, compact);
    }

    Ok(())
}

/// Output rich formatted banner — compact lsd-style layout
fn output_rich(path: &Path, summary: &DirSummary, git_info: &GitInfo, _compact: bool) {
    let path_str = path.to_string_lossy();
    let size_str = format_size_compact(summary.total_size);
    let project_icon = summary.project_type.icon();
    let project_label = summary.project_type.label();

    let home = std::env::var("HOME").unwrap_or_default();
    let path_display = if path_str.starts_with(&home) {
        let relative = &path_str[home.len()..];
        if relative.is_empty() || relative == "/" {
            "~".to_string()
        } else {
            format!("~{}", relative)
        }
    } else {
        path_str.to_string()
    };

    let git_branch = git_info.branch.as_deref().unwrap_or("");
    let hidden_count = summary.top_items.iter().filter(|item| item.name.starts_with('.')).count();

    // Build git branch with color: blue if clean, yellow if dirty
    let branch_display = if !git_branch.is_empty() {
        if git_info.is_dirty {
            format!("{}[{}{}{}]", YELLOW, git_branch, YELLOW, RESET)
        } else {
            format!("{}[{}{}{}]", BLUE_BOLD, git_branch, BLUE_BOLD, RESET)
        }
    } else {
        String::new()
    };

    // Build git status indicators (p10k-style)
    let mut git_status = Vec::new();
    if git_info.modified > 0 {
        git_status.push(format!("{}*{}{}", YELLOW, git_info.modified, RESET));
    }
    if git_info.staged > 0 {
        git_status.push(format!("{}+{}{}", GREEN, git_info.staged, RESET));
    }
    if git_info.untracked > 0 {
        git_status.push(format!("{}?{}{}", DIM, git_info.untracked, RESET));
    }
    if git_info.ahead > 0 {
        git_status.push(format!("{}↑{}{}", CYAN, git_info.ahead, RESET));
    }
    if git_info.behind > 0 {
        git_status.push(format!("{}↓{}{}", RED, git_info.behind, RESET));
    }
    let git_status_str = git_status.join(" ");

    let header = if git_info.is_repo {
        let mut parts = vec![
            format!("{} {} {}", project_icon, path_display, BOLD),
        ];
        if !branch_display.is_empty() {
            parts.push(format!("{} │", branch_display));
        }
        parts.push(format!("{} │", project_label));
        parts.push(format!("{} │", size_str));
        parts.push(format!("{} files │", summary.files));
        parts.push(format!("{} dirs", summary.dirs));
        if !git_status_str.is_empty() {
            parts.push(format!("│ {}", git_status_str));
        }
        parts.join(" ")
    } else {
        let mut parts = vec![
            format!("{} {} {}", project_icon, path_display, BOLD),
        ];
        parts.push(format!("{} │", project_label));
        parts.push(format!("{} │", size_str));
        parts.push(format!("{} files │", summary.files));
        parts.push(format!("{} dirs", summary.dirs));
        if hidden_count > 0 {
            parts.push(format!("│ {} hidden", hidden_count));
        }
        parts.push(format!("│ {} total", summary.total_items));
        parts.join(" ")
    };

    println!("{}", header);
    println!();

    let mut visible_items: Vec<&crate::fs::DirEntry> = Vec::new();
    let mut hidden_items: Vec<&crate::fs::DirEntry> = Vec::new();

    for item in &summary.top_items {
        if item.name.starts_with('.') {
            hidden_items.push(item);
        } else {
            visible_items.push(item);
        }
    }

    let total_visible = visible_items.len();
    let show_hidden = total_visible < 30;

    let mut display_items: Vec<&crate::fs::DirEntry> = if show_hidden {
        visible_items.iter().chain(hidden_items.iter()).copied().collect()
    } else {
        visible_items.to_vec()
    };

    display_items.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            return b.is_dir.cmp(&a.is_dir);
        }
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });

    // Compute max column widths for alignment
    let mut max_perm = 10; // "drwxr-xr-x"
    let mut max_owner = 5; // "OWNER"
    let mut max_group = 5; // "GROUP"
    let mut max_size = 4;  // "SIZE"

    for item in &display_items {
        max_perm = max_perm.max(item.perms.len());
        max_owner = max_owner.max(item.owner.len());
        max_group = max_group.max(item.group.len());
        let size_str = if item.is_dir {
            count_items_in_dir(item).to_string()
        } else {
            format_size_compact(item.size)
        };
        max_size = max_size.max(size_str.len());
    }

    // Print each row — lsd/exa order: PERM OWNER SIZE DATE NAME
    for item in display_items {
        let icon_str = icon::icon_for(&item.name, item.is_dir, item.is_exec, item.is_symlink);

        // Per-file git status icon (with trailing space)
        let git_icon = git_info.file_statuses.get(item.name.as_str()).map(|fs| {
            format!("{}{}{} ", fs.color(), fs.icon(), RESET)
        }).unwrap_or_else(|| " ".to_string());

        // Color the name based on type (like lsd/exa)
        let (name_prefix, name_suffix) = if item.is_dir {
            (BLUE_BOLD, RESET)
        } else if item.is_symlink {
            (MAGENTA, RESET)
        } else if item.is_exec {
            (GREEN_BOLD, RESET)
        } else if item.name.starts_with('.') {
            (DIM, RESET)
        } else {
            ("", "")
        };

        // Build name with optional symlink target
        let name_display = if item.is_symlink {
            if let Some(target) = &item.symlink_target {
                format!("{}{}{} {}→{} {}", name_prefix, item.name, name_suffix, DIM, RESET, target)
            } else {
                format!("{}{}{}", name_prefix, item.name, name_suffix)
            }
        } else {
            format!("{}{}{}", name_prefix, item.name, name_suffix)
        };

        let size_or_count = if item.is_dir {
            count_items_in_dir(item).to_string()
        } else {
            format_size_compact(item.size)
        };

        let modified = item.modified.as_ref()
            .map(|dt| format_exact_time(dt))
            .unwrap_or_default();

        // Pad columns for alignment
        let perm_padded = format!("{:<width$}", item.perms, width = max_perm);
        let owner_padded = format!("{:<width$}", item.owner, width = max_owner);
        let group_padded = format!("{:<width$}", item.group, width = max_group);
        let size_padded = format!("{:>width$}", size_or_count, width = max_size);

        // lsd/exa order: PERM OWNER GROUP SIZE DATE NAME
        println!("{} {} {} {} {}  {} {}{}",
            perm_padded, owner_padded, group_padded, size_padded, modified,
            git_icon, icon_str, name_display);
    }

    if !show_hidden && !hidden_items.is_empty() {
        println!("  ... and {} hidden items ({} total items)", hidden_items.len(), summary.total_items);
    }
}

/// Truncate a string to a given display width, accounting for ANSI escape codes
fn truncate_ansi(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let mut visible_width = 0;
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == '\x1b' {
            result.push(c);
            chars.next();
            while let Some(next) = chars.next() {
                result.push(next);
                if next == 'm' {
                    break;
                }
            }
            continue;
        }

        let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if visible_width + char_width > width.saturating_sub(1) {
            result.push('…');
            return result;
        }

        visible_width += char_width;
        result.push(c);
        chars.next();
    }

    result
}

fn output_raw(summary: &DirSummary) {
    for item in &summary.top_items {
        println!("{}", item.path.display());
    }
}

fn output_json(path: &Path, summary: &DirSummary, git_info: &GitInfo) {
    use serde_json::json;

    let items: Vec<_> = summary.top_items.iter().map(|item| {
        json!({
            "name": item.name,
            "path": item.path.to_string_lossy(),
            "is_dir": item.is_dir,
            "is_symlink": item.is_symlink,
            "is_exec": item.is_exec,
            "size": item.size,
            "perms": item.perms,
            "owner": item.owner,
            "group": item.group,
            "symlink_target": item.symlink_target,
        })
    }).collect();

    let output = json!({
        "path": path.to_string_lossy(),
        "total_items": summary.total_items,
        "total_size": summary.total_size,
        "project_type": summary.project_type.label(),
        "items": items,
        "git": {
            "is_repo": git_info.is_repo,
            "branch": git_info.branch,
            "ahead": git_info.ahead,
            "behind": git_info.behind,
            "modified": git_info.modified,
            "untracked": git_info.untracked,
        }
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn count_items_in_dir(entry: &crate::fs::DirEntry) -> usize {
    std::fs::read_dir(&entry.path)
        .map(|d| d.count())
        .unwrap_or(0)
}
