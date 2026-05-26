//! Banner command — the crown jewel
//! 
//! Prints a rich, context-aware directory dashboard and exits.
//! This is the main feature that makes cfm magical.

use anyhow::Result;
use std::path::Path;
use console::Term;

use crate::fs::{DirSummary, format_size_compact};
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
    
    // Output based on flags
    // Priority: --json > --raw > auto-detect
    // Auto-detect: If we can't detect a TTY, default to rich output for better UX.
    // Users in non-TTY environments can use --raw to get clean output.
    let is_stdout_tty = atty::is(atty::Stream::Stdout);
    
    if json {
        output_json(&path, &summary, &git_info);
    } else if raw {
        // Explicit --raw flag always means raw
        output_raw(&summary);
    } else if is_stdout_tty {
        // stdout is a TTY - use rich output
        output_rich(&path, &summary, &git_info, compact);
    } else {
        // Not a TTY (piped/redirected) but no --raw flag given
        // Default to rich output since we can't reliably detect terminal
        // Users who need raw output can use --raw flag
        output_rich(&path, &summary, &git_info, compact);
    }
    
    Ok(())
}

/// Output rich formatted banner — compact, horizontal, show everything
fn output_rich(path: &Path, summary: &DirSummary, git_info: &GitInfo, _compact: bool) {
    let term_width = Term::stdout().size().1 as usize;
    
    // Single-line header: compact, fits terminal
    let path_str = path.to_string_lossy();
    let size_str = format_size_compact(summary.total_size);
    let project_icon = summary.project_type.icon();
    let project_label = summary.project_type.label();
    
    // Truncate path if needed
    let path_display: String = if path_str.chars().count() > 40 {
        let last = path_str.rsplit('/').next().unwrap_or("");
        format!(".../{}", last).chars().take(20).collect()
    } else {
        path_str.to_string()
    };
    
    let header = if git_info.is_repo {
        if let Some(ref msg) = git_info.last_commit_msg {
            let commit_short: String = msg.chars().take(12).collect();
            let commit_display = if commit_short.len() < msg.len() {
                format!("{}...", commit_short)
            } else {
                commit_short
            };
            format!("{} {} │ {} │ {}", 
                project_icon, path_display, project_label, size_str)
        } else {
            format!("{} {} │ {} │ {} │ {} files │ {} dirs │ {} items", 
                project_icon, path_display, project_label, size_str,
                summary.files, summary.dirs, summary.total_items)
        }
    } else {
        format!("{} {} │ {} │ {} │ {} files │ {} dirs │ {} items", 
            project_icon, path_display, project_label, size_str,
            summary.files, summary.dirs, summary.total_items)
    };
    
    let header_display: String = header.chars().take(term_width.saturating_sub(2)).collect();
    println!("{}", header_display);
    println!("{}", "─".repeat(term_width.saturating_sub(2).max(60)));
    
    // Show ALL items, one per line, compact
    let _remaining = summary.total_items; // total item count for reference
    for item in &summary.top_items {
        if item.is_dir {
            let count = count_items_in_dir(item);
            let name = item.name.chars().take(40).collect::<String>();
            let count_str = if count == 1 { "item" } else { "items" };
            println!("  📂 {:<40} {} {}", name, count, count_str);
        } else {
            let size = format_size_compact(item.size);
            let ext = get_extension_label(&item.name);
            let name = item.name.chars().take(40).collect::<String>();
            println!("  📄 {:<40} {:>8}  {}", name, size, ext);
        }
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

/// Get extension-based label — compact
fn get_extension_label(name: &str) -> String {
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