//! Banner command — the crown jewel
//! 
//! Prints a rich, context-aware directory dashboard and exits.
//! This is the main feature that makes cfm magical.

use anyhow::Result;
use std::path::Path;
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
    
    // Get directory summary
    let summary = DirSummary::scan(&path)?;
    
    // Get git info
    let git_info = crate::git::get_git_info(&path)?;
    
    // Auto-detect based on TTY
    if json {
        output_json(&path, &summary, &git_info);
    } else if raw || !is_stdout_tty {
        output_raw(&summary);
    } else {
        output_rich(&path, &summary, &git_info, compact);
    }
    
    Ok(())
}

/// Output rich formatted banner with table layout
fn output_rich(path: &Path, summary: &DirSummary, git_info: &GitInfo, _compact: bool) {
    let term_width = Term::stdout().size().1 as usize;
    
    // Path and size
    let path_str = path.to_string_lossy();
    let size_str = format_size_compact(summary.total_size);
    let project_icon = summary.project_type.icon();
    let project_label = summary.project_type.label();
    
    // Relative path from home
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
    
    // Build header
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
    
    let header_display: String = header.chars().take(term_width.saturating_sub(2)).collect();
    println!("{}", header_display);
    
    let sep = "─".repeat(term_width.saturating_sub(2).max(60));
    println!("{}", sep);
    
    // Table header - 4 cells per row
    let cell_hdr = format!("{:<16}│{:>9}│{:>7}│{:.<14}", "NAME", "SIZE", "TYPE", "MODIFIED");
    println!("{}   {}   {}   {}", cell_hdr, cell_hdr, cell_hdr, cell_hdr);
    println!("{}", sep);
    
    // Separate items
    let mut visible_items: Vec<&crate::fs::DirEntry> = Vec::new();
    let mut hidden_items: Vec<&crate::fs::DirEntry> = Vec::new();
    
    for item in &summary.top_items {
        if item.name.starts_with('.') {
            hidden_items.push(item);
        } else {
            visible_items.push(item);
        }
    }
    
    // Smart hidden
    let total_visible = visible_items.len();
    let show_hidden = total_visible.div_ceil(4) <= 12 && total_visible < 30;
    
    let display_items = if show_hidden {
        visible_items.iter().chain(hidden_items.iter()).copied().collect()
    } else {
        visible_items.to_vec()
    };
    
    // Table data
    for chunk in display_items.chunks(4) {
        let parts: Vec<String> = chunk.iter().map(|item| {
            if item.is_dir {
                let count = count_items_in_dir(item);
                let count_str = if count == 1 { "1 item" } else { &format!("{} items", count) };
                let name = item.name.chars().take(16).collect::<String>();
                let modified = item.modified.map(|dt| format_relative_time(&dt)).unwrap_or_default();
                format!("📂 {:<16}│{:>9}│{:>7}│{:.<14}", name, count_str, "--", modified)
            } else {
                let size = format_size_compact(item.size);
                let ext = get_extension_label(item);
                let name = item.name.chars().take(16).collect::<String>();
                let modified = item.modified.map(|dt| format_relative_time(&dt)).unwrap_or_default();
                format!("📄 {:<16}│{:>9}│{:>7}│{:.<14}", name, size, ext, modified)
            }
        }).collect();
        println!("{}", parts.join("   "));
    }
    
    if !show_hidden && !hidden_items.is_empty() {
        println!("\n  ... and {} hidden items ({} total items)", hidden_items.len(), summary.total_items);
    }
}

/// Output raw (for piping)
fn output_raw(summary: &DirSummary) {
    for item in &summary.top_items {
        println!("{}", item.path.display());
    }
}

/// Output JSON (for scripting)
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

/// Count items in a directory
fn count_items_in_dir(entry: &crate::fs::DirEntry) -> usize {
    std::fs::read_dir(&entry.path)
        .map(|d| d.count())
        .unwrap_or(0)
}

/// Get extension-based label
fn get_extension_label(item: &crate::fs::DirEntry) -> String {
    let name = &item.name;
    if let Some(dot) = name.rfind('.') {
        let ext = &name[dot+1..].to_lowercase();
        match ext.as_str() {
            "rs" => "Rust".to_string(),
            "toml" => "TOML".to_string(),
            "md" => "MD".to_string(),
            "json" => "JSON".to_string(),
            "yaml" | "yml" => "YAML".to_string(),
            "txt" => "TXT".to_string(),
            "sh" => "SH".to_string(),
            "py" => "Py".to_string(),
            "js" | "ts" => "JS".to_string(),
            "lock" => "Lock".to_string(),
            "gitignore" => "GIT-IGN".to_string(),
            "gitignore-local" => "GIT-IGN".to_string(),
            "gitattributes" => "GIT-ATT".to_string(),
            _ => ext.to_uppercase().chars().take(4).collect(),
        }
    } else {
        "File".to_string()
    }
}
