//! Banner command - the crown jewel
//!
//! Prints a rich, context-aware directory dashboard and exits.
//! This is the main feature that makes f magical.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::path::Path;

use crate::fs::{format_exact_time, format_size_compact, DirSummary};
use crate::git::GitInfo;
use crate::icon;

// ANSI color codes - only emitted when stdout is a tty
static COLORS_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

fn color(code: &str) -> &str {
    if COLORS_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
        code
    } else {
        ""
    }
}

fn set_colors_enabled(enabled: bool) {
    COLORS_ENABLED.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

/// Make filesystem text safe to print in a terminal. Raw mode intentionally
/// preserves paths for scripting, but human-facing names must not allow a
/// filename containing a newline or escape sequence to rewrite the display.
fn escape_terminal_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_control() {
            use std::fmt::Write as _;
            let _ = write!(escaped, "\\u{{{:x}}}", ch as u32);
        } else {
            escaped.push(ch);
        }
    }
    escaped
}

/// Percent-encode a path for a terminal `file://` hyperlink URI.
fn file_url(path: &Path) -> String {
    let mut url = String::from("file://");
    for byte in path.to_string_lossy().as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'.' | b'_' | b'~' | b'/') {
            url.push(*byte as char);
        } else {
            use std::fmt::Write as _;
            let _ = write!(url, "%{byte:02X}");
        }
    }
    url
}

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const BLUE: &str = "\x1b[34m";
const BLUE_BOLD: &str = "\x1b[1;34m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";
const BRIGHT_WHITE: &str = "\x1b[97m";
const ORANGE: &str = "\x1b[38;5;214m";
const ROW_TINT: &str = "\x1b[48;5;234m"; // ultra-subtle dark charcoal for alternating rows

/// Options for the banner command
#[derive(Default)]
#[allow(dead_code)]
pub struct BannerOptions<'a> {
    pub path: Option<&'a Path>,
    pub raw: bool,
    pub json: bool,
    pub compact: bool,
    pub verbose: bool,
    pub sort: Option<String>,
    pub timesort: bool,
    pub sizesort: bool,
    pub extensionsort: bool,
    pub gitsort: bool,
    pub versionsort: bool,
    pub no_sort: bool,
    pub group_dirs: Option<String>,
    pub reverse: bool,
    pub hidden: bool,
    pub relative_date: bool,
    pub filter: Option<&'a str>,
    pub max: Option<usize>,
    pub group: bool,
    pub classify: bool,
    pub blocks: Option<&'a str>,
    pub tree: Option<Option<usize>>,
    pub icons: bool,
    pub colors: bool,
    pub max_items: usize,
    pub oneline: bool,
    pub total_size: bool,
    pub ignore_glob: Vec<String>,
    pub no_symlink: bool,
    pub hyperlink: bool,
    pub recursive: bool,
    pub only_dirs: bool,
    pub only_files: bool,
    pub git_ignore: bool,
    pub level: Option<usize>,
    pub highlight_recent: Option<String>,
    pub highlight_old: Option<String>,
    pub action: Option<String>,
    pub force_edit: bool,
    pub force_run: bool,
}

fn colorize_date(_dt: &DateTime<Utc>, formatted: &str) -> String {
    format!("{}{}{}", color(GREEN), formatted, color(RESET))
}

/// Return whether a file is recent enough to highlight.
/// Binary: files modified within the last 6 hours are "recent", everything
/// else is "old". Future mtimes (clock skew) are treated as "old" rather
/// than "just modified" — the 6-hour window would otherwise fold any
/// ahead-of-clock mtime into the "recent" bucket and never catch up.
fn is_recent(dt: &DateTime<Utc>) -> bool {
    let now = Utc::now();
    let age_secs = (now - *dt).num_seconds();
    if age_secs <= 0 {
        return false;
    }
    age_secs < 21600 // 6 hours
}

/// Apply a background highlight or bold to an entire row.
/// Uses named colors or "bold" for theme-independent highlighting.
fn highlight_row(row: &str, bg_color: &str) -> String {
    if bg_color.is_empty()
        || bg_color == "none"
        || !COLORS_ENABLED.load(std::sync::atomic::Ordering::Relaxed)
    {
        return row.to_string();
    }

    // "bold" is universal — works on any terminal background
    if bg_color == "bold" {
        // Bold for visibility, re-inject after every reset
        let highlighted = row.replace(color(RESET), &format!("{}{}", color(RESET), "\x1b[1m"));
        return format!("\x1b[1m{}{}", highlighted, color(RESET));
    }

    // Convert color name to 256-color code
    let color_code = match bg_color {
        "green" => "22",
        "blue" => "17",
        "red" => "52",
        "yellow" => "58",
        "cyan" => "17",
        "magenta" => "53",
        "gray" | "grey" => "236",
        "dark" => "235",
        "black" => "234",
        "light" => "252",
        _ => bg_color, // assume it's already a color code
    };

    let bg_seq = format!("\x1b[48;5;{}m", color_code);
    // Re-inject background after every reset so the highlight persists
    let highlighted = row.replace(color(RESET), &format!("{}{}", color(RESET), bg_seq));
    format!("{}{}{}", bg_seq, highlighted, color(RESET))
}

/// Apply a subtle background tint to an entire row (e.g. for alternating rows).
/// Re-injects the background sequence after every ANSI reset so the tint covers all columns.
fn apply_row_tint(row: &str, tint: &str) -> String {
    if tint.is_empty() || !COLORS_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
        return row.to_string();
    }
    let highlighted = row.replace(RESET, &format!("{}{}", RESET, tint));
    format!("{}{}{}", tint, highlighted, RESET)
}

/// Collapse repetitive files sharing common prefix/suffix patterns (e.g. RELEASE_NOTES_*.md)
/// keeping the top N newest files and folding older ones to preserve screen real estate.
fn fold_repetitive_patterns(items: &mut Vec<&crate::fs::DirEntry>, max_per_pattern: usize) {
    if items.len() <= 20 {
        return;
    }
    let mut prefix_groups: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();

    for (idx, item) in items.iter().enumerate() {
        if item.is_dir {
            continue;
        }
        let name = &item.name;
        if let Some(pos) = name.rfind(['_', '-']) {
            if pos >= 3 {
                let ext = name.rfind('.').map(|p| &name[p..]).unwrap_or("");
                let prefix = format!("{}{}", &name[..=pos], ext);
                prefix_groups.entry(prefix).or_default().push(idx);
            }
        }
    }

    let mut to_drop: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (_prefix, indices) in prefix_groups {
        if indices.len() >= 4 {
            let mut sorted_indices = indices.clone();
            sorted_indices.sort_by(|&a, &b| {
                let time_a = items[a]
                    .modified
                    .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap_or_default());
                let time_b = items[b]
                    .modified
                    .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap_or_default());
                time_b.cmp(&time_a)
            });
            for &idx in sorted_indices.iter().skip(max_per_pattern) {
                to_drop.insert(idx);
            }
        }
    }

    if !to_drop.is_empty() {
        let mut new_items = Vec::with_capacity(items.len() - to_drop.len());
        for (idx, item) in items.drain(..).enumerate() {
            if !to_drop.contains(&idx) {
                new_items.push(item);
            }
        }
        *items = new_items;
    }
}

/// Build the display items list using the exact same pipeline as output_rich.
/// This ensures navigate_by_number uses the same ordering as the banner display.
/// Returns (display_items, hidden_count).
fn build_display_items<'a>(
    path: &Path,
    summary: &'a crate::fs::DirSummary,
    git_info: &crate::git::GitInfo,
    opts: &BannerOptions,
    config: &crate::state::Config,
    include_implicit_hidden: bool,
) -> (Vec<&'a crate::fs::DirEntry>, usize) {
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
    let show_hidden_flag = opts.hidden || (include_implicit_hidden && total_visible < 30);

    let mut display_items: Vec<&crate::fs::DirEntry> = if show_hidden_flag {
        visible_items
            .iter()
            .chain(hidden_items.iter())
            .copied()
            .collect()
    } else {
        visible_items.to_vec()
    };

    // Apply filter if specified
    if let Some(pattern) = opts.filter {
        let lower_pattern = pattern.to_lowercase();
        display_items.retain(|item| {
            let name_lower = item.name.to_lowercase();
            name_lower.contains(&lower_pattern)
                || item
                    .name
                    .rsplit('.')
                    .next()
                    .map(|ext| ext.to_lowercase().contains(&lower_pattern))
                    .unwrap_or(false)
        });
    }

    // Apply the same ignore-glob filter in rich, navigation, and machine
    // output modes. Previously only oneline/recursive output used this flag.
    if !opts.ignore_glob.is_empty() {
        display_items.retain(|item| {
            !opts
                .ignore_glob
                .iter()
                .any(|pattern| glob_match(pattern, &item.name))
        });
    }

    // Apply only-dirs / only-files filter
    if opts.only_dirs {
        display_items.retain(|item| item.is_dir);
    }
    if opts.only_files {
        display_items.retain(|item| item.is_file);
    }

    // Apply git-ignore filter
    if opts.git_ignore {
        display_items.retain(|item| !is_git_ignored(&item.path));
    }

    // Apply smart pattern folding for repetitive version/log files
    if config.smart_truncation && opts.max.is_none() {
        fold_repetitive_patterns(&mut display_items, 2);
    }

    // Apply max limit if specified (smart truncation for big folders)
    let total_before_truncation = display_items.len();
    if opts.max.is_none()
        && config.smart_truncation
        && config.max_display_items > 0
        && total_before_truncation > config.max_display_items
    {
        display_items.sort_by(|a, b| {
            let a_git = git_info.file_statuses.contains_key(a.name.as_str());
            let b_git = git_info.file_statuses.contains_key(b.name.as_str());
            let git_order = b_git.cmp(&a_git);
            if git_order != std::cmp::Ordering::Equal {
                return git_order;
            }
            let a_time = a
                .modified
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap_or_default());
            let b_time = b
                .modified
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap_or_default());
            b_time.cmp(&a_time)
        });
        let term_height = get_terminal_height();
        let max_fit = if term_height > 10 {
            term_height.saturating_sub(7)
        } else if config.inline_preview {
            25
        } else {
            config.max_display_items
        };
        display_items.truncate(max_fit.max(config.max_display_items));
    }
    let mut hidden_count = total_before_truncation - display_items.len();

    // Group by type if requested
    if opts.group {
        let mut dirs: Vec<&crate::fs::DirEntry> =
            display_items.iter().filter(|i| i.is_dir).copied().collect();
        let mut files: Vec<&crate::fs::DirEntry> = display_items
            .iter()
            .filter(|i| i.is_file && !i.is_symlink)
            .copied()
            .collect();
        let mut symlinks: Vec<&crate::fs::DirEntry> = display_items
            .iter()
            .filter(|i| i.is_symlink)
            .copied()
            .collect();
        // `to_lowercase` allocates; cache the key once per item instead of
        // lowercasing on every comparison.
        dirs.sort_by_cached_key(|i| i.name.to_lowercase());
        files.sort_by_cached_key(|i| i.name.to_lowercase());
        symlinks.sort_by_cached_key(|i| i.name.to_lowercase());
        display_items = files.into_iter().chain(symlinks).chain(dirs).collect();
    }

    // Sort based on --sort flag or short flags
    if !opts.no_sort {
        let sort_mode = if let Some(s) = opts.sort.as_deref() {
            s
        } else if opts.timesort {
            "date"
        } else if opts.sizesort {
            "size"
        } else if opts.extensionsort {
            "extension"
        } else if opts.gitsort {
            "git"
        } else if opts.versionsort {
            "version"
        } else {
            "name"
        };

        let group_dirs_mode = opts
            .group_dirs
            .as_deref()
            .unwrap_or(config.group_dirs.as_str());

        // Pre-compute lowercase names, extensions, and date keys once so the
        // per-comparison sort callback does no allocation. This converts the
        // N log N allocations in the old `a.name.to_lowercase().cmp(...)`
        // calls into a single O(N) pass.
        struct SortKeys {
            lower: String,
            ext: String,
            date: chrono::DateTime<chrono::Utc>,
            git: u8,
        }
        let sort_keys: Vec<SortKeys> = display_items
            .iter()
            .map(|i| SortKeys {
                lower: i.name.to_lowercase(),
                ext: i
                    .name
                    .rfind('.')
                    .map(|pos| i.name[pos..].to_lowercase())
                    .unwrap_or_default(),
                date: i
                    .modified
                    .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap_or_default()),
                git: {
                    let rel = i.path.strip_prefix(path).unwrap_or(&i.path);
                    let rel_str = rel.to_string_lossy();
                    git_info
                        .file_statuses
                        .get(rel_str.as_ref())
                        .or_else(|| git_info.file_statuses.get(i.name.as_str()))
                        .map(|fs| match fs {
                            crate::git::FileStatus::Conflict => 5,
                            crate::git::FileStatus::Deleted => 4,
                            crate::git::FileStatus::Modified => 3,
                            crate::git::FileStatus::Added => 2,
                            crate::git::FileStatus::Renamed => 1,
                            crate::git::FileStatus::Untracked => 0,
                        })
                        .unwrap_or(0)
                },
            })
            .collect();
        // Permute indices so the sort can index into sort_keys cheaply.
        let mut order: Vec<usize> = (0..display_items.len()).collect();
        order.sort_by(|&ia, &ib| {
            let a = &display_items[ia];
            let b = &display_items[ib];
            let ka = &sort_keys[ia];
            let kb = &sort_keys[ib];

            if group_dirs_mode != "none" && a.is_dir != b.is_dir {
                return if group_dirs_mode == "last" {
                    if opts.reverse {
                        b.is_dir.cmp(&a.is_dir)
                    } else {
                        a.is_dir.cmp(&b.is_dir)
                    }
                } else {
                    if opts.reverse {
                        a.is_dir.cmp(&b.is_dir)
                    } else {
                        b.is_dir.cmp(&a.is_dir)
                    }
                };
            }

            let ordering = match sort_mode {
                "size" => b.size.cmp(&a.size),
                "date" => ka.date.cmp(&kb.date),
                "type" => {
                    if ka.ext != kb.ext {
                        ka.ext.cmp(&kb.ext)
                    } else {
                        ka.lower.cmp(&kb.lower)
                    }
                }
                "extension" => {
                    if ka.ext != kb.ext {
                        ka.ext.cmp(&kb.ext)
                    } else {
                        ka.lower.cmp(&kb.lower)
                    }
                }
                "git" => {
                    if kb.git != ka.git {
                        kb.git.cmp(&ka.git)
                    } else {
                        let churn_a = git_info
                            .file_churn
                            .get(a.name.as_str())
                            .copied()
                            .unwrap_or(0);
                        let churn_b = git_info
                            .file_churn
                            .get(b.name.as_str())
                            .copied()
                            .unwrap_or(0);
                        if churn_b != churn_a {
                            churn_b.cmp(&churn_a)
                        } else {
                            kb.date.cmp(&ka.date)
                        }
                    }
                }
                "version" => natural_cmp(&a.name, &b.name),
                _ => ka.lower.cmp(&kb.lower),
            };

            if opts.reverse {
                ordering.reverse()
            } else {
                ordering
            }
        });
        // Project the permutation back into display_items.
        let permuted: Vec<&crate::fs::DirEntry> = order.iter().map(|&i| display_items[i]).collect();
        display_items = permuted;
    }

    // Apply an explicit maximum after sorting so `--sizesort --max 20` really
    // selects the twenty largest entries (and the analogous time/git/type
    // modes select their requested top entries).
    if let Some(max_items) = opts.max {
        hidden_count += display_items.len().saturating_sub(max_items);
        display_items.truncate(max_items);
    }

    (display_items, hidden_count)
}

/// Navigate to item by number - cd if directory, open in editor if file
/// Uses build_display_items() to ensure exact same ordering as banner display
/// Look up the Nth display item and return its path (or run an action on it).
///
/// Uses the exact same `build_display_items` pipeline as the banner display,
/// so the index-to-path mapping is guaranteed to match the numbers shown in
/// the banner.
fn navigate_by_number(
    num: usize,
    path: &std::path::Path,
    opts: &BannerOptions,
    action: Option<&str>,
) -> Result<()> {
    let config = crate::state::Config::load().unwrap_or_default();

    // Try daemon cache first, then direct scan — same as run_banner
    let (summary, git_info) = if let Some(cached) = crate::daemon_client::get_banner_cached(path) {
        (cached.summary, cached.git_info.unwrap_or_default())
    } else {
        let summary =
            crate::fs::DirSummary::scan_with_options(path, false, false, false, false, false, &[])?;
        // Filtered, like run_banner: an unfiltered get_git_info collects every
        // deep untracked path (tens of thousands of entries, e.g. target/).
        let git_filter_paths = crate::git::status_filter_paths_for_items(&summary.top_items);
        let git_info =
            crate::git::get_git_info_filtered(path, &git_filter_paths).unwrap_or_default();
        (summary, git_info)
    };

    // Use the EXACT same pipeline as banner display
    let (display_items, _hidden_count) =
        build_display_items(path, &summary, &git_info, opts, &config, true);

    if num == 0 || num > display_items.len() {
        anyhow::bail!(
            "Error: number {} out of range (1-{}). Use 'f' to see available items.",
            num,
            display_items.len()
        );
    }

    let entry = &display_items[num - 1];
    let target = &entry.path;

    if let Some(app) = action {
        // Explicit action: run the app with the target path
        let status = std::process::Command::new(app).arg(target).status()?;
        if !status.success() {
            eprintln!("'{}' exited with status: {}", app, status);
        }
    } else if opts.force_edit {
        // --edit flag: force open in editor
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| config.open_command.clone());
        let status = std::process::Command::new(&editor).arg(target).status()?;
        if !status.success() {
            eprintln!("Editor '{}' exited with status: {}", editor, status);
        }
    } else if opts.force_run {
        // --run flag: force run the file directly
        let status = std::process::Command::new(target).status()?;
        if !status.success() {
            eprintln!("'{}' exited with status: {}", target.display(), status);
        }
    } else {
        // Print path for shell wrapper to handle
        // Shell wrapper checks if target is dir (cd) or file (open editor)
        println!("{}", target.display());
    }

    Ok(())
}

pub fn run_banner(mut opts: BannerOptions) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Check if the path argument is a number for navigation BEFORE resolving it.
    // If it is, the directory to scan is the current directory (or the explicitly
    // passed non-numeric path), not the number string itself.
    let is_numeric_nav = opts
        .path
        .as_ref()
        .and_then(|p| p.to_str())
        .and_then(|s| s.parse::<usize>().ok())
        .is_some();

    // For navigation, the directory path is cwd (or the original path if it was
    // a number — the number is the item index, not the path).
    // For non-numeric paths, resolve the canonical path for scanning.
    let path = if is_numeric_nav {
        // Use cwd for navigation — the number is the item index, not a path
        cwd.canonicalize()
            .unwrap_or_else(|_| cwd.as_path().to_path_buf())
    } else {
        let requested_path = opts.path.unwrap_or(cwd.as_path());
        requested_path
            .canonicalize()
            .with_context(|| format!("No such file or directory: {}", requested_path.display()))?
    };

    // Load config and apply env var overrides
    let config = crate::state::Config::load().unwrap_or_default();
    let icons = std::env::var("FAB_ICONS")
        .map(|v| v == "1" || v == "nerd")
        .unwrap_or(config.icons);
    let no_color = std::env::var("NO_COLOR").is_ok();
    let colors = if no_color {
        false
    } else {
        std::env::var("FAB_COLORS")
            .map(|v| v == "1")
            .unwrap_or(config.colors)
    };
    let max_items = config.max_display_items;
    opts.icons = icons;
    opts.colors = colors;
    opts.max_items = max_items;
    set_colors_enabled(opts.colors);

    // Apply config defaults (CLI flags override these)
    if !opts.compact && config.compact {
        opts.compact = true;
    }
    if !opts.verbose && config.verbose {
        opts.verbose = true;
    }
    if opts.sort.is_none() && config.sort != "name" {
        opts.sort = Some(config.sort.clone());
    }
    if !opts.reverse && config.reverse {
        opts.reverse = true;
    }
    if opts.group_dirs.is_none() && config.group_dirs != "none" {
        opts.group_dirs = Some(config.group_dirs.clone());
    }
    if !opts.hidden && config.hidden {
        opts.hidden = true;
    }
    if !opts.classify && config.classify {
        opts.classify = true;
    }
    if !opts.relative_date && config.date == "relative" {
        opts.relative_date = true;
    }
    if !opts.total_size && config.total_size {
        opts.total_size = true;
    }
    if !opts.no_symlink && config.no_symlink {
        opts.no_symlink = true;
    }
    if !opts.hyperlink && config.hyperlink {
        opts.hyperlink = true;
    }
    if opts.highlight_recent.is_none() && !config.highlight_recent.is_empty() {
        opts.highlight_recent = Some(config.highlight_recent.clone());
    }
    if opts.highlight_old.is_none() && !config.highlight_old.is_empty() {
        opts.highlight_old = Some(config.highlight_old.clone());
    }

    // Check if path argument is a number for navigation (after config is loaded)
    if let Some(path_arg) = &opts.path {
        if let Some(num_str) = path_arg.to_str() {
            if let Ok(num) = num_str.parse::<usize>() {
                // Numeric navigation - look up item by number using same display pipeline.
                // Pass the resolved `path` (not `cwd`) so the item ordering matches
                // the banner display exactly.
                return navigate_by_number(num, &path, &opts, opts.action.as_deref());
            }
        }
    }

    // Tree view mode
    if let Some(depth) = opts.tree {
        let max_depth = opts.level.unwrap_or(depth.unwrap_or(0)); // -L overrides tree depth
        output_tree(&path, max_depth, opts.hidden, opts.filter, icons, colors);
        return Ok(());
    }

    // Recursive mode
    if opts.recursive {
        output_recursive(&path, &opts)?;
        return Ok(());
    }

    // Try daemon cache - if daemon isn't running, start it and retry
    if let Some(cached) = crate::daemon_client::get_banner_cached(&path) {
        let summary = cached.summary;
        let git_info = cached.git_info.unwrap_or_default();

        if opts.oneline {
            output_oneline(&path, &summary, &git_info, &opts, &config);
        } else if opts.json {
            output_json(&path, &summary, &git_info, &opts, &config);
        } else if opts.raw {
            output_raw(&path, &summary, &git_info, &opts, &config);
        } else {
            output_rich(&path, &summary, &git_info, &opts);
        }

        // Warm daemon cache for likely next directories (parent + siblings)
        warm_nearby_dirs(&path);
        return Ok(());
    }

    // Daemon not available or cache miss - try direct scan.
    // Todos/ports/docker/metrics are disabled by default for speed; set FAB_*=1 to enable.
    tracing::debug!("f daemon not available, falling back to direct scan");
    let scan_todos =
        crate::utils::direct_feature_enabled(config.todo_count, "FAB_TODOS", "FAB_NO_TODOS");
    let scan_ports =
        crate::utils::direct_feature_enabled(config.ports, "FAB_PORTS", "FAB_NO_PORTS");
    let scan_docker =
        crate::utils::direct_feature_enabled(config.docker, "FAB_DOCKER", "FAB_NO_DOCKER");
    let scan_metrics =
        crate::utils::direct_feature_enabled(config.languages, "FAB_METRICS", "FAB_NO_METRICS");

    // Build extra skip dirs from config
    let extra_skip_dirs: Vec<&str> = config.ignore_dirs.iter().map(|s| s.as_str()).collect();

    let summary = DirSummary::scan_with_options(
        &path,
        // Build checks spawn subprocesses (cargo check ≈ 6.7s) — opt-in via
        // `f config` (build_status), off by default.
        config.build_status,
        scan_todos,
        scan_ports,
        scan_docker,
        scan_metrics,
        &extra_skip_dirs,
    )?;
    let needs_git_for_sort = opts.gitsort || opts.sort.as_deref() == Some("git");
    let git_info = if !config.git_status || ((opts.oneline || opts.raw) && !needs_git_for_sort) {
        crate::git::GitInfo::default()
    } else {
        let git_filter_paths = crate::git::status_filter_paths_for_items(&summary.top_items);
        crate::git::get_git_info_filtered(&path, &git_filter_paths)?
    };

    // Display the banner
    if opts.oneline {
        output_oneline(&path, &summary, &git_info, &opts, &config);
    } else if opts.json {
        output_json(&path, &summary, &git_info, &opts, &config);
    } else if opts.raw {
        output_raw(&path, &summary, &git_info, &opts, &config);
    } else {
        output_rich(&path, &summary, &git_info, &opts);
    }

    // Warm daemon cache for likely next directories (parent + siblings)
    warm_nearby_dirs(&path);

    Ok(())
}

/// Output tree view of directory
fn output_tree(
    path: &Path,
    max_depth: usize,
    show_hidden: bool,
    filter: Option<&str>,
    icons: bool,
    _colors: bool,
) {
    println!("{}", escape_terminal_text(&path.to_string_lossy()));
    print_tree_recursive(path, "", max_depth, 0, show_hidden, filter, icons);
}

fn print_tree_recursive(
    path: &Path,
    prefix: &str,
    max_depth: usize,
    current_depth: usize,
    show_hidden: bool,
    filter: Option<&str>,
    icons: bool,
) {
    if max_depth > 0 && current_depth >= max_depth {
        return;
    }

    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut items: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if !show_hidden && name.starts_with('.') {
                return false;
            }
            if let Some(pat) = filter {
                let lower = name.to_lowercase();
                let lower_pat = pat.to_lowercase();
                if !lower.contains(&lower_pat) {
                    return false;
                }
            }
            true
        })
        .collect();

    // Sort: dirs first, then by name
    items.sort_by(|a, b| {
        let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a
                .file_name()
                .to_string_lossy()
                .to_lowercase()
                .cmp(&b.file_name().to_string_lossy().to_lowercase()),
        }
    });

    let len = items.len();
    for (idx, entry) in items.iter().enumerate() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let is_last = idx == len - 1;

        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        let icon_str = if icons {
            if is_dir {
                "📁 "
            } else {
                "📄 "
            }
        } else {
            ""
        };

        // Get file metadata for display
        let meta = entry.metadata().ok();
        let size_str = meta
            .as_ref()
            .map(|m| format_size_compact(m.len()))
            .unwrap_or_default();
        let date_str = meta
            .as_ref()
            .map(|m| {
                m.modified()
                    .ok()
                    .map(|t| {
                        let dt: chrono::DateTime<chrono::Utc> = t.into();
                        dt.format("%Y-%m-%d %H:%M").to_string()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        if is_dir {
            println!(
                "{}{}{}{}/",
                prefix,
                connector,
                icon_str,
                escape_terminal_text(&name)
            );
            let new_prefix = format!("{}{}", prefix, child_prefix);
            print_tree_recursive(
                &entry.path(),
                &new_prefix,
                max_depth,
                current_depth + 1,
                show_hidden,
                filter,
                icons,
            );
        } else {
            // Show file with metadata
            let meta_str = if !size_str.is_empty() || !date_str.is_empty() {
                format!(" {} {}", size_str, date_str)
            } else {
                String::new()
            };
            println!(
                "{}{}{}{}{}",
                prefix,
                connector,
                icon_str,
                escape_terminal_text(&name),
                meta_str
            );
        }
    }
}

/// Pre-compute banners for the parent directory and the immediate children of
/// the current directory, so moving up to the parent or into a sibling
/// directory is served from the daemon cache.
fn warm_nearby_dirs(path: &Path) {
    // Warm the parent so `cd ..` is fast.
    let mut paths_to_warm = Vec::new();
    if let Some(parent) = path.parent() {
        if parent.is_dir() {
            paths_to_warm.push(parent.to_path_buf());
        }
    }

    // Warm immediate children of the current directory so `cd <child>` or fuzzy
    // jump into a child is served from the daemon cache. Bounded to a modest
    // number of children to avoid excessive background scans in very large dirs.
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.take(30).flatten() {
            let child = entry.path();
            if child.is_dir() {
                paths_to_warm.push(child);
            }
        }
    }

    // Warm the grandparent so `cd ../..` is also fast.
    if let Some(parent) = path.parent() {
        if let Some(grandparent) = parent.parent() {
            if grandparent.is_dir() {
                paths_to_warm.push(grandparent.to_path_buf());
            }
        }
    }

    if paths_to_warm.is_empty() {
        return;
    }

    // Deduplicate (paths_to_warm may contain parent and grandparent).
    paths_to_warm.sort();
    paths_to_warm.dedup();

    // Send bounded warm requests before exiting. The daemon handles Warm
    // requests asynchronously, so this should not block on full scans.
    crate::daemon_client::warm_paths(&paths_to_warm);
}

/// Output rich formatted banner - compact lsd-style layout
#[allow(clippy::too_many_arguments)]
/// Build git branch display with color (blue if clean, yellow if dirty).
/// The closing bracket is intentionally kept inside the same color/bold span
/// as the branch name so it never renders in a darker shade.
fn build_branch_display(git_info: &GitInfo) -> String {
    let git_branch = git_info.branch.as_deref().unwrap_or("");
    if git_branch.is_empty() {
        return String::new();
    }
    if git_info.is_dirty {
        format!(
            "{yellow}[{branch}*]{reset}",
            yellow = color(YELLOW),
            branch = git_branch,
            reset = color(RESET)
        )
    } else {
        format!(
            "{blue}[{branch}]{reset}",
            blue = color(BLUE_BOLD),
            branch = git_branch,
            reset = color(RESET)
        )
    }
}

/// Build git status indicators (p10k-style): *modified +staged ?untracked ↑ahead ↓behind
fn build_git_status_indicators(git_info: &GitInfo) -> String {
    let mut indicators = Vec::new();
    if git_info.modified > 0 {
        indicators.push(format!(
            "{}*{}{}",
            color(YELLOW),
            git_info.modified,
            color(RESET)
        ));
    }
    if git_info.staged > 0 {
        indicators.push(format!(
            "{}+{}{}",
            color(GREEN),
            git_info.staged,
            color(RESET)
        ));
    }
    if git_info.untracked > 0 {
        indicators.push(format!(
            "{}?{}{}",
            color(DIM),
            git_info.untracked,
            color(RESET)
        ));
    }
    if git_info.ahead > 0 {
        indicators.push(format!(
            "{}↑{}{}",
            color(CYAN),
            git_info.ahead,
            color(RESET)
        ));
    }
    if git_info.behind > 0 {
        indicators.push(format!(
            "{}↓{}{}",
            color(RED),
            git_info.behind,
            color(RESET)
        ));
    }
    indicators.join(" ")
}

/// Build the second header row (stats/details) for both git-repo and plain outputs.
///
/// The git and plain branches historically duplicated ~120 lines differing only
/// in 3 details: clean indicator (handled in row1's git_status), port list
/// rendering note, and docker fallback (plain shows generic docker when
/// has_compose/has_dockerfile). The helper also captures the remaining
/// variant differences: build error string (git includes count), ordering
/// (git: todos→metrics→build, plain: build→todos→metrics), and test-results
/// (git only). Callers in both `if git_info.is_repo` branches share this
/// helper so the diff stays small — variant is chosen via `git_info.is_repo`.
fn build_details_row(
    _path: &Path,
    summary: &DirSummary,
    git_info: &GitInfo,
    opts: &BannerOptions,
    config: &crate::state::Config,
) -> Vec<String> {
    let todo_info = if config.todo_count {
        summary.todo_info.as_ref()
    } else {
        None
    };
    let code_metrics = if config.languages {
        summary.code_metrics.as_ref()
    } else {
        None
    };
    let build_status = if config.build_status {
        summary.build_status.as_ref()
    } else {
        None
    };
    let port_info = if config.ports {
        summary.port_info.as_ref()
    } else {
        None
    };
    let docker_info = if config.docker {
        summary.docker_info.as_ref()
    } else {
        None
    };

    let project_icon = summary.project_type.icon();
    let size_str = format_size_for_mode(summary.total_size, &config.size);
    let size_label = if summary.truncated { "sample" } else { "total" };

    let mut details = Vec::new();

    // File stats (common)
    if opts.total_size || config.total_size {
        details.push(format!(
            "{}{}{} {}{}{}",
            color(CYAN),
            size_str,
            color(RESET),
            color(DIM),
            size_label,
            color(RESET)
        ));
    }
    if summary.truncated {
        details.push(format!(
            "{}{}+{} {}items{}",
            color(BRIGHT_WHITE),
            crate::fs::MAX_DIRECTORY_SCAN_ITEMS,
            color(RESET),
            color(DIM),
            color(RESET)
        ));
    } else {
        details.push(format!(
            "{}{}{} {}files{}",
            color(BRIGHT_WHITE),
            summary.files,
            color(RESET),
            color(DIM),
            color(RESET)
        ));
        details.push(format!(
            "{}{}{} {}{}{}",
            color(BRIGHT_WHITE),
            summary.dirs,
            color(RESET),
            color(DIM),
            if summary.dirs == 1 { "dir" } else { "dirs" },
            color(RESET)
        ));
    }

    // Shared language-breakdown push — used in both variants
    let push_code_metrics = |details: &mut Vec<String>| {
        if let Some(metrics) = code_metrics {
            if metrics.total_loc > 0 {
                let loc_str = format_loc(metrics.total_loc);
                details.push(format!(
                    "{}{}{} {}lines{}",
                    color(GREEN),
                    loc_str,
                    color(RESET),
                    color(DIM),
                    color(RESET)
                ));
                if !metrics.by_extension.is_empty() && metrics.total_loc > 0 {
                    let lang_parts: Vec<String> = metrics
                        .by_extension
                        .iter()
                        .filter(|(ext, _)| {
                            !ext.chars().all(|c| c.is_numeric())
                                && ext != "no-ext"
                                && !ext.is_empty()
                        })
                        .take(3)
                        .map(|(ext, loc)| {
                            let pct = (*loc as f64 / metrics.total_loc as f64 * 100.0) as usize;
                            let name = match ext.as_str() {
                                "rs" => "Rust",
                                "md" | "mdx" => "Markdown",
                                "sh" | "bash" => "Shell",
                                "py" => "Python",
                                "js" | "mjs" => "JavaScript",
                                "ts" | "tsx" => "TypeScript",
                                "svelte" => "Svelte",
                                "go" => "Go",
                                "c" | "h" => "C",
                                "cpp" | "cc" | "cxx" | "hpp" => "C++",
                                "java" => "Java",
                                "rb" => "Ruby",
                                "toml" => "TOML",
                                "yaml" | "yml" => "YAML",
                                "json" => "JSON",
                                "html" | "htm" => "HTML",
                                "css" => "CSS",
                                "sql" => "SQL",
                                "vim" => "VimL",
                                "el" => "Emacs Lisp",
                                _ => ext,
                            };
                            format!("{}% {}{}{}", pct, color(DIM), name, color(RESET))
                        })
                        .collect();
                    if !lang_parts.is_empty() {
                        details.push(format!(
                            "{} {}{}{}",
                            project_icon,
                            color(DIM),
                            lang_parts.join(", "),
                            color(RESET)
                        ));
                    }
                }
            }
        }
    };

    if git_info.is_repo {
        // Git-repo variant — 3 differing details handled here:
        // clean indicator lives in row1's git_status, port list is identical,
        // docker fallback is omitted (only containers>0). Also: build error
        // includes count, and test results are shown.
        if let Some(todos) = todo_info {
            if todos.count > 0 {
                details.push(format!(
                    "{}{}{} {}TODOs{}",
                    color(YELLOW),
                    todos.count,
                    color(RESET),
                    color(DIM),
                    color(RESET)
                ));
            }
        }
        push_code_metrics(&mut details);
        if let Some(build) = build_status {
            if build.ok {
                let duration_str = if build.duration_ms > 0 {
                    if build.duration_ms < 1000 {
                        format!(" ({}ms)", build.duration_ms)
                    } else {
                        format!(" ({:.1}s)", build.duration_ms as f64 / 1000.0)
                    }
                } else {
                    String::new()
                };
                details.push(format!(
                    "{}✓ builds{}{}",
                    color(GREEN),
                    duration_str,
                    color(RESET)
                ));
            } else {
                let err_str = if build.errors > 0 {
                    format!(" ({} err)", build.errors)
                } else {
                    String::new()
                };
                details.push(format!(
                    "{}✗ build errors{}{}",
                    color(RED),
                    err_str,
                    color(RESET)
                ));
            }
        }
        if let Some(ports) = port_info {
            if !ports.ports.is_empty() {
                let port_str: Vec<String> = ports.ports.iter().map(|p| format!(":{}", p)).collect();
                details.push(format!(
                    "{}🔌 {}{}",
                    color(CYAN),
                    port_str.join(", "),
                    color(RESET)
                ));
            }
        }
        if let Some(docker) = docker_info {
            let running = docker
                .containers
                .iter()
                .filter(|c| c.status.contains("Up"))
                .count();
            let total = docker.containers.len();
            if total > 0 {
                details.push(format!(
                    "{}🐳 {} container(s){}",
                    color(CYAN),
                    if running > 0 {
                        format!("{} up", running)
                    } else {
                        total.to_string()
                    },
                    color(RESET)
                ));
            }
        }
        if let Some(test_results) = crate::test_cache::TestResults::load() {
            if test_results.failed > 0 {
                details.push(format!(
                    "{}✗ {} failed{}",
                    color(RED),
                    test_results.failed,
                    color(RESET)
                ));
            } else if test_results.passed > 0 {
                details.push(format!(
                    "{}✓ {} tests{} ({})",
                    color(GREEN),
                    test_results.passed,
                    color(RESET),
                    test_results.format_time_ago()
                ));
            }
        }
    } else {
        // Plain variant — todos after build, generic docker fallback, build error without count
        if let Some(build) = build_status {
            if build.ok {
                let duration_str = if build.duration_ms > 0 {
                    if build.duration_ms < 1000 {
                        format!(" ({}ms)", build.duration_ms)
                    } else {
                        format!(" ({:.1}s)", build.duration_ms as f64 / 1000.0)
                    }
                } else {
                    String::new()
                };
                details.push(format!(
                    "{}✓ builds{}{}",
                    color(GREEN),
                    duration_str,
                    color(RESET)
                ));
            } else {
                details.push(format!("{}✗ build errors{}", color(RED), color(RESET)));
            }
        }
        if let Some(todos) = todo_info {
            if todos.count > 0 {
                details.push(format!(
                    "{}{}{} {}TODOs{}",
                    color(YELLOW),
                    todos.count,
                    color(RESET),
                    color(DIM),
                    color(RESET)
                ));
            }
        }
        push_code_metrics(&mut details);
        if let Some(ports) = port_info {
            if !ports.ports.is_empty() {
                let port_str: Vec<String> = ports.ports.iter().map(|p| format!(":{}", p)).collect();
                details.push(format!(
                    "{}🔌 {}{}",
                    color(CYAN),
                    port_str.join(", "),
                    color(RESET)
                ));
            }
        }
        if let Some(docker) = docker_info {
            let running = docker
                .containers
                .iter()
                .filter(|c| c.status.contains("Up"))
                .count();
            let total = docker.containers.len();
            if total > 0 {
                details.push(format!(
                    "{}🐳 {} container(s){}",
                    color(CYAN),
                    if running > 0 {
                        format!("{} up", running)
                    } else {
                        total.to_string()
                    },
                    color(RESET)
                ));
            } else if docker.has_compose || docker.has_dockerfile {
                details.push(format!("{}🐳 docker{}", color(DIM), color(RESET)));
            }
        }
    }

    details
}

fn output_rich(path: &Path, summary: &DirSummary, git_info: &GitInfo, opts: &BannerOptions) {
    // Load config for display settings
    let config = crate::state::Config::load().unwrap_or_default();
    let path_str = path.to_string_lossy();
    let project_label = summary.project_type.label();

    let home = std::env::var("HOME").unwrap_or_default();
    let (path_prefix, path_leaf) = if path_str.starts_with(&home) {
        let relative = &path_str[home.len()..];
        if relative.is_empty() || relative == "/" {
            ("".to_string(), "~".to_string())
        } else if let Some(pos) = relative.rfind('/') {
            (
                format!("~{}/", &relative[..pos]),
                relative[pos + 1..].to_string(),
            )
        } else {
            ("~".to_string(), relative.to_string())
        }
    } else if let Some(pos) = path_str.rfind('/') {
        (
            format!("{}/", &path_str[..pos]),
            path_str[pos + 1..].to_string(),
        )
    } else {
        ("".to_string(), path_str.to_string())
    };

    let path_display = if path_prefix.is_empty() {
        format!(
            "{}{}{}",
            color(BOLD),
            escape_terminal_text(&path_leaf),
            color(RESET)
        )
    } else {
        format!(
            "{}{}{}{}{}",
            color(DIM),
            escape_terminal_text(&path_prefix),
            color(RESET),
            color(BOLD),
            escape_terminal_text(&path_leaf)
        )
    };

    let _git_branch = git_info.branch.as_deref().unwrap_or("");
    let hidden_count = summary
        .top_items
        .iter()
        .filter(|item| item.name.starts_with('.'))
        .count();

    // Deferred header: compute only when about to display
    let sep = format!(" {}·{} ", color(DIM), color(RESET));
    let header = {
        let branch_display = build_branch_display(git_info);
        let git_status_str = build_git_status_indicators(git_info);
        let mut git_status = if git_status_str.is_empty() {
            Vec::new()
        } else {
            git_status_str.split(' ').map(String::from).collect()
        };
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
        // Clean indicator
        if !git_info.is_dirty
            && git_info.modified == 0
            && git_info.staged == 0
            && git_info.untracked == 0
        {
            git_status.push(format!("{}✓ clean{}", color(GREEN), color(RESET)));
        }
        let git_status_str = git_status.join(" ");

        if git_info.is_repo {
            // Row 1: Path + Git details (explicit)
            let mut parts = vec![format!("{}", path_display)];
            if !branch_display.is_empty() {
                parts.push(format!("{}{}{}", color(BOLD), branch_display, color(RESET)));
            }
            if let Some(ref tag) = git_info.tag {
                parts.push(format!("{}{}{}", color(YELLOW), tag, color(RESET)));
            }
            // Git status indicators
            if !git_status_str.is_empty() {
                parts.push(git_status_str.clone());
            }
            // Last commit time
            if let Some(time) = git_info.last_commit_time {
                let now = chrono::Utc::now().timestamp();
                let diff = now - time;
                let time_str = if diff < 60 {
                    "just now".to_string()
                } else if diff < 3600 {
                    format!("{}m ago", diff / 60)
                } else if diff < 86400 {
                    format!("{}h ago", diff / 3600)
                } else {
                    format!("{}d ago", diff / 86400)
                };
                parts.push(time_str);
            }
            // Commits today
            if git_info.commits_today > 0 {
                parts.push(format!("{} today", git_info.commits_today));
            }
            // Diff stats (colored)
            if git_info.lines_added > 0 || git_info.lines_deleted > 0 {
                parts.push(format!(
                    "{}+{}{} {}-{}{}",
                    color(GREEN),
                    git_info.lines_added,
                    color(RESET),
                    color(RED),
                    git_info.lines_deleted,
                    color(RESET)
                ));
            }
            let row1 = parts.join(&sep);

            let details = build_details_row(path, summary, git_info, opts, &config);

            // Combine rows with dynamic truncation
            if details.is_empty() {
                row1
            } else {
                let details_str = details.join(&sep);
                let term_width = get_terminal_width();
                let row1_width = display_width(&row1);
                let details_width = display_width(&details_str);

                if term_width > 0 && row1_width + details_width > term_width {
                    let available = term_width.saturating_sub(row1_width + 3);
                    if available > 20 {
                        let truncated = truncate_details(&details, available);
                        format!("{}\n{}", row1, truncated)
                    } else {
                        row1
                    }
                } else {
                    format!("{}\n{}", row1, details_str)
                }
            }
        } else {
            // Row 1: Core info
            let mut parts = vec![path_display.clone()];
            if !project_label.is_empty() {
                parts.push(format!("{}{}{}", color(DIM), project_label, color(RESET)));
            }
            if hidden_count > 0 {
                parts.push(format!(
                    "{}{} hidden{}",
                    color(DIM),
                    hidden_count,
                    color(RESET)
                ));
            }
            let row1 = parts.join(&sep);

            let details = build_details_row(path, summary, git_info, opts, &config);

            // Combine rows with dynamic truncation
            if details.is_empty() {
                row1
            } else {
                let details_str = details.join(&sep);
                let term_width = get_terminal_width();
                let row1_width = display_width(&row1);
                let details_width = display_width(&details_str);

                if term_width > 0 && row1_width + details_width > term_width {
                    let available = term_width.saturating_sub(row1_width + 3);
                    if available > 20 {
                        let truncated = truncate_details(&details, available);
                        format!("{}\n{}", row1, truncated)
                    } else {
                        row1
                    }
                } else {
                    format!("{}\n{}", row1, details_str)
                }
            }
        }
    };

    println!("{}", header);
    // Underline only the second row
    if let Some(last_line) = header.lines().last() {
        let last_width = display_width(last_line);
        println!("{}{}{}", color(DIM), "─".repeat(last_width), color(RESET));
    }

    // Use shared build_display_items for consistent ordering with navigation
    let (display_items, hidden_count) =
        build_display_items(path, summary, git_info, opts, &config, true);

    // Determine the effective column set so we can skip the expensive
    // per-file contents probe (PNG/JPG/ZIP header reads, text line counts)
    // when the user has hidden the contents column. This is the single biggest
    // performance win in directories with many images, archives, or large text
    // files.
    let mut effective_columns: Vec<String> = if let Some(blocks_str) = opts.blocks {
        blocks_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
    } else if opts.compact {
        vec!["size".to_string(), "date".to_string(), "name".to_string()]
    } else if opts.verbose {
        vec![
            "permission".to_string(),
            "owner".to_string(),
            "group".to_string(),
            "size".to_string(),
            "contents".to_string(),
            "date".to_string(),
            "name".to_string(),
        ]
    } else {
        config.columns.clone()
    };
    if opts.blocks.is_none() {
        effective_columns
            .retain(|column| !config.hide_columns.iter().any(|hidden| hidden == column));
    }
    let show_contents_column = effective_columns.iter().any(|c| c == "contents");
    let _profile = std::env::var("FAB_PROFILE").is_ok();
    let _t0 = std::time::Instant::now();

    // Precompute expensive contents metadata once so directory counts and file
    // content probes are not repeated during width calculation and row rendering.
    // Skip the per-file probe entirely when the contents column is hidden.
    let mut _dir_probe_total = std::time::Duration::ZERO;
    let mut _file_probe_total = std::time::Duration::ZERO;
    let display_meta: Vec<_> = display_items
        .iter()
        .map(|item| {
            if !show_contents_column {
                return (*item, String::new());
            }
            let t_p = std::time::Instant::now();
            let contents_raw = if item.is_dir {
                crate::cmd::file_metadata::count_items_in_dir_display(item)
            } else {
                get_file_contents_raw(item)
            };
            if item.is_dir {
                _dir_probe_total += t_p.elapsed();
            } else {
                _file_probe_total += t_p.elapsed();
            }
            (*item, contents_raw)
        })
        .collect();
    if _profile {
        eprintln!(
            "[FAB_PROFILE] display_meta ({} items, contents={}): total={:?} dir={:?} file={:?}",
            display_items.len(),
            show_contents_column,
            _t0.elapsed(),
            _dir_probe_total,
            _file_probe_total
        );
    }
    let _t1 = std::time::Instant::now();

    // Compute max column widths for alignment
    let mut max_owner = 5; // "OWNER"
    let mut max_group = 5; // "GROUP"
    let mut max_size = 4; // "SIZE"
    let mut max_contents = 4; // dynamic
    let mut max_git = 1; // git status icon (always 1 char)

    for (item, contents_raw) in &display_meta {
        max_owner = max_owner.max(display_width(&item.owner));
        max_group = max_group.max(display_width(&item.group));
        let size_str = format_size_for_mode(item.size, &config.size);
        max_size = max_size.max(display_width(&size_str));
        max_contents = max_contents.max(display_width(contents_raw).max(4));
        // Git status is always 1 char, but we need a column for it
        max_git = 1;
    }

    // Print each row - PERM OWNER GROUP CONTENTS SIZE DATE NAME
    let num_width = display_items.len().to_string().len(); // for right-aligned numbering
    for (idx, (item, contents_raw)) in display_meta.iter().enumerate() {
        let icon_str = if opts.icons {
            icon::icon_for(&item.name, item.is_dir, item.is_exec, item.is_symlink)
        } else {
            String::new()
        };

        // Per-file git status - try relative path first, then filename
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
                    // Tracked but clean - show dim dot
                    if git_info.is_repo {
                        format!("{}●{}", color(DIM), color(RESET))
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
        // Add classify indicator if enabled
        let classify_suffix = if opts.classify || config.classify {
            if item.is_dir {
                "/".to_string()
            } else if item.is_symlink {
                "@".to_string()
            } else if item.is_exec {
                "*".to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let item_name = escape_terminal_text(&item.name);
        let item_name_str = if opts.hyperlink {
            let canonical = item
                .path
                .canonicalize()
                .unwrap_or_else(|_| item.path.clone());
            format!(
                "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
                file_url(&canonical),
                item_name
            )
        } else {
            item_name
        };

        let name_display = if item.is_symlink && !opts.no_symlink {
            if let Some(target) = &item.symlink_target {
                let indicator = if item.symlink_valid { "→" } else { "✗→" };
                format!(
                    "{}{}{}{} {}{}{} {}",
                    name_prefix,
                    item_name_str,
                    classify_suffix,
                    name_suffix,
                    color(DIM),
                    indicator,
                    color(RESET),
                    escape_terminal_text(target)
                )
            } else {
                format!(
                    "{}{}{}{}",
                    name_prefix, item_name_str, classify_suffix, name_suffix
                )
            }
        } else {
            format!(
                "{}{}{}{}",
                name_prefix, item_name_str, classify_suffix, name_suffix
            )
        };

        let modified = item
            .modified
            .as_ref()
            .map(|dt| {
                let formatted = if opts.relative_date {
                    crate::fs::format_relative_time(dt)
                } else {
                    format_exact_time(dt)
                };
                // Row-level recency intensity is applied at print time
                colorize_date(dt, &formatted)
            })
            .unwrap_or_default();

        // Pad columns for alignment
        let owner_padded = format!("{:<width$}", item.owner, width = max_owner);
        let group_padded = format!("{:<width$}", item.group, width = max_group);
        let size_str = if item.is_dir {
            if item.size > 0 {
                format_size_for_mode(item.size, &config.size)
            } else {
                "-".to_string()
            }
        } else {
            format_size_for_mode(item.size, &config.size)
        };
        let size_padded = format!("{:>width$}", size_str, width = max_size);
        let contents_padded = format!("{:>width$}", contents_raw, width = max_contents);

        // Colorize permissions based on config
        let perm_colored = match config.permission.as_str() {
            "octal" => {
                if let Some(octal) = perms_to_octal(&item.perms) {
                    format!("{}{}{}", color(DIM), octal, color(RESET))
                } else {
                    colorize_perms(&item.perms)
                }
            }
            "disable" => String::new(),
            _ => colorize_perms(&item.perms), // default: rwx
        };

        // Owner/group: blue
        let owner_colored = format!("{}{}{}", color(BLUE), owner_padded, color(RESET));
        let group_colored = format!("{}{}{}", color(BLUE), group_padded, color(RESET));

        // Size: orange, row-level intensity applied at print time
        let size_colored = format!("{}{}{}", color(ORANGE), size_padded, color(RESET));

        // Contents: orange (like size)
        let contents_colored = format!("{}{}{}", color(ORANGE), contents_padded, color(RESET));

        // Git status: colored dot (right-aligned in column)
        let git_colored = if git_icon.is_empty() {
            format!("{:width$}", "", width = max_git)
        } else {
            git_icon
        };

        // Build row based on effective columns (or --blocks override)
        let columns = effective_columns.clone();

        let mut row_parts = Vec::new();
        if columns.contains(&"permission".to_string()) && !perm_colored.is_empty() {
            row_parts.push(perm_colored);
        }
        if columns.contains(&"owner".to_string()) {
            row_parts.push(owner_colored);
        }
        if columns.contains(&"group".to_string()) {
            row_parts.push(group_colored);
        }
        if columns.contains(&"date".to_string()) {
            row_parts.push(modified);
        }
        if columns.contains(&"size".to_string()) {
            row_parts.push(size_colored);
        }
        if columns.contains(&"contents".to_string()) {
            row_parts.push(contents_colored);
        }
        if config.git_status {
            row_parts.push(git_colored);
        }
        // Add navigation number if enabled
        if config.numbered {
            let num = idx + 1; // 1-based numbering
            let num_padded = format!("{:>width$}", num, width = num_width);
            let num_str = format!(
                "{bold}{white}[{num}{reset}{white}]",
                bold = color(BOLD),
                white = color(BRIGHT_WHITE),
                num = num_padded,
                reset = color(RESET)
            );
            row_parts.push(num_str);
        }
        row_parts.push(icon_str);
        row_parts.push(name_display);

        // Add inline preview for directories if there's space
        if item.is_dir && config.inline_preview {
            let term_width = get_terminal_width();
            let current_row_len = display_width(&row_parts.join(" "));
            let available_for_preview = term_width.saturating_sub(current_row_len + 4); // +4 for spacing
            if term_width > 0 && available_for_preview > 10 {
                if let Some(preview) = get_dir_inline_preview(item, available_for_preview) {
                    row_parts.push(format!("{}│{}", color(DIM), color(RESET)));
                    row_parts.push(preview);
                }
            }
        }

        // Apply recency-based highlight to entire row.
        // Recent files get a background highlight, old files get no highlight.
        let row_str = item
            .modified
            .as_ref()
            .map(|dt| {
                let row = row_parts.join(" ");
                if is_recent(dt) {
                    // Recent file - apply highlight if configured
                    if let Some(ref bg) = opts.highlight_recent {
                        if !bg.is_empty() && bg != "none" {
                            highlight_row(&row, bg)
                        } else {
                            row
                        }
                    } else {
                        row
                    }
                } else {
                    // Old file - apply highlight if configured
                    if let Some(ref bg) = opts.highlight_old {
                        if !bg.is_empty() && bg != "none" {
                            highlight_row(&row, bg)
                        } else {
                            row
                        }
                    } else {
                        row
                    }
                }
            })
            .unwrap_or_else(|| row_parts.join(" "));

        let formatted_row = if config.zebra_rows && idx % 2 == 0 {
            apply_row_tint(&row_str, ROW_TINT)
        } else {
            row_str
        };

        println!("{}", formatted_row);
    }
    if _profile {
        eprintln!("[FAB_PROFILE] row loop + widths: {:?}", _t1.elapsed());
    }

    // Show smart truncation summary for big folders
    if hidden_count > 0 && config.smart_truncation {
        // Build a set of displayed paths once so the per-item hidden counters
        // are O(N) total instead of O(N*M) for each category.
        let displayed_paths: std::collections::HashSet<&std::path::Path> =
            display_items.iter().map(|d| d.path.as_path()).collect();
        let hidden_dirs = summary
            .top_items
            .iter()
            .filter(|i| i.is_dir && !displayed_paths.contains(i.path.as_path()))
            .count();
        let hidden_files = summary
            .top_items
            .iter()
            .filter(|i| i.is_file && !displayed_paths.contains(i.path.as_path()))
            .count();
        if hidden_dirs > 0 || hidden_files > 0 {
            let mut parts = Vec::new();
            if hidden_dirs > 0 {
                parts.push(format!(
                    "{}{} dirs{}",
                    color(DIM),
                    hidden_dirs,
                    color(RESET)
                ));
            }
            if hidden_files > 0 {
                parts.push(format!(
                    "{}{} files{}",
                    color(DIM),
                    hidden_files,
                    color(RESET)
                ));
            }
            println!(
                "  {} {} hidden (sorted by git status & recency){}",
                color(DIM),
                parts.join(", "),
                color(RESET)
            );
        }
    }

    // Show mini tree on the right side if there's enough terminal space
    if config.mini_tree {
        let term_width = get_terminal_width();
        if term_width > 120 {
            let tree_width = (term_width / 3).min(40); // Use 1/3 of terminal, max 40 chars
            let dirs: Vec<&crate::fs::DirEntry> = summary
                .top_items
                .iter()
                .filter(|i| i.is_dir && !i.name.starts_with('.'))
                .take(5)
                .collect();
            if !dirs.is_empty() {
                let mut tree_lines = Vec::new();
                tree_lines.push(format!(
                    "{}{}{}",
                    color(BOLD),
                    path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.display().to_string()),
                    color(RESET)
                ));
                for (i, dir) in dirs.iter().enumerate() {
                    let is_last = i == dirs.len() - 1;
                    let connector = if is_last { "└── " } else { "├── " };
                    let name = if tree_width >= 7 && dir.name.len() > tree_width - 4 {
                        // Truncate on a UTF-8 char boundary so the slice can
                        // never split a multibyte codepoint and panic.
                        let keep = (tree_width - 7).min(dir.name.len());
                        let end = dir
                            .name
                            .char_indices()
                            .take_while(|(i, _)| *i < keep)
                            .last()
                            .map(|(i, c)| i + c.len_utf8())
                            .unwrap_or(keep);
                        format!("{}...{}", &dir.name[..end], color(RESET))
                    } else {
                        dir.name.clone()
                    };
                    tree_lines.push(format!(
                        "{}{}{}{}{}",
                        color(DIM),
                        connector,
                        color(BLUE_BOLD),
                        name,
                        color(RESET)
                    ));
                }
                // Print tree aligned to the right
                let tree_str = tree_lines.join("\n");
                let tree_display_width = tree_str.lines().map(display_width).max().unwrap_or(0);
                let padding = " ".repeat(term_width.saturating_sub(tree_display_width + 5));
                for (i, line) in tree_str.lines().enumerate() {
                    if i == 0 {
                        println!("{}{}{}", padding, color(DIM), line);
                    } else {
                        println!("{}{}", padding, line);
                    }
                }
            }
        }
    }
}

fn format_size_for_mode(bytes: u64, mode: &str) -> String {
    match mode {
        "bytes" => bytes.to_string(),
        "short" => {
            const UNITS: [(u64, &str); 5] = [
                (1024 * 1024 * 1024 * 1024, "T"),
                (1024 * 1024 * 1024, "G"),
                (1024 * 1024, "M"),
                (1024, "k"),
                (1, ""),
            ];
            let (divisor, suffix) = UNITS
                .iter()
                .find(|(divisor, _)| bytes >= *divisor)
                .copied()
                .unwrap_or((1, ""));
            format!("{}{}", bytes / divisor, suffix)
        }
        _ => format_size_compact(bytes),
    }
}

fn output_raw(
    path: &Path,
    summary: &DirSummary,
    git_info: &GitInfo,
    opts: &BannerOptions,
    config: &crate::state::Config,
) {
    let (display_items, _) = build_display_items(path, summary, git_info, opts, config, false);
    for item in display_items {
        println!("{}", item.path.display());
    }
}

/// One file per line output (like ls -1)
fn output_oneline(
    path: &Path,
    summary: &DirSummary,
    git_info: &GitInfo,
    opts: &BannerOptions,
    config: &crate::state::Config,
) {
    let (display_items, _) = build_display_items(path, summary, git_info, opts, config, false);
    for item in display_items {
        println!("{}", escape_terminal_text(&item.name));
    }
}

/// Simple glob matching for a single pattern against a filename
fn glob_match(pattern: &str, name: &str) -> bool {
    if let Some(inner) = pattern.strip_prefix('*').and_then(|s| s.strip_suffix('*')) {
        // *foo* - contains
        name.contains(inner)
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        // *.ext - ends with
        name.ends_with(suffix)
    } else if let Some(prefix) = pattern.strip_suffix('*') {
        // prefix* - starts with
        name.starts_with(prefix)
    } else {
        // exact match
        name == pattern
    }
}

/// Recursive directory listing (flat output like ls -R)
fn output_recursive(root: &Path, opts: &BannerOptions) -> Result<()> {
    use std::collections::VecDeque;
    use std::path::PathBuf;

    let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::new();
    queue.push_back((root.to_path_buf(), 0));
    let mut count = 0usize;
    let max = opts.max.unwrap_or(usize::MAX);

    while let Some((dir, depth)) = queue.pop_front() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let mut items: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        items.sort_by_key(|e| e.file_name());

        for entry in items {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Hidden filter
            if !opts.hidden && name_str.starts_with('.') {
                continue;
            }

            // Ignore glob filter
            if opts.ignore_glob.iter().any(|g| glob_match(g, &name_str)) {
                continue;
            }

            // Pattern filter
            if let Some(pat) = opts.filter {
                if !name_str.contains(pat) && !entry.path().to_string_lossy().contains(pat) {
                    continue;
                }
            }

            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            // Only-dirs / only-files filter
            if opts.only_dirs && !is_dir {
                continue;
            }
            if opts.only_files && is_dir {
                continue;
            }

            // Git-ignore filter
            if opts.git_ignore && is_git_ignored(&entry.path()) {
                continue;
            }

            // Output - show relative path from root
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap_or(&path);
            let relative_str = relative.to_string_lossy();
            if opts.oneline {
                println!("{}", escape_terminal_text(&relative_str));
            } else if opts.raw {
                println!("{}", path.display());
            } else {
                // Rich mode - show type indicator
                let prefix = if is_dir { "/" } else { "" };
                println!("{}{}", escape_terminal_text(&relative_str), prefix);
            }

            count += 1;
            if count >= max {
                return Ok(());
            }

            // Queue subdirectories for recursion
            if is_dir {
                let can_descend = match opts.level {
                    None | Some(0) => true,
                    Some(limit) => depth + 1 < limit,
                };
                if can_descend {
                    queue.push_back((entry.path(), depth + 1));
                }
            }
        }
    }

    Ok(())
}

/// Check if a path is git-ignored
fn is_git_ignored(path: &Path) -> bool {
    let name = match path.file_name() {
        Some(n) => n.to_string_lossy(),
        None => return false,
    };

    // Common patterns to ignore
    let ignore_patterns = [
        ".git",
        "node_modules",
        "target",
        "__pycache__",
        ".venv",
        "dist",
        "build",
    ];

    // Check if filename matches
    for pattern in &ignore_patterns {
        if name == *pattern {
            return true;
        }
    }

    // Check if any parent component matches
    let path_str = path.to_string_lossy();
    for pattern in &ignore_patterns {
        if path_str.contains(&format!("/{}/", pattern))
            || path_str.ends_with(&format!("/{}", pattern))
        {
            return true;
        }
    }

    // Check parent .gitignore files
    let mut current = path.parent();
    while let Some(dir) = current {
        let gitignore = dir.join(".gitignore");
        if gitignore.exists() {
            if let Ok(content) = std::fs::read_to_string(&gitignore) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if glob_match(line, &name) {
                        return true;
                    }
                }
            }
        }
        current = dir.parent();
    }

    false
}

fn output_json(
    path: &Path,
    summary: &DirSummary,
    git_info: &GitInfo,
    opts: &BannerOptions,
    config: &crate::state::Config,
) {
    use serde_json::json;

    let (display_items, hidden_count) =
        build_display_items(path, summary, git_info, opts, config, false);
    let items: Vec<_> = display_items
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
        "truncated": summary.truncated,
        "hidden_items": hidden_count,
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

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    );
}

/// Aggregate git status for a directory - returns the most severe status
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

/// Get raw contents description without ANSI colors (for width calculation).
///
/// In `0.6.25` and earlier this always called `get_file_contents`, which
/// re-opens and re-reads each file's header on every `f` invocation. As
/// of `0.6.26` the daemon populates `entry.content_probe` during its
/// directory scan and ships the results in `BannerData`, so the client
/// just reads the pre-computed string. We still fall back to the
/// synchronous probe if the field is missing (e.g. for entries built
/// outside the daemon pipeline, or for non-probed text files where the
/// line count is intentionally not cached).
fn get_file_contents_raw(entry: &crate::fs::DirEntry) -> String {
    if let Some(probe) = entry.content_probe.as_ref() {
        return probe.clone();
    }
    crate::cmd::file_metadata::get_file_contents(entry)
}

/// Get inline preview for a directory (top 2-3 items with icons)
fn get_dir_inline_preview(entry: &crate::fs::DirEntry, max_width: usize) -> Option<String> {
    if !entry.is_dir {
        return None;
    }

    let dir_path = &entry.path;
    let items = match std::fs::read_dir(dir_path) {
        Ok(rd) => rd
            // A preview needs only three visible children. Bound the raw
            // iterator as well so a child containing a huge run of hidden or
            // unreadable entries cannot block the banner render.
            .take(crate::fs::MAX_DIRECTORY_SCAN_ITEMS)
            .filter_map(|e| e.ok())
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .take(3)
            .collect::<Vec<_>>(),
        Err(_) => return None,
    };

    if items.is_empty() {
        return None;
    }

    let mut preview_parts = Vec::new();
    let mut total_len = 0;

    for item in &items {
        let name = item.file_name().to_string_lossy().to_string();
        let is_dir = item.file_type().map(|t| t.is_dir()).unwrap_or(false);

        let icon = if is_dir {
            format!("{}{}{}", color(BLUE_BOLD), name, color(RESET))
        } else {
            format!("{}{}{}", color(DIM), name, color(RESET))
        };

        let plain_len = name.len();
        if total_len + plain_len + 2 > max_width {
            // Add ellipsis if we ran out of space
            if !preview_parts.is_empty() {
                preview_parts.push(format!("{}...{}", color(DIM), color(RESET)));
            }
            break;
        }

        preview_parts.push(icon);
        total_len += plain_len + 2; // +2 for ", "
    }

    if preview_parts.is_empty() {
        return None;
    }

    Some(format!(
        "{}{}{}",
        color(DIM),
        preview_parts.join(" "),
        color(RESET)
    ))
}

/// Natural comparison for version sorting (e.g., file1, file2, file10)
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(&a_c), Some(&b_c)) => {
                if a_c.is_ascii_digit() && b_c.is_ascii_digit() {
                    // Compare numbers
                    let mut a_num = String::new();
                    let mut b_num = String::new();
                    while a_chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        if let Some(c) = a_chars.next() {
                            a_num.push(c);
                        }
                    }
                    while b_chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        if let Some(c) = b_chars.next() {
                            b_num.push(c);
                        }
                    }
                    // Compare arbitrarily long digit runs without parsing to
                    // a bounded integer. Parsing overflow as zero made names
                    // such as `file18446744073709551616` sort before `file2`.
                    let a_trimmed = a_num.trim_start_matches('0');
                    let b_trimmed = b_num.trim_start_matches('0');
                    let a_normalized = if a_trimmed.is_empty() { "0" } else { a_trimmed };
                    let b_normalized = if b_trimmed.is_empty() { "0" } else { b_trimmed };
                    if a_normalized.len() != b_normalized.len() {
                        return a_normalized.len().cmp(&b_normalized.len());
                    }
                    if a_normalized != b_normalized {
                        return a_normalized.cmp(b_normalized);
                    }
                    // Equal numeric values: prefer the shorter spelling, so
                    // `file1` precedes `file01` deterministically.
                    if a_num.len() != b_num.len() {
                        return a_num.len().cmp(&b_num.len());
                    }
                } else {
                    // Compare characters (case-insensitive)
                    let a_lower = a_c.to_ascii_lowercase();
                    let b_lower = b_c.to_ascii_lowercase();
                    if a_lower != b_lower {
                        return a_lower.cmp(&b_lower);
                    }
                    a_chars.next();
                    b_chars.next();
                }
            }
        }
    }
}

/// Convert formatted permission string (e.g. "-rwxr-xr-x" or "drwxr-xr-x" or "rwxr-xr-x") to octal string (e.g. "755")
pub fn perms_to_octal(perms: &str) -> Option<String> {
    let bytes = perms.as_bytes();
    let p = if bytes.len() >= 10 {
        &bytes[1..10]
    } else if bytes.len() >= 9 {
        &bytes[0..9]
    } else {
        return None;
    };

    let parse_triplet = |r: u8, w: u8, x: u8| -> u32 {
        let mut val = 0;
        if r == b'r' {
            val += 4;
        }
        if w == b'w' {
            val += 2;
        }
        if x == b'x' || x == b's' || x == b'S' || x == b't' || x == b'T' {
            val += 1;
        }
        val
    };

    let user = parse_triplet(p[0], p[1], p[2]);
    let group = parse_triplet(p[3], p[4], p[5]);
    let other = parse_triplet(p[6], p[7], p[8]);
    Some(format!("{}{}{}", user, group, other))
}

/// Colorize permission string like exa - each char colored by meaning
/// d=blue, l=magenta, r=green, w=yellow, x=red, -=dim
fn colorize_perms(perms: &str) -> String {
    let mut result = String::with_capacity(perms.len() * 10);
    for c in perms.chars() {
        match c {
            'd' => {
                result.push_str(color(BLUE_BOLD));
                result.push('d');
                result.push_str(color(RESET));
            }
            'l' => {
                result.push_str(color(MAGENTA));
                result.push('l');
                result.push_str(color(RESET));
            }
            'r' => {
                result.push_str(color(GREEN));
                result.push('r');
                result.push_str(color(RESET));
            }
            'w' => {
                result.push_str(color(YELLOW));
                result.push('w');
                result.push_str(color(RESET));
            }
            'x' | 's' | 'S' | 't' | 'T' => {
                result.push_str(color(RED));
                result.push(c);
                result.push_str(color(RESET));
            }
            '-' => {
                result.push_str(color(DIM));
                result.push('-');
                result.push_str(color(RESET));
            }
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

/// Get terminal width (returns 0 if cannot determine)
fn get_terminal_width() -> usize {
    #[cfg(unix)]
    {
        unsafe {
            let mut winsize: libc::winsize = std::mem::zeroed();
            if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut winsize) == 0 {
                winsize.ws_col as usize
            } else {
                0
            }
        }
    }
    #[cfg(not(unix))]
    {
        0
    }
}

/// Get terminal height (returns 0 if cannot determine)
fn get_terminal_height() -> usize {
    #[cfg(unix)]
    {
        unsafe {
            let mut winsize: libc::winsize = std::mem::zeroed();
            if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut winsize) == 0 {
                winsize.ws_row as usize
            } else {
                0
            }
        }
    }
    #[cfg(not(unix))]
    {
        0
    }
}

/// Strip ANSI escape codes from a string
fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some(&'[') => {
                    chars.next();
                    // CSI sequence: consume until final byte (0x40-0x7E, e.g. 'm', 'H', etc.)
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if (0x40..=0x7e).contains(&(next as u32)) {
                            break;
                        }
                    }
                }
                Some(&']') => {
                    chars.next();
                    // OSC sequence: consume until ST (\x1b\\ or \x07)
                    while let Some(next) = chars.next() {
                        if next == '\x07' {
                            break;
                        }
                        if next == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => {
                    let _ = chars.next();
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Measure visual terminal column width of a string after stripping ANSI escape sequences.
fn display_width(s: &str) -> usize {
    let plain = strip_ansi(s);
    unicode_width::UnicodeWidthStr::width(plain.as_str())
}

/// Truncate details to fit within available width
fn truncate_details(details: &[String], available: usize) -> String {
    if details.is_empty() {
        return String::new();
    }

    let mut kept = Vec::new();
    let mut current_len = 0;

    for detail in details {
        let plain = strip_ansi(detail);
        let detail_width = unicode_width::UnicodeWidthStr::width(plain.as_str());
        let item_len = detail_width + if kept.is_empty() { 0 } else { 3 }; // 3 for " │ "

        if current_len + item_len <= available {
            kept.push(detail.clone());
            current_len += item_len;
        } else {
            // Try to fit a truncated version
            let remaining = available.saturating_sub(current_len);
            if remaining > 10 {
                // Truncate this item safely by visual column width
                let mut truncated = String::new();
                let mut width = 0;
                let target_width = remaining.saturating_sub(1); // Leave room for "..."
                for ch in plain.chars() {
                    let ch_w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
                    if width + ch_w > target_width {
                        break;
                    }
                    truncated.push(ch);
                    width += ch_w;
                }
                kept.push(format!("{}...", truncated));
            }
            break;
        }
    }

    kept.join(" │ ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_branch_display_keeps_closing_bracket_in_same_style() {
        set_colors_enabled(true);

        let dirty = GitInfo {
            branch: Some("main".to_string()),
            is_dirty: true,
            ..GitInfo::default()
        };
        assert_eq!(build_branch_display(&dirty), "\x1b[33m[main*]\x1b[0m");

        let clean = GitInfo {
            branch: Some("main".to_string()),
            is_dirty: false,
            ..GitInfo::default()
        };
        assert_eq!(build_branch_display(&clean), "\x1b[1;34m[main]\x1b[0m");
    }

    #[test]
    fn test_build_branch_display_empty_branch() {
        set_colors_enabled(true);

        let no_branch = GitInfo::default();
        assert_eq!(build_branch_display(&no_branch), "");
    }

    #[test]
    fn test_format_loc() {
        assert_eq!(format_loc(0), "0");
        assert_eq!(format_loc(500), "500");
        assert_eq!(format_loc(1000), "1.0k");
        assert_eq!(format_loc(1500), "1.5k");
        assert_eq!(format_loc(10000), "10k");
        assert_eq!(format_loc(123456), "123k");
    }

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("hello"), "hello");
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi("a\x1b[1mb\x1b[0mc"), "abc");
        assert_eq!(
            strip_ansi("\x1b]8;;file:///tmp/file.txt\x1b\\file.txt\x1b]8;;\x1b\\"),
            "file.txt"
        );
    }

    #[test]
    fn test_escape_terminal_text_and_file_url() {
        assert_eq!(escape_terminal_text("a\n\u{1b}[31m"), "a\\u{a}\\u{1b}[31m");
        assert_eq!(
            file_url(Path::new("/tmp/name with#query.txt")),
            "file:///tmp/name%20with%23query.txt"
        );
    }

    #[test]
    fn test_natural_cmp() {
        assert_eq!(natural_cmp("file1", "file2"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("file2", "file10"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("file10", "file2"), std::cmp::Ordering::Greater);
        assert_eq!(natural_cmp("file1", "file1"), std::cmp::Ordering::Equal);
        assert_eq!(
            natural_cmp("file18446744073709551616", "file2"),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn test_format_size_modes() {
        assert_eq!(format_size_for_mode(1536, "default"), "1.5k");
        assert_eq!(format_size_for_mode(1536, "short"), "1k");
        assert_eq!(format_size_for_mode(1536, "bytes"), "1536");
    }

    #[test]
    fn test_highlight_row_is_plain_when_colors_are_disabled() {
        set_colors_enabled(false);
        assert_eq!(highlight_row("plain", "bold"), "plain");
        assert_eq!(highlight_row("plain", "22"), "plain");
        set_colors_enabled(true);
    }

    #[test]
    fn test_display_pipeline_filters_and_applies_max_after_sort() {
        let entries = vec![
            crate::fs::DirEntry {
                name: "small.rs".to_string(),
                path: std::path::PathBuf::from("/tmp/small.rs"),
                is_dir: false,
                is_file: true,
                is_symlink: false,
                is_exec: false,
                size: 1,
                modified: None,
                perms: String::new(),
                owner: String::new(),
                group: String::new(),
                symlink_target: None,
                symlink_valid: true,
                content_probe: None,
            },
            crate::fs::DirEntry {
                name: "large.rs".to_string(),
                path: std::path::PathBuf::from("/tmp/large.rs"),
                is_dir: false,
                is_file: true,
                is_symlink: false,
                is_exec: false,
                size: 100,
                modified: None,
                perms: String::new(),
                owner: String::new(),
                group: String::new(),
                symlink_target: None,
                symlink_valid: true,
                content_probe: None,
            },
            crate::fs::DirEntry {
                name: "skip.txt".to_string(),
                path: std::path::PathBuf::from("/tmp/skip.txt"),
                is_dir: false,
                is_file: true,
                is_symlink: false,
                is_exec: false,
                size: 1000,
                modified: None,
                perms: String::new(),
                owner: String::new(),
                group: String::new(),
                symlink_target: None,
                symlink_valid: true,
                content_probe: None,
            },
        ];
        let summary = crate::fs::DirSummary {
            total_items: entries.len(),
            total_size: 1101,
            files: entries.len(),
            dirs: 0,
            truncated: false,
            top_items: entries,
            project_type: crate::fs::ProjectType::Generic,
            last_modified: None,
            build_status: None,
            todo_info: None,
            code_metrics: None,
            port_info: None,
            docker_info: None,
        };
        let opts = BannerOptions {
            sizesort: true,
            max: Some(1),
            ignore_glob: vec!["*.txt".to_string()],
            ..Default::default()
        };
        let config = crate::state::Config::default();
        let (items, hidden_count) = build_display_items(
            Path::new("/tmp"),
            &summary,
            &GitInfo::default(),
            &opts,
            &config,
            false,
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "large.rs");
        assert_eq!(hidden_count, 1);
    }

    #[test]
    fn test_perms_to_octal() {
        assert_eq!(perms_to_octal("-rwxr-xr-x").as_deref(), Some("755"));
        assert_eq!(perms_to_octal("drwxr-xr-x").as_deref(), Some("755"));
        assert_eq!(perms_to_octal("-rw-r--r--").as_deref(), Some("644"));
        assert_eq!(perms_to_octal("-rwx------").as_deref(), Some("700"));
        assert_eq!(perms_to_octal("lrwxrwxrwx").as_deref(), Some("777"));
        assert_eq!(perms_to_octal("-r-xr-xr-x").as_deref(), Some("555"));
        assert_eq!(perms_to_octal("-rwsr-xr-x").as_deref(), Some("755"));
        assert_eq!(perms_to_octal("rwxr-xr-x").as_deref(), Some("755"));
        assert_eq!(perms_to_octal("short"), None);
    }

    #[test]
    fn test_colorize_perms() {
        let result = colorize_perms("drwxr-xr-x");
        // Should contain ANSI codes
        assert!(result.contains("\x1b["));
        // Should contain the original characters
        assert!(result.contains("d"));
        assert!(result.contains("r"));
        assert!(result.contains("w"));
        assert!(result.contains("x"));
        assert!(result.contains("-"));
    }

    #[test]
    fn test_truncate_details() {
        let details = vec![
            "short".to_string(),
            "medium length".to_string(),
            "this is a very long detail that should be truncated".to_string(),
        ];

        // Test with enough space
        let result = truncate_details(&details, 100);
        assert!(result.contains("short"));
        assert!(result.contains("medium length"));

        // Test with limited space
        let result = truncate_details(&details, 20);
        assert!(result.contains("short"));
    }

    #[test]
    fn test_display_width() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width("\x1b[31mred\x1b[0m"), 3);
        assert_eq!(display_width("🦀"), 2);
        assert_eq!(display_width("📦 crate"), 8);
    }

    #[test]
    fn test_apply_row_tint() {
        set_colors_enabled(true);
        let input = "col1 \x1b[34mcol2\x1b[0m col3";
        let tinted = apply_row_tint(input, ROW_TINT);
        // Should start with ROW_TINT
        assert!(tinted.starts_with(ROW_TINT));
        // Should re-inject ROW_TINT after reset
        assert!(tinted.contains(&format!("{}{}", RESET, ROW_TINT)));
        // Should end with RESET
        assert!(tinted.ends_with(RESET));
    }

    #[test]
    fn test_get_terminal_width() {
        // Should return 0 or a positive number
        let width = get_terminal_width();
        assert!(width <= 1000); // Reasonable upper bound
    }

    #[test]
    fn test_fold_repetitive_patterns() {
        let mut entries = Vec::new();
        // Create 25 entries where 10 share the prefix "release_notes_"
        for i in 1..=10 {
            entries.push(crate::fs::DirEntry {
                name: format!("release_notes_{}.md", i),
                path: std::path::PathBuf::from(format!("/tmp/release_notes_{}.md", i)),
                is_dir: false,
                is_file: true,
                is_symlink: false,
                is_exec: false,
                size: 100,
                modified: Some(chrono::DateTime::from_timestamp(i as i64 * 100, 0).unwrap()),
                perms: String::new(),
                owner: String::new(),
                group: String::new(),
                symlink_target: None,
                symlink_valid: true,
                content_probe: None,
            });
        }
        for i in 1..=15 {
            entries.push(crate::fs::DirEntry {
                name: format!("other_file_{}.rs", i),
                path: std::path::PathBuf::from(format!("/tmp/other_file_{}.rs", i)),
                is_dir: false,
                is_file: true,
                is_symlink: false,
                is_exec: false,
                size: 200,
                modified: Some(chrono::DateTime::from_timestamp(i as i64 * 50, 0).unwrap()),
                perms: String::new(),
                owner: String::new(),
                group: String::new(),
                symlink_target: None,
                symlink_valid: true,
                content_probe: None,
            });
        }

        let mut item_refs: Vec<&crate::fs::DirEntry> = entries.iter().collect();
        assert_eq!(item_refs.len(), 25);
        fold_repetitive_patterns(&mut item_refs, 2);

        // From 10 release notes, only 2 kept (8 dropped). From 15 other files with prefix "other_file_", 2 kept (13 dropped).
        // Total should be 4 (2 + 2)
        assert_eq!(item_refs.len(), 4);
    }
}
