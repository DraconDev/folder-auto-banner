//! Banner command — the crown jewel
//! 
//! Prints a rich, context-aware directory dashboard and exits.
//! This is the main feature that makes cfm magical.

use anyhow::Result;
use std::path::Path;

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

/// Output rich formatted banner - list view like a file manager
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
    println!("{}", "─".repeat(90));
    println!("{:<4}  {:<32}  {:>10}  {:<8}  {:<15}", " ", "NAME", "COUNT/SIZE", "TYPE", "MODIFIED");
    println!("{}", "─".repeat(90));
    
    for item in display_items {
        if item.is_dir {
            let count = count_items_in_dir(item);
            let count_str = if count == 1 { "1" } else { &format!("{}", count) };
            let modified = item.modified.map(|dt| format_relative_time(&dt)).unwrap_or_default();
            println!("{:<4}  {:<32}  {:>5} items  {:<8}  {}", "d--", item.name, count_str, "[DIR]", modified);
        } else {
            let size = format_size_compact(item.size);
            let ext = get_extension_label(item);
            let modified = item.modified.map(|dt| format_relative_time(&dt)).unwrap_or_default();
            println!("{:<4}  {:<32}  {:>8}  {:<8}  {}", "---", item.name, size, ext, modified);
        }
    }
    
    if !show_hidden && !hidden_items.is_empty() {
        println!("
  ... and {} hidden items ({} total items)", hidden_items.len(), summary.total_items);
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
