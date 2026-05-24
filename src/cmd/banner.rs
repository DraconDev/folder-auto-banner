//! Banner command — the crown jewel
//! 
//! Prints a rich, context-aware directory dashboard and exits.
//! This is the main feature that makes cfm magical.

use anyhow::Result;
use std::path::Path;
use comfy_table::{Table, Cell};
use console::Term;
use atty::Stream;

use crate::fs::{DirSummary, format_size};
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
    // We check both stdin and stdout for TTY status.
    // If we can't detect either way, we default to rich output for better UX.
    // This ensures the banner looks great when run directly from a terminal.
    let is_stdin_tty = atty::is(atty::Stream::Stdin);
    let is_stdout_tty = atty::is(atty::Stream::Stdout);
    let is_not_tty = !is_stdin_tty && !is_stdout_tty;
    
    if json {
        output_json(&path, &summary, &git_info);
    } else if raw {
        // Explicit --raw flag always means raw
        output_raw(&summary);
    } else if is_not_tty {
        // Definitively piped/redirected - use raw output
        output_raw(&summary);
    } else {
        // User is in a terminal - show rich banner
        output_rich(&path, &summary, &git_info, compact);
    }
    
    Ok(())
}

/// Output rich formatted banner
fn output_rich(path: &Path, summary: &DirSummary, git_info: &GitInfo, compact: bool) {
    let term_width = Term::stdout().size().1 as usize;
    let max_items = if compact { 4 } else { 8 };

    // Header line with path and git status
    let git_status = crate::git::format_git_status(git_info);
    let path_str = path.to_string_lossy();
    
    println!();
    
    // Try to use box drawing, fall back to ASCII if needed
    if supports_box_drawing() {
        print_box_drawing_header(&path_str, &git_status, term_width);
    } else {
        print_ascii_header(&path_str, &git_status, term_width);
    }
    
    // Project type and stats line
    let size_str = format_size(summary.total_size);
    let item_count = format!("{} items", summary.total_items);
    let project_icon = summary.project_type.icon();
    let project_label = summary.project_type.label();
    
    let stats_line = if git_info.is_repo && git_info.last_commit_msg.is_some() {
        let commit_preview = git_info.last_commit_msg.as_ref().unwrap();
        let commit_short = if commit_preview.len() > 40 {
            format!("{}...", &commit_preview[..37])
        } else {
            commit_preview.clone()
        };
        format!("{} {} │ {} │ {}", project_icon, project_label, size_str, commit_short)
    } else {
        format!("{} {} │ {} │ {} items", project_icon, project_label, size_str, item_count)
    };
    
    println!("│ {}", stats_line);
    println!("│ {}", "-".repeat(term_width.saturating_sub(4).max(60)));
    
    // Items grid
    let top_items = summary.top_items(max_items);
    let remaining = summary.remaining(max_items);
    
    if !top_items.is_empty() {
        // Build table
        let mut table = Table::new();
        table.set_header(vec!["", "Name", "Type", "Size"]);
        
        for (i, item) in top_items.iter().enumerate() {
            let icon = if item.is_dir { "📂" } else { "📄" };
            let type_label = if item.is_dir {
                format!("{} item(s)", count_items_in_dir(item))
            } else {
                get_extension_label(&item.name)
            };
            let size_str = if item.is_dir { "-".to_string() } else { format_size(item.size) };
            
            table.add_row(vec![
                Cell::new(icon),
                Cell::new(&item.name),
                Cell::new(&type_label),
                Cell::new(&size_str),
            ]);
        }
        
        println!("{}", table);
    }
    
    // Footer
    if remaining > 0 {
        println!("│ ... and {} more items. (Use 'ls' to see all)", remaining);
    }
    
    // Close box
    if supports_box_drawing() {
        println!("└─{}", "─".repeat(term_width.saturating_sub(2).max(70)));
    } else {
        println!("└{}", "~".repeat(term_width.saturating_sub(2).max(70)));
    }
    println!();
}

/// Print ASCII fallback header
fn print_ascii_header(path: &str, git_status: &str, _term_width: usize) {
    let header = format!("{} {}", path, git_status);
    println!("{}", header);
}

/// Print box drawing header
fn print_box_drawing_header(path: &str, git_status: &str, term_width: usize) {
    let header_text = format!("📂 {} {}", path, git_status);
    let dashes = "─".repeat(term_width.saturating_sub(2).max(header_text.len()));
    println!("┌─{}┐", &dashes[..header_text.len().min(term_width.saturating_sub(4))]);
    println!("│ {}", header_text);
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

/// Check if terminal supports box drawing characters
fn supports_box_drawing() -> bool {
    // Check for NO_COLOR
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    
    // Check TERM
    let term = std::env::var("TERM").unwrap_or_default();
    if term == "dumb" || term.contains("screen") && term.contains("1") {
        return false;
    }
    
    true
}

/// Count items in a directory
fn count_items_in_dir(entry: &crate::fs::DirEntry) -> usize {
    std::fs::read_dir(&entry.path)
        .map(|d| d.count())
        .unwrap_or(0)
}

/// Get extension-based label
fn get_extension_label(name: &str) -> String {
    if let Some(dot) = name.rfind('.') {
        let ext = &name[dot+1..].to_lowercase();
        match ext.as_str() {
            "rs" => "Rust".to_string(),
            "toml" => "Config".to_string(),
            "md" => "Markdown".to_string(),
            "json" => "JSON".to_string(),
            "yaml" | "yml" => "YAML".to_string(),
            "txt" => "Text".to_string(),
            "sh" => "Shell".to_string(),
            "py" => "Python".to_string(),
            "js" | "ts" => "JavaScript".to_string(),
            "lock" => "Lockfile".to_string(),
            _ => ext.to_uppercase(),
        }
    } else {
        "File".to_string()
    }
}