//! Banner command — the crown jewel
//!
//! Prints a rich, context-aware directory dashboard and exits.
//! This is the main feature that makes cfm magical.

use anyhow::Result;
use std::path::Path;

use comfy_table::{
    Cell, ColumnConstraint, ContentArrangement, Table, Width,
};
use console::Term;

use crate::fs::{DirSummary, format_size_compact, format_relative_time};
use crate::git::GitInfo;
use crate::icon;

// ANSI color codes
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";
const BLUE: &str = "\x1b[34m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";

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

/// Output rich formatted banner - table view like a file manager
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

    let header = if git_info.is_repo {
        let status_parts = [
            if git_info.modified > 0 { format!("{} modified", git_info.modified) } else { String::new() },
            if git_info.untracked > 0 { format!("{} untracked", git_info.untracked) } else { String::new() },
            if git_info.staged > 0 { format!("{} staged", git_info.staged) } else { String::new() },
            if git_info.ahead > 0 { format!("↑{}", git_info.ahead) } else { String::new() },
            if git_info.behind > 0 { format!("↓{}", git_info.behind) } else { String::new() },
        ].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" │ ");

        if git_branch.is_empty() {
            if status_parts.is_empty() {
                format!("{} {} │ {} │ {} │ {} files │ {} dirs",
                    project_icon, path_display, project_label, size_str,
                    summary.files, summary.dirs)
            } else {
                format!("{} {} │ {} │ {} │ {} files │ {} dirs │ {}",
                    project_icon, path_display, project_label, size_str,
                    summary.files, summary.dirs, status_parts)
            }
        } else {
            if status_parts.is_empty() {
                format!("{} {} [{}] │ {} │ {} │ {} files │ {} dirs",
                    project_icon, path_display, git_branch, project_label, size_str,
                    summary.files, summary.dirs)
            } else {
                format!("{} {} [{}] │ {} │ {} │ {} files │ {} dirs │ {}",
                    project_icon, path_display, git_branch, project_label, size_str,
                    summary.files, summary.dirs, status_parts)
            }
        }
    } else {
        if hidden_count > 0 {
            format!("{} {} │ {} │ {} │ {} files │ {} dirs │ {} hidden │ {} total",
                project_icon, path_display, project_label, size_str,
                summary.files, summary.dirs, hidden_count, summary.total_items)
        } else {
            format!("{} {} │ {} │ {} │ {} files │ {} dirs │ {} items",
                project_icon, path_display, project_label, size_str,
                summary.files, summary.dirs, summary.total_items)
        }
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

    // Build the file listing table — icon merged into name, responsive width
    let term = Term::stdout();
    let (tw, _) = term.size();
    let term_width = if tw > 0 { (tw as usize).min(100).max(60) } else { 80_usize };

    // Columns: NAME (rest) + SIZE (12) + MODIFIED (18) + borders/padding (10)
    let overhead = 40;
    let name_width = term_width.saturating_sub(overhead).max(25);

    let mut table = Table::new();
    table
        .load_preset("││──├─┼┤│    ┬┴┌┐└┘")
        .set_content_arrangement(ContentArrangement::Disabled)
        .set_header(vec![
            Cell::new(format!("{}NAME{}", BOLD, RESET)),
            Cell::new(format!("{}SIZE{}", BOLD, RESET)),
            Cell::new(format!("{}MODIFIED{}", BOLD, RESET)),
        ]);

    for item in display_items {
        let icon_str = icon::icon_for(&item.name, item.is_dir, item.is_exec, item.is_symlink);

        // Color: dirs=blue, executables=green, symlinks=magenta, hidden=dim
        let name_color = if item.is_dir {
            BLUE
        } else if item.is_symlink {
            MAGENTA
        } else if item.is_exec {
            GREEN
        } else if item.name.starts_with('.') {
            DIM
        } else {
            ""
        };

        // Build name: "icon colored_name"
        let colored_name = if item.is_symlink {
            if let Some(target) = &item.symlink_target {
                format!("{}{}{} {}→{} {}",
                    name_color, item.name, RESET,
                    DIM, RESET,
                    target)
            } else {
                format!("{}{}{}", name_color, item.name, RESET)
            }
        } else {
            format!("{}{}{}", name_color, item.name, RESET)
        };

        let name_cell = format!("{} {}{}", icon_str, colored_name, RESET);
        let name_display = truncate_ansi(&name_cell, name_width);

        let size_or_count = if item.is_dir {
            count_items_in_dir(item).to_string()
        } else {
            format_size_compact(item.size)
        };

        let modified = item.modified.as_ref()
            .map(|dt| {
                let t = format_relative_time(dt);
                if t == "just now" {
                    format!("{}just now{}", GREEN, RESET)
                } else {
                    t
                }
            })
            .unwrap_or_default();

        table.add_row(vec![
            Cell::new(name_display),
            Cell::new(size_or_count),
            Cell::new(modified),
        ]);
    }

    table.set_constraints(vec![
        ColumnConstraint::Absolute(Width::Fixed(name_width as u16)),
        ColumnConstraint::Absolute(Width::Fixed(12)),
        ColumnConstraint::Absolute(Width::Fixed(18)),
    ]);

    // Prevent row wrapping
    for row in table.row_iter_mut() {
        row.max_height(1);
    }

    println!("{table}");

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
            // ANSI escape sequence — pass through entirely
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

        // Use unicode_width to compute display width of this char
        let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);

        if visible_width + char_width > width.saturating_sub(1) {
            // Truncate — add ellipsis
            result.push('…');
            return result;
        }

        visible_width += char_width;
        result.push(c);
        chars.next();
    }

    result
}

/// Plain-text truncate (no ANSI) — used for non-colored strings
fn truncate(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let mut visible_width = 0;
    let mut result = String::new();

    for c in s.chars() {
        let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if visible_width + char_width > width.saturating_sub(1) {
            result.push('…');
            return result;
        }
        visible_width += char_width;
        result.push(c);
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
