//! Banner command — the crown jewel
//! 
//! Prints a rich, context-aware directory dashboard and exits.
//! This is the main feature that makes cfm magical.

use anyhow::Result;
use std::path::Path;

use comfy_table::{
    Cell, ColumnConstraint, ContentArrangement, Table, Width, presets,
};
use console::Term;

use crate::fs::{DirSummary, format_size_compact, format_relative_time};
use crate::git::GitInfo;

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
    
    // Build the file listing table with responsive column widths
    let term = Term::stdout();
    let (term_w, _) = term.size();
    let term_width = if term_w > 0 { (term_w as usize).min(120).max(60) } else { 80_usize };

    // Fixed columns: icon(4) + size(10) + modified(14) + border overhead(7) = 35
    // Name gets the rest, minimum 20
    let name_width = term_width.saturating_sub(35).max(20);

    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Disabled)
        .set_width(term_width as u16)
        .set_header(vec![
            Cell::new(""),
            Cell::new("NAME"),
            Cell::new("SIZE"),
            Cell::new("MODIFIED"),
        ]);

    for item in display_items {
        let icon = if item.is_dir { "📂" } else { "📄" };
        let size_or_count = if item.is_dir {
            count_items_in_dir(item).to_string()
        } else {
            format_size_compact(item.size)
        };
        let modified = item.modified.as_ref()
            .map(|dt| format_relative_time(dt))
            .unwrap_or_default();
        let name = truncate(&item.name, name_width);

        table.add_row(vec![
            Cell::new(icon),
            Cell::new(name),
            Cell::new(size_or_count),
            Cell::new(modified),
        ]);
    }

    table.set_constraints(vec![
        ColumnConstraint::Absolute(Width::Fixed(4)),
        ColumnConstraint::Absolute(Width::Fixed(name_width as u16)),
        ColumnConstraint::Absolute(Width::Fixed(10)),
        ColumnConstraint::Absolute(Width::Fixed(14)),
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

fn truncate(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > width {
        format!("{}…", chars[..width.saturating_sub(1)].iter().collect::<String>())
    } else {
        s.to_string()
    }
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
            "size": item.size,
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

