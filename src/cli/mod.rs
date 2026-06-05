//! CLI module — argument parsing and command routing
//!
//! Simplified: just the banner and env commands.

use anyhow::Result;
use clap::{Parser, Subcommand, ValueHint};
use std::path::{Path, PathBuf};

/// f — Folder Auto Banner
///
/// A directory listing with instant context.
#[derive(Parser, Debug)]
#[command(
    name = "f",
    bin_name = "f",
    about = "f — Folder Auto Banner\nA directory listing with instant context.",
    version,
    author
)]
pub struct Cli {
    /// Enable debug output
    #[arg(short, long, global = true)]
    pub debug: bool,

    /// Path (defaults to current directory)
    #[arg(value_hint = ValueHint::DirPath)]
    pub path: Option<PathBuf>,

    /// Sort by time modified
    #[arg(short = 't', long = "timesort")]
    pub timesort: bool,

    /// Sort by size
    #[arg(short = 'S', long = "sizesort")]
    pub sizesort: bool,

    /// Sort by file extension
    #[arg(short = 'X', long = "extensionsort")]
    pub extensionsort: bool,

    /// Sort by git status
    #[arg(short = 'G', long = "gitsort")]
    pub gitsort: bool,

    /// Reverse sort order
    #[arg(short = 'r', long = "reverse")]
    pub reverse: bool,

    /// Show hidden files (dotfiles)
    #[arg(short = 'a', long = "hidden")]
    pub hidden: bool,

    /// One file per line (for piping)
    #[arg(short = '1', long = "oneline")]
    pub oneline: bool,

    /// Filter items by pattern
    #[arg(short = 'f', long = "filter")]
    pub filter: Option<String>,

    /// Maximum number of items to display
    #[arg(short = 'm', long = "max")]
    pub max: Option<usize>,

    /// Tree view with specified depth (0 = unlimited)
    #[arg(long)]
    pub tree: Option<Option<usize>>,

    /// Limit recursion depth (for --tree or --recursive)
    #[arg(short = 'L', long = "level")]
    pub level: Option<usize>,

    /// Compact output (fewer lines)
    #[arg(short = 'c', long = "compact")]
    pub compact: bool,

    /// Verbose output (more info)
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Output raw paths (no formatting, for piping)
    #[arg(long = "raw")]
    pub raw: bool,

    /// Output JSON (for scripting)
    #[arg(long)]
    pub json: bool,

    /// Recurse into directories
    #[arg(short = 'R', long = "recursive")]
    pub recursive: bool,

    /// List only directories
    #[arg(short = 'D', long = "only-dirs")]
    pub only_dirs: bool,

    /// List only files
    #[arg(long = "only-files")]
    pub only_files: bool,

    /// Respect .gitignore
    #[arg(long = "git-ignore")]
    pub git_ignore: bool,

    /// Background highlight for recent files (e.g., "22", "green", "none")
    #[arg(long = "highlight-recent")]
    pub highlight_recent: Option<String>,

    /// Background highlight for old files (e.g., "236", "gray", "none")
    #[arg(long = "highlight-old")]
    pub highlight_old: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Print the contextual directory dashboard
    Banner {
        /// Directory to analyze (default: current directory)
        #[arg(value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,

        /// Output raw paths (no formatting, for piping)
        #[arg(short, long)]
        raw: bool,

        /// Output JSON (for scripting)
        #[arg(long)]
        json: bool,

        /// Compact output (fewer lines)
        #[arg(short, long)]
        compact: bool,

        /// Verbose output (more info)
        #[arg(short, long)]
        verbose: bool,

        /// Sort order: name, size, date, type, git, extension, version
        #[arg(long, value_parser = ["name", "size", "date", "type", "git", "extension", "version"])]
        sort: Option<String>,

        /// Sort by time modified
        #[arg(short = 't', long = "timesort")]
        timesort: bool,

        /// Sort by size
        #[arg(short = 'S', long = "sizesort")]
        sizesort: bool,

        /// Sort by file extension
        #[arg(short = 'X', long = "extensionsort")]
        extensionsort: bool,

        /// Sort by git status
        #[arg(short = 'G', long = "gitsort")]
        gitsort: bool,

        /// Natural sort (version numbers)
        #[arg(long = "versionsort")]
        versionsort: bool,

        /// No sort — list in directory order
        #[arg(short = 'U', long = "no-sort")]
        no_sort: bool,

        /// Group directories first/last
        #[arg(long, value_parser = ["none", "first", "last"])]
        group_dirs: Option<String>,

        /// Reverse sort order
        #[arg(long = "reverse")]
        reverse: bool,

        /// Show hidden files (dotfiles)
        #[arg(long)]
        hidden: bool,

        /// Show relative dates (e.g., "2h ago", "just now")
        #[arg(long)]
        relative_date: bool,

        /// Filter items by pattern (glob or extension)
        #[arg(short, long)]
        filter: Option<String>,

        /// Maximum number of items to display
        #[arg(short, long)]
        max: Option<usize>,

        /// Group items by type (dirs, files, symlinks)
        #[arg(long)]
        group: bool,

        /// Append type indicator (*/=>@|)
        #[arg(long)]
        classify: bool,

        /// Columns to display (comma-separated: permission,owner,group,size,contents,date,name)
        #[arg(long)]
        blocks: Option<String>,

        /// Tree view with specified depth (0 = unlimited)
        #[arg(long)]
        tree: Option<Option<usize>>,

        /// Display one entry per line (for piping)
        #[arg(short = '1', long = "oneline")]
        oneline: bool,

        /// Show total directory size in header
        #[arg(long)]
        total_size: bool,

        /// Exclude files matching glob pattern (can repeat)
        #[arg(long)]
        ignore_glob: Vec<String>,

        /// Hide symlink targets
        #[arg(long)]
        no_symlink: bool,

        /// Attach terminal hyperlinks to filenames
        #[arg(long)]
        hyperlink: bool,

        /// Open file with this program instead of editor (e.g., "cat", "krita", "ranger")
        #[arg(value_name = "ACTION")]
        action: Option<String>,
    },

    /// Output shell aliases for current project type
    Env {
        /// Directory to analyze (default: current directory)
        #[arg(value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,

        /// Output format (shell or json)
        #[arg(long, default_value = "shell")]
        format: Option<String>,
    },

    /// Open configuration file in editor
    Config,

    /// Manage the background daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum DaemonAction {
    /// Stop the daemon
    Stop,
    /// Show daemon status
    Status,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        use Commands::*;

        // Initialize logging if debug mode
        if self.debug {
            tracing_subscriber::fmt()
                .with_env_filter("f=debug,f=trace")
                .init();
        } else {
            tracing_subscriber::fmt().with_env_filter("f=warn").init();
        }

        match &self.command {
            // Banner (also default when no subcommand)
            Some(Banner {
                path,
                raw,
                json,
                compact,
                verbose,
                sort,
                timesort,
                sizesort,
                extensionsort,
                gitsort,
                versionsort,
                no_sort,
                group_dirs,
                reverse,
                hidden,
                relative_date,
                filter,
                max,
                group,
                classify,
                blocks,
                tree,
                oneline,
                total_size,
                ignore_glob,
                no_symlink,
                hyperlink,
            }) => {
                let p: Option<&Path> = path.as_ref().map(|p| p.as_path());
                crate::cmd::banner::run_banner(crate::cmd::banner::BannerOptions {
                    path: p,
                    raw: *raw,
                    json: *json,
                    compact: *compact,
                    verbose: *verbose,
                    sort: sort.as_deref(),
                    timesort: *timesort,
                    sizesort: *sizesort,
                    extensionsort: *extensionsort,
                    gitsort: *gitsort,
                    versionsort: *versionsort,
                    no_sort: *no_sort,
                    group_dirs: group_dirs.as_deref(),
                    reverse: *reverse,
                    hidden: *hidden,
                    relative_date: *relative_date,
                    filter: filter.as_deref(),
                    max: *max,
                    group: *group,
                    classify: *classify,
                    blocks: blocks.as_deref(),
                    tree: *tree,
                    oneline: *oneline,
                    total_size: *total_size,
                    ignore_glob: ignore_glob.clone(),
                    no_symlink: *no_symlink,
                    hyperlink: *hyperlink,
                    ..Default::default()
                })
            }
            None => {
                // `f` with no args = `f banner` — use top-level flags
                let p: Option<&Path> = self.path.as_deref();
                crate::cmd::banner::run_banner(crate::cmd::banner::BannerOptions {
                    path: p,
                    raw: self.raw,
                    json: self.json,
                    compact: self.compact,
                    verbose: self.verbose,
                    sort: None,
                    timesort: self.timesort,
                    sizesort: self.sizesort,
                    extensionsort: self.extensionsort,
                    gitsort: self.gitsort,
                    versionsort: false,
                    no_sort: false,
                    group_dirs: None,
                    reverse: self.reverse,
                    hidden: self.hidden,
                    relative_date: false,
                    filter: self.filter.as_deref(),
                    max: self.max,
                    group: false,
                    classify: false,
                    blocks: None,
                    tree: self.tree,
                    icons: false,
                    colors: false,
                    max_items: 0,
                    oneline: self.oneline,
                    total_size: false,
                    ignore_glob: vec![],
                    no_symlink: false,
                    hyperlink: false,
                    recursive: self.recursive,
                    only_dirs: self.only_dirs,
                    only_files: self.only_files,
                    git_ignore: self.git_ignore,
                    level: self.level,
                    highlight_recent: self.highlight_recent.clone(),
                    highlight_old: self.highlight_old.clone(),
                })
            }

            // Env
            Some(Env { path, format }) => {
                crate::cmd::env::run_env(path.as_ref().map(|p| p.as_path()), format.as_deref())
            }

            // Config
            Some(Config) => {
                let config_path = crate::state::Config::config_path()
                    .map_err(|e| anyhow::anyhow!("Failed to get config path: {}", e))?;

                // Ensure config file exists
                if !config_path.exists() {
                    if let Some(parent) = config_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let default_config = crate::state::Config::default();
                    default_config.save()?;
                    println!("📝 Created config file: {}", config_path.display());
                }

                // Open in editor
                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                let status = std::process::Command::new(&editor)
                    .arg(&config_path)
                    .status();

                match status {
                    Ok(s) => {
                        if s.success() {
                            println!("✅ Config updated: {}", config_path.display());
                        } else {
                            eprintln!("⚠️  Editor exited with status: {}", s);
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to open editor: {}", e);
                        println!("💡 Config file: {}", config_path.display());
                    }
                }

                Ok(())
            }

            // Daemon
            Some(Daemon { action }) => match action {
                DaemonAction::Stop => {
                    if crate::daemon_client::is_daemon_running() {
                        crate::daemon_client::send_shutdown();
                        println!("✅ Daemon stopped");
                    } else {
                        println!("ℹ️  Daemon is not running");
                    }
                    Ok(())
                }
                DaemonAction::Status => {
                    if crate::daemon_client::is_daemon_running() {
                        println!("✅ Daemon is running");
                    } else {
                        println!("ℹ️  Daemon is not running");
                    }
                    Ok(())
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banner_default() {
        let cli = Cli::parse_from(["f"]);
        match cli.command {
            Some(Commands::Banner { .. }) => {}
            None => {} // No subcommand = banner
            _ => panic!("Expected Banner or None, got something else"),
        }
    }
}
