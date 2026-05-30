//! Banner command — the crown jewel
//!
//! Prints a rich, context-aware directory dashboard and exits.
//! This is the main feature that makes cfm magical.

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::Path;

use crate::fs::{format_exact_time, format_size_compact, DirSummary};
use crate::git::GitInfo;
use crate::icon;

// ANSI color codes — only emitted when stdout is a tty
fn color(code: &str) -> &str {
    code
}

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const BLUE: &str = "\x1b[34m";
const BLUE_BOLD: &str = "\x1b[1;34m";
const GREEN: &str = "\x1b[32m";
#[allow(dead_code)]
const GREEN_BOLD: &str = "\x1b[1;32m";
const YELLOW: &str = "\x1b[33m";
#[allow(dead_code)]
const YELLOW_BOLD: &str = "\x1b[1;33m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";
#[allow(dead_code)]
const GRAY: &str = "\x1b[90m";
#[allow(dead_code)]
const WHITE: &str = "\x1b[97m";
const ORANGE: &str = "\x1b[38;5;214m";
const ROW_TINT: &str = "\x1b[48;5;236m"; // subtle dark gray for alternating rows

/// Options for the banner command
#[derive(Default)]
pub struct BannerOptions<'a> {
    pub path: Option<&'a Path>,
    pub raw: bool,
    pub json: bool,
    pub compact: bool,
    #[allow(dead_code)]
    pub no_build_check: bool,
    pub no_todos: bool,
    pub no_ports: bool,
    pub no_docker: bool,
    pub no_metrics: bool,
    pub sort: Option<&'a str>,
    pub reverse: bool,
}

fn colorize_date(_dt: &DateTime<Utc>, formatted: &str) -> String {
    format!("{}{}{}", color(GREEN), formatted, color(RESET))
}

pub fn run_banner(opts: &BannerOptions) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let path = opts
        .path
        .unwrap_or(cwd.as_path())
        .canonicalize()
        .unwrap_or_else(|_| opts.path.unwrap_or(cwd.as_path()).to_path_buf());

    // Try daemon cache — if daemon isn't running, start it and retry
    if let Some(cached) = crate::daemon_client::get_banner_cached(&path) {
        let summary = cached.summary;
        let git_info = cached.git_info.unwrap_or_default();

        if opts.json {
            output_json(&path, &summary, &git_info);
        } else if opts.raw {
            output_raw(&summary);
        } else {
            output_rich(&path, &summary, &git_info, opts.compact, opts.sort, opts.reverse);
        }

        // Warm daemon cache for likely next directories (parent + siblings)
        warm_nearby_dirs(&path);
        return Ok(());
    }

    // Daemon not available or cache miss — try direct scan
    eprintln!("cfmd: daemon not available, falling back to direct scan");
    let no_todos = opts.no_todos || std::env::var("CFM_NO_TODOS").unwrap_or_default() == "1";
    let no_ports = opts.no_ports || std::env::var("CFM_NO_PORTS").unwrap_or_default() == "1";
    let no_docker = opts.no_docker || std::env::var("CFM_NO_DOCKER").unwrap_or_default() == "1";
    let no_metrics = opts.no_metrics || std::env::var("CFM_NO_METRICS").unwrap_or_default() == "1";

    let summary = DirSummary::scan_with_options(
        &path,
        false, // build check disabled by default — too slow (cargo check = 6.7s)
        !no_todos,
        !no_ports,
        !no_docker,
        !no_metrics,
    )?;
    let git_info = crate::git::get_git_info(&path)?;

    // Display the banner
    if opts.json {
        output_json(&path, &summary, &git_info);
    } else if opts.raw {
        output_raw(&summary);
    } else {
        output_rich(&path, &summary, &git_info, opts.compact, opts.sort, opts.reverse);
    }

    // Warm daemon cache for likely next directories (parent + siblings)
    warm_nearby_dirs(&path);

    Ok(())
}

/// Pre-compute banners for parent and sibling directories
fn warm_nearby_dirs(path: &Path) {
    let Some(parent) = path.parent() else { return };
    if !parent.is_dir() { return; }

    // Warm parent
    crate::daemon_client::send_warm(parent);

    // Warm siblings (up to 20, to avoid overwhelming the daemon)
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten().take(20) {
            if entry.path().is_dir() && entry.path() != path {
                crate::daemon_client::send_warm(&entry.path());
            }
        }
    }
}

/// Output rich formatted banner — compact lsd-style layout
fn output_rich(
    path: &Path,
    summary: &DirSummary,
    git_info: &GitInfo,
    _compact: bool,
    sort: Option<&str>,
    reverse: bool,
) {
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
    let hidden_count = summary
        .top_items
        .iter()
        .filter(|item| item.name.starts_with('.'))
        .count();

    // Build git branch with color: blue if clean, yellow if dirty
    let branch_display = if !git_branch.is_empty() {
        if git_info.is_dirty {
            format!("{}[{}*{}]", color(YELLOW), git_branch, color(RESET))
        } else {
            format!("{}[{}{}]", color(BLUE_BOLD), git_branch, color(RESET))
        }
    } else {
        String::new()
    };

    // Build git status indicators (p10k-style)
    let mut git_status = Vec::new();
    if git_info.modified > 0 {
        git_status.push(format!(
            "{}*{}{}",
            color(YELLOW),
            git_info.modified,
            color(RESET)
        ));
    }
    if git_info.staged > 0 {
        git_status.push(format!(
            "{}+{}{}",
            color(GREEN),
            git_info.staged,
            color(RESET)
        ));
    }
    if git_info.untracked > 0 {
        git_status.push(format!(
            "{}?{}{}",
            color(DIM),
            git_info.untracked,
            color(RESET)
        ));
    }
    if git_info.ahead > 0 {
        git_status.push(format!(
            "{}↑{}{}",
            color(CYAN),
            git_info.ahead,
            color(RESET)
        ));
    }
    if git_info.behind > 0 {
        git_status.push(format!(
            "{}↓{}{}",
            color(RED),
            git_info.behind,
            color(RESET)
        ));
    }
    if git_info.stash_count > 0 {
        git_status.push(format!(
            "{}≡{}{}",
            color(MAGENTA),
            git_info.stash_count,
            color(RESET)
        ));
    }
    if let Some(ref state) = git_info.merge_state {
        git_status.push(format!("{}{}{}", color(RED), state, color(RESET)));
    }
    let git_status_str = git_status.join(" ");

    let header = if git_info.is_repo {
        let mut parts = vec![format!("{} {} {}", project_icon, path_display, color(BOLD))];
        if !branch_display.is_empty() {
            parts.push(format!("{} │", branch_display));
        }
        if let Some(ref tag) = git_info.tag {
            parts.push(format!("{}{}{} │", color(YELLOW), tag, color(RESET)));
        }
        parts.push(format!("{} │", project_label));
        parts.push(format!("{}💾 {}{} │", color(CYAN), size_str, color(RESET)));
        parts.push(format!(
            "{}📄 {} files{} │",
            color(DIM),
            summary.files,
            color(RESET)
        ));
        parts.push(format!(
            "{}📂 {} dirs{}",
            color(DIM),
            summary.dirs,
            color(RESET)
        ));
        if !git_status_str.is_empty() {
            parts.push(format!("│ {}", git_status_str));
        }
        // Build status
        if let Some(ref build) = summary.build_status {
            if build.ok {
                parts.push(format!("│ {}✓ builds{}", color(GREEN), color(RESET)));
            } else {
                let err_str = if build.errors > 0 {
                    format!(" ({} err)", build.errors)
                } else {
                    String::new()
                };
                parts.push(format!(
                    "│ {}✗ build errors{}{}",
                    color(RED),
                    err_str,
                    color(RESET)
                ));
            }
        }
        // TODO count
        if let Some(ref todos) = summary.todo_info {
            if todos.count > 0 {
                parts.push(format!(
                    "│ {}📝 {} TODOs{}",
                    color(YELLOW),
                    todos.count,
                    color(RESET)
                ));
            }
        }
        // Port usage
        if let Some(ref ports) = summary.port_info {
            if !ports.ports.is_empty() {
                let port_str: Vec<String> = ports.ports.iter().map(|p| format!(":{}", p)).collect();
                parts.push(format!(
                    "│ {}🔌 {}{}",
                    color(CYAN),
                    port_str.join(", "),
                    color(RESET)
                ));
            }
        }
        // Docker info
        if let Some(ref docker) = summary.docker_info {
            let running = docker
                .containers
                .iter()
                .filter(|c| c.status.contains("Up"))
                .count();
            let total = docker.containers.len();
            if total > 0 {
                parts.push(format!(
                    "│ {}🐳 {} containers ({} running){}",
                    color(BLUE),
                    total,
                    running,
                    color(RESET)
                ));
            } else if docker.has_compose || docker.has_dockerfile {
                parts.push(format!("│ {}🐳 docker{}", color(DIM), color(RESET)));
            }
        }
        // Code metrics — just total lines, no breakdown
        if let Some(ref metrics) = summary.code_metrics {
            if metrics.total_loc > 0 {
                let loc_str = format_loc(metrics.total_loc);
                parts.push(format!(
                    "│ {}📊 {} lines{}",
                    color(GREEN),
                    loc_str,
                    color(RESET)
                ));
            }
        }
        // Diff stats
        if git_info.lines_added > 0 || git_info.lines_deleted > 0 {
            parts.push(format!(
                "│ {}+{}{} {}-{}{}",
                color(GREEN),
                git_info.lines_added,
                color(RESET),
                color(RED),
                git_info.lines_deleted,
                color(RESET)
            ));
        }
        // Clean indicator
        if !git_info.is_dirty
            && git_info.modified == 0
            && git_info.staged == 0
            && git_info.untracked == 0
        {
            parts.push(format!("│ {}✓ clean{}", color(GREEN), color(RESET)));
        }
        parts.join(" ")
    } else {
        let mut parts = vec![format!("{} {} {}", project_icon, path_display, color(BOLD))];
        parts.push(format!("{} │", project_label));
        parts.push(format!("{}💾 {}{} │", color(CYAN), size_str, color(RESET)));
        parts.push(format!(
            "{}📄 {} files{} │",
            color(DIM),
            summary.files,
            color(RESET)
        ));
        parts.push(format!(
            "{}📂 {} dirs{}",
            color(DIM),
            summary.dirs,
            color(RESET)
        ));
        if hidden_count > 0 {
            parts.push(format!("│ {} hidden", hidden_count));
        }
        // Build status (non-git)
        if let Some(ref build) = summary.build_status {
            if build.ok {
                parts.push(format!("│ {}✓ builds{}", color(GREEN), color(RESET)));
            } else {
                parts.push(format!("│ {}✗ build errors{}", color(RED), color(RESET)));
            }
        }
        // TODO count (non-git)
        if let Some(ref todos) = summary.todo_info {
            if todos.count > 0 {
                parts.push(format!(
                    "│ {}📝 {} TODOs{}",
                    color(YELLOW),
                    todos.count,
                    color(RESET)
                ));
            }
        }
        // Port usage (non-git)
        if let Some(ref ports) = summary.port_info {
            if !ports.ports.is_empty() {
                let port_str: Vec<String> = ports.ports.iter().map(|p| format!(":{}", p)).collect();
                parts.push(format!(
                    "│ {}🔌 {}{}",
                    color(CYAN),
                    port_str.join(", "),
                    color(RESET)
                ));
            }
        }
        // Docker info (non-git)
        if let Some(ref docker) = summary.docker_info {
            let running = docker
                .containers
                .iter()
                .filter(|c| c.status.contains("Up"))
                .count();
            let total = docker.containers.len();
            if total > 0 {
                parts.push(format!(
                    "│ {}🐳 {} containers ({} running){}",
                    color(BLUE),
                    total,
                    running,
                    color(RESET)
                ));
            } else if docker.has_compose || docker.has_dockerfile {
                parts.push(format!("│ {}🐳 docker{}", color(DIM), color(RESET)));
            }
        }
        // Code metrics (non-git)
        if let Some(ref metrics) = summary.code_metrics {
            if metrics.total_loc > 0 {
                let loc_str = format_loc(metrics.total_loc);
                parts.push(format!(
                    "│ {}📊 {} lines{}",
                    color(GREEN),
                    loc_str,
                    color(RESET)
                ));
            }
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
        visible_items
            .iter()
            .chain(hidden_items.iter())
            .copied()
            .collect()
    } else {
        visible_items.to_vec()
    };

    // Sort based on --sort flag
    let sort_mode = sort.unwrap_or("name");
    display_items.sort_by(|a, b| {
        // Always keep directories first unless sorting by type
        if sort_mode != "type" && a.is_dir != b.is_dir {
            return if reverse {
                a.is_dir.cmp(&b.is_dir)
            } else {
                b.is_dir.cmp(&a.is_dir)
            };
        }

        let ordering = match sort_mode {
            "size" => a.size.cmp(&b.size),
            "date" => {
                let a_time = a
                    .modified
                    .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
                let b_time = b
                    .modified
                    .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
                a_time.cmp(&b_time)
            }
            "type" => {
                let a_ext = a.name.rfind('.').map(|i| &a.name[i..]).unwrap_or("");
                let b_ext = b.name.rfind('.').map(|i| &b.name[i..]).unwrap_or("");
                let ext_cmp = a_ext.cmp(b_ext);
                if ext_cmp != std::cmp::Ordering::Equal {
                    ext_cmp
                } else {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                }
            }
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()), // "name" or default
        };

        if reverse {
            ordering.reverse()
        } else {
            ordering
        }
    });

    // Compute max column widths for alignment
    let mut max_owner = 5; // "OWNER"
    let mut max_group = 5; // "GROUP"
    let mut max_size = 4; // "SIZE"
    let mut max_contents = 4; // dynamic
    let mut max_git = 1; // git status icon (always 1 char)

    for item in &display_items {
        max_owner = max_owner.max(item.owner.len());
        max_group = max_group.max(item.group.len());
        let size_str = format_size_compact(item.size);
        max_size = max_size.max(size_str.len());
        let contents_len = if item.is_dir {
            count_items_in_dir(item).to_string().len()
        } else {
            get_file_contents_raw(item).len()
        };
        max_contents = max_contents.max(contents_len.max(4));
        // Git status is always 1 char, but we need a column for it
        max_git = 1;
    }

    // Print each row — PERM OWNER GROUP CONTENTS SIZE DATE NAME
    for (idx, item) in display_items.iter().enumerate() {
        let row_tint = if idx % 2 == 0 { ROW_TINT } else { "" };
        let tint_reset = if idx % 2 == 0 { color(RESET) } else { "" };
        let icon_str = icon::icon_for(&item.name, item.is_dir, item.is_exec, item.is_symlink);

        // Per-file git status — try relative path first, then filename
        // For directories, aggregate status from child files
        let git_icon = {
            let rel = item.path.strip_prefix(path).unwrap_or(&item.path);
            let rel_str = rel.to_string_lossy();
            git_info
                .file_statuses
                .get(rel_str.as_ref())
                .or_else(|| git_info.file_statuses.get(item.name.as_str()))
                .map(|fs| format!("{}{}{}", color(fs.color()), fs.icon(), color(RESET)))
                .or_else(|| {
                    // For directories: aggregate status from child files
                    if item.is_dir {
                        aggregate_dir_git_status(&git_info.file_statuses, &item.name)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| {
                    // Tracked but clean — show dim dot
                    if git_info.is_repo {
                        format!("{}●{}", color(GREEN), color(RESET))
                    } else {
                        String::new()
                    }
                })
        };

        // Color the name based on type (like lsd/exa)
        let (name_prefix, name_suffix) = if item.is_dir {
            (color(BLUE_BOLD), color(RESET))
        } else if item.is_symlink {
            (color(MAGENTA), color(RESET))
        } else if item.is_exec {
            (color(RED), color(RESET))
        } else if item.name.starts_with('.') {
            (color(DIM), color(RESET))
        } else {
            // Color by file extension
            let ext = item.name.rfind('.').map(|i| &item.name[i..]).unwrap_or("");
            match ext {
                // Scripts - red (execution risk)
                ".sh" | ".bash" | ".py" | ".rb" | ".pl" | ".php" | ".js" | ".ts" | ".jsx"
                | ".tsx" | ".ruby" | ".perl" | ".lua" | ".r" | ".R" | ".ps1" | ".bat" | ".cmd" => {
                    (color(RED), color(RESET))
                }
                // Config files - dim
                ".json" | ".yaml" | ".yml" | ".toml" | ".ini" | ".conf" | ".cfg" | ".env" => {
                    (color(DIM), color(RESET))
                }
                // Documentation - cyan
                ".md" | ".txt" | ".rst" | ".doc" | ".docx" | ".pdf" => (color(CYAN), color(RESET)),
                // Source code - green
                ".rs" | ".go" | ".java" | ".c" | ".cpp" | ".h" | ".hpp" | ".cs" | ".swift"
                | ".kt" | ".scala" | ".zig" | ".nim" => (color(GREEN), color(RESET)),
                // Images - magenta
                ".png" | ".jpg" | ".jpeg" | ".gif" | ".svg" | ".webp" | ".ico" => {
                    (color(MAGENTA), color(RESET))
                }
                // Videos - cyan
                ".mp4" | ".mkv" | ".avi" | ".mov" | ".webm" | ".flv" => (color(CYAN), color(RESET)),
                // Archives - dim
                ".zip" | ".tar" | ".gz" | ".bz2" | ".xz" | ".7z" | ".rar" | ".tgz" => {
                    (color(DIM), color(RESET))
                }
                // Default - no color
                _ => ("", ""),
            }
        };

        // Build name with optional symlink target
        let name_display = if item.is_symlink {
            if let Some(target) = &item.symlink_target {
                format!(
                    "{}{}{} {}→{} {}",
                    name_prefix,
                    item.name,
                    name_suffix,
                    color(DIM),
                    color(RESET),
                    target
                )
            } else {
                format!("{}{}{}", name_prefix, item.name, name_suffix)
            }
        } else {
            format!("{}{}{}", name_prefix, item.name, name_suffix)
        };

        let modified = item
            .modified
            .as_ref()
            .map(|dt| {
                let formatted = format_exact_time(dt);
                colorize_date(dt, &formatted)
            })
            .unwrap_or_default();

        // Pad columns for alignment
        let owner_padded = format!("{:<width$}", item.owner, width = max_owner);
        let group_padded = format!("{:<width$}", item.group, width = max_group);
        let size_str = if item.is_dir {
            if item.size > 0 {
                format_size_compact(item.size)
            } else {
                "-".to_string()
            }
        } else {
            format_size_compact(item.size)
        };
        let size_padded = format!("{:>width$}", size_str, width = max_size);
        let contents_raw = if item.is_dir {
            count_items_in_dir(item).to_string()
        } else {
            get_file_contents_raw(item)
        };
        let contents_padded = format!("{:>width$}", contents_raw, width = max_contents);

        // Colorize permissions
        let perm_colored = colorize_perms(&item.perms);

        // Owner/group: blue
        let owner_colored = format!("{}{}{}", color(BLUE), owner_padded, color(RESET));
        let group_colored = format!("{}{}{}", color(BLUE), group_padded, color(RESET));

        // Size: orange
        let size_colored = format!("{}{}{}", color(ORANGE), size_padded, color(RESET));

        // Contents: orange (like size)
        let contents_colored =
            format!("{}{}{}", color(ORANGE), contents_padded, color(RESET));

        // Git status: colored dot (right-aligned in column)
        let git_colored = if git_icon.is_empty() {
            format!("{:width$}", "", width = max_git)
        } else {
            git_icon
        };

        // PERM OWNER GROUP DATE SIZE CONTENTS GIT NAME
        println!(
            "{}{} {} {} {} {} {} {} {}{}{}",
            row_tint,
            perm_colored,
            owner_colored,
            group_colored,
            modified,
            size_colored,
            contents_colored,
            git_colored,
            icon_str,
            name_display,
            tint_reset
        );
    }

    if !show_hidden && !hidden_items.is_empty() {
        println!(
            "  ... and {} hidden items ({} total items)",
            hidden_items.len(),
            summary.total_items
        );
    }
}

/// Truncate a string to a given display width, accounting for ANSI escape codes
#[allow(dead_code)]
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
            for next in chars.by_ref() {
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

    let items: Vec<_> = summary
        .top_items
        .iter()
        .map(|item| {
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
        })
        .collect();

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
    crate::cmd::file_metadata::count_items_in_dir(entry)
}

/// Aggregate git status for a directory — returns the most severe status
/// from any child file (Conflict > Deleted > Modified > Added > Untracked)
fn aggregate_dir_git_status(
    file_statuses: &std::collections::HashMap<String, crate::git::FileStatus>,
    dir_name: &str,
) -> Option<String> {
    let prefix = format!("{}/", dir_name);
    let mut worst: Option<&crate::git::FileStatus> = None;

    for (file_path, status) in file_statuses {
        if file_path.starts_with(&prefix) {
            let is_worse = match worst {
                None => true,
                Some(current) => {
                    let severity = |s: &crate::git::FileStatus| match s {
                        crate::git::FileStatus::Conflict => 5,
                        crate::git::FileStatus::Deleted => 4,
                        crate::git::FileStatus::Modified => 3,
                        crate::git::FileStatus::Added => 2,
                        crate::git::FileStatus::Renamed => 1,
                        crate::git::FileStatus::Untracked => 0,
                    };
                    severity(status) > severity(current)
                }
            };
            if is_worse {
                worst = Some(status);
            }
        }
    }

    worst.map(|fs| format!("{}{}{}", color(fs.color()), fs.icon(), color(RESET)))
}

/// Get contents description for a file — line count for text, resolution for image, etc.
/// Returns plain text (no ANSI codes) — coloring is applied by the renderer.
#[allow(dead_code)]
fn get_file_contents(entry: &crate::fs::DirEntry) -> String {
    crate::cmd::file_metadata::get_file_contents(entry)
}

/// Get raw contents description without ANSI colors (for width calculation)
fn get_file_contents_raw(entry: &crate::fs::DirEntry) -> String {
    crate::cmd::file_metadata::get_file_contents(entry)
}

/// Colorize permission string like exa — each char colored by meaning
/// d=blue, l=magenta, r=green, w=yellow, x=red, -=dim
fn colorize_perms(perms: &str) -> String {
    let mut result = String::with_capacity(perms.len() * 10);
    for c in perms.chars() {
        match c {
            'd' => result.push_str(&format!("{}d{}", color(BLUE_BOLD), color(RESET))),
            'l' => result.push_str(&format!("{}l{}", color(MAGENTA), color(RESET))),
            'r' => result.push_str(&format!("{}r{}", color(GREEN), color(RESET))),
            'w' => result.push_str(&format!("{}w{}", color(YELLOW), color(RESET))),
            'x' | 's' | 'S' | 't' | 'T' => {
                result.push_str(&format!("{}{}{}", color(RED), c, color(RESET)))
            }
            '-' => result.push_str(&format!("{}-{}", color(DIM), color(RESET))),
            _ => result.push(c),
        }
    }
    result
}

/// Format LOC count compactly (e.g., 4.2k, 1.1k, 983)
fn format_loc(loc: usize) -> String {
    if loc < 1000 {
        format!("{}", loc)
    } else if loc < 10000 {
        format!("{:.1}k", loc as f64 / 1000.0)
    } else {
        format!("{}k", loc / 1000)
    }
}
