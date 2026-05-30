//! CLI module — argument parsing and command routing
//!
//! All commands follow the "ephemeral" rule: wake up, read state, print output, exit.

use anyhow::Result;
use clap::{Parser, Subcommand, ValueHint};
use std::path::{Path, PathBuf};

/// cfm — Contextual File Manager
///
/// An ephemeral, zero-hostage intelligence layer for the shell.
/// Type `fm`. Get context.
#[derive(Parser, Debug)]
#[command(
    name = "f",
    about = "cfm — Contextual File Manager\nAn ephemeral, zero-hostage intelligence layer for the shell.",
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

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // === Phase 2: The Banner ===
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

        /// Disable build status check
        #[arg(long)]
        no_build_check: bool,

        /// Disable TODO/FIXME scanning
        #[arg(long)]
        no_todos: bool,

        /// Disable port detection
        #[arg(long)]
        no_ports: bool,

        /// Disable Docker detection
        #[arg(long)]
        no_docker: bool,

        /// Disable code metrics
        #[arg(long)]
        no_metrics: bool,

        /// Sort order: name, size, date, type
        #[arg(long, value_parser = ["name", "size", "date", "type"])]
        sort: Option<String>,

        /// Reverse sort order
        #[arg(long)]
        reverse: bool,

        /// Show hidden files (dotfiles)
        #[arg(long)]
        hidden: bool,

        /// Filter items by pattern (glob or extension)
        #[arg(short, long)]
        filter: Option<String>,

        /// Maximum number of items to display
        #[arg(short, long)]
        max: Option<usize>,

        /// Group items by type (dirs, files, symlinks)
        #[arg(long)]
        group: bool,

        /// Tree view with specified depth (0 = unlimited)
        #[arg(long)]
        tree: Option<Option<usize>>,
    },

    // === Phase 4: Context-Aware Environment ===
    /// Output shell aliases for current project type
    Env {
        /// Directory to analyze (default: current directory)
        #[arg(value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,

        /// Output format (shell or json)
        #[arg(long, default_value = "shell")]
        format: Option<String>,
    },

    // === Phase 6: Ephemeral Clipboard ===
    /// Yank (copy) files to the clipboard
    Yank {
        /// Files or patterns to yank
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },

    /// Paste (copy) yanked files to current directory
    Paste {
        /// Move instead of copy
        #[arg(short, long)]
        move_files: bool,

        /// Overwrite existing files
        #[arg(short, long)]
        overwrite: bool,
    },

    /// Show current clipboard contents
    Clipboard {
        /// Clear the clipboard
        #[arg(short, long)]
        clear: bool,
    },

    // === Phase 5: File Operations ===
    /// Move files with visual split-context dashboard
    Mv {
        /// Source files to move
        #[arg(required = true)]
        sources: Vec<PathBuf>,

        /// Destination directory
        #[arg(required = true, value_hint = ValueHint::DirPath)]
        dest: PathBuf,

        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,

        /// Rename conflicting files
        #[arg(long)]
        rename: bool,

        /// Skip conflicting files
        #[arg(long)]
        skip: bool,

        /// Dry run (preview only)
        #[arg(long = "dry-run")]
        dry_run: bool,
    },

    /// Copy files
    Cp {
        /// Source files to copy
        #[arg(required = true)]
        sources: Vec<PathBuf>,

        /// Destination directory
        #[arg(required = true, value_hint = ValueHint::DirPath)]
        dest: PathBuf,

        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,

        /// Dry run (preview only)
        #[arg(long = "dry-run")]
        dry_run: bool,
    },

    // === Phase 7: Safe File Operations ===
    /// Remove files (permanent delete)
    Rm {
        /// Files to remove
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Force (skip confirmation)
        #[arg(short = 'f', long)]
        force: bool,

        /// Dry run (preview only)
        #[arg(long = "dry-run")]
        dry_run: bool,
    },

    /// Move files to trash
    Trash {
        /// Files to trash
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Force (skip confirmation)
        #[arg(short = 'f', long)]
        force: bool,

        /// Dry run (preview only)
        #[arg(long = "dry-run")]
        dry_run: bool,
    },

    /// Open files with default application
    Open {
        /// Files to open
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Dry run (preview only)
        #[arg(long = "dry-run")]
        dry_run: bool,
    },

    // === Phase 8: Smart Piping ===
    /// Act on piped file paths (pipe destination)
    Do {
        /// Action to perform (list, count, delete, open, cat, or custom command with {})
        #[arg(default_value = "list")]
        action: Option<String>,

        /// Dry run (preview only)
        #[arg(long = "dry-run")]
        dry_run: bool,
    },

    /// Preview file with syntax highlighting
    Peek {
        /// File to preview
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        file: PathBuf,

        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },

    // === Phase 9: Directory Stats ===
    /// Deep directory synthesis chart
    Stats {
        /// Directory to analyze (default: current)
        #[arg(value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,

        /// Output JSON
        #[arg(long)]
        json: bool,
    },

    // === Phase 10: Spatial Memory ===
    /// Pin a directory
    Pin {
        /// Name for the pin
        #[arg(required = true)]
        name: String,
    },

    /// Jump to a pinned directory
    Jump {
        /// Name of the pin
        #[arg(required = true)]
        name: String,

        /// Just print the cd command (for shell wrapper)
        #[arg(long)]
        print_cd: bool,
    },

    /// Jump to git repo root
    Root {
        /// Just print the cd command (for shell wrapper)
        #[arg(long)]
        print_cd: bool,
    },

    /// List all pins
    Pins,

    /// Unpin a directory
    Unpin {
        /// Name of the pin to remove
        #[arg(required = true)]
        name: String,
    },

    // === Phase 11: Session Management ===
    /// Save current session
    SaveSession {
        /// Name for the session
        #[arg(required = true)]
        name: String,

        /// Optional description
        #[arg(long = "desc")]
        description: Option<String>,
    },

    /// Load a saved session
    LoadSession {
        /// Name of the session
        #[arg(required = true)]
        name: String,

        /// Just print the cd command (for shell wrapper)
        #[arg(long)]
        print_cd: bool,
    },

    /// List all sessions
    Sessions,

    /// Delete a session
    DeleteSession {
        /// Name of the session to delete
        #[arg(required = true)]
        name: String,
    },

    // === Phase 12: Directory Comparison ===
    /// Compare two directories
    Diff {
        /// First directory
        #[arg(required = true, value_hint = ValueHint::DirPath)]
        dir1: PathBuf,

        /// Second directory
        #[arg(required = true, value_hint = ValueHint::DirPath)]
        dir2: PathBuf,

        /// Shallow comparison (top-level only)
        #[arg(long)]
        shallow: bool,

        /// Output JSON
        #[arg(long)]
        json: bool,
    },

    // === Phase 3: Shell Integration ===
    /// Install shell hooks (Zsh/Bash)
    InstallHook {
        /// Shell to install for (auto-detect if not specified)
        #[arg(value_parser = clap::value_parser!(String))]
        shell: Option<String>,
    },

    /// Uninstall shell hooks
    UninstallHook,

    /// Generate shell completions
    Completion {
        /// Shell to generate for (bash, zsh, fish, powershell)
        #[arg(required = true)]
        shell: String,
    },

    // === Phase 14: Configuration ===
    /// Edit configuration
    Config {
        /// Open config in editor
        #[arg(long)]
        edit: bool,

        /// Get a specific config value
        #[arg(long)]
        get: Option<String>,

        /// Set a config value
        #[arg(long, value_name = "KEY=VALUE")]
        set: Option<String>,
    },

    // === Daemon Management ===
    /// Manage the cfmd background daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum DaemonAction {
    /// Start the daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Show daemon status
    Status,
    /// Restart the daemon
    Restart,
    /// Clear the daemon cache
    ClearCache,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        use Commands::*;

        // Initialize logging if debug mode
        if self.debug {
            tracing_subscriber::fmt()
                .with_env_filter("cfm=debug,cfm=trace")
                .init();
        } else {
            tracing_subscriber::fmt().with_env_filter("cfm=warn").init();
        }

        match &self.command {
            // Banner (also default when no subcommand)
            Some(Banner {
                path,
                raw,
                json,
                compact,
                no_build_check,
                no_todos,
                no_ports,
                no_docker,
                no_metrics,
                sort,
                reverse,
                hidden,
                filter,
                max,
                group,
                tree,
            }) => {
                let p: Option<&Path> = path.as_ref().map(|p| p.as_path());
                crate::cmd::banner::run_banner(&crate::cmd::banner::BannerOptions {
                    path: p,
                    raw: *raw,
                    json: *json,
                    compact: *compact,
                    no_build_check: *no_build_check,
                    no_todos: *no_todos,
                    no_ports: *no_ports,
                    no_docker: *no_docker,
                    no_metrics: *no_metrics,
                    sort: sort.as_deref(),
                    reverse: *reverse,
                    hidden: *hidden,
                    filter: filter.as_deref(),
                    max: *max,
                    group: *group,
                    tree: *tree,
                })
            }
            None => {
                // `fm` with no args = `fm banner`
                let p: Option<&Path> = self.path.as_deref();
                crate::cmd::banner::run_banner(&crate::cmd::banner::BannerOptions {
                    path: p,
                    ..Default::default()
                })
            }

            // Phase 4: Env
            Some(Env { path, format }) => {
                crate::cmd::env::run_env(path.as_ref().map(|p| p.as_path()), format.as_deref())
            }

            // Phase 6: Clipboard
            Some(Yank { paths }) => crate::cmd::yank::run_yank(paths),
            Some(Paste {
                move_files,
                overwrite,
            }) => crate::cmd::paste::run_paste(*move_files, *overwrite),
            Some(Clipboard { clear }) => crate::cmd::clipboard::run_clipboard(*clear),

            // Phase 5: File ops
            Some(Mv {
                sources,
                dest,
                overwrite,
                rename,
                skip: _,
                dry_run,
            }) => crate::cmd::mv::run_mv(sources, dest, *overwrite, *rename, true, *dry_run),
            Some(Cp {
                sources,
                dest,
                overwrite,
                dry_run,
            }) => crate::cmd::cp::run_cp(&crate::cmd::cp::CpOptions {
                sources,
                dest,
                overwrite: *overwrite,
                rename_on_collision: false,
                verbose: true,
                preserve: true,
                dry_run: *dry_run,
            }),

            // Phase 7: Safe ops
            Some(Rm {
                paths,
                force,
                dry_run,
            }) => crate::cmd::rm::run_rm(paths, false, *force, true, *dry_run),
            Some(Trash {
                paths,
                force: _,
                dry_run,
            }) => crate::cmd::trash::run_trash(paths, true, *dry_run),
            Some(Open {
                paths,
                dry_run,
            }) => crate::cmd::open::run_open(paths, true, *dry_run),

            // Phase 8: Smart piping
            Some(Do {
                action,
                dry_run,
            }) => crate::cmd::do_cmd::run_do(action.as_deref(), true, *dry_run),
            Some(Peek { file, lines }) => crate::cmd::peek::run_peek(file, *lines),

            // Phase 9: Stats
            Some(Stats { path, json }) => {
                let p: Option<&Path> = path.as_ref().map(|p| p.as_path());
                crate::cmd::stats::run_stats(p, *json)
            }

            // Phase 10: Pin/Jump/Root
            Some(Pin { name }) => crate::cmd::pin::run_pin(name),
            Some(Jump { name, print_cd }) => crate::cmd::jump::run_jump(name, *print_cd),
            Some(Root { print_cd }) => crate::cmd::root::run_root(*print_cd),
            Some(Pins) => crate::cmd::pins::run_pins(),
            Some(Unpin { name }) => crate::cmd::unpin::run_unpin(name),

            // Phase 11: Sessions
            Some(SaveSession { name, description }) => {
                crate::cmd::save_session::run_save_session(name, description.as_deref())
            }
            Some(LoadSession { name, print_cd: _ }) => {
                crate::cmd::load_session::run_load_session(name)
            }
            Some(Sessions) => crate::cmd::sessions::run_sessions(),
            Some(DeleteSession { name }) => crate::cmd::delete_session::run_delete_session(name),

            // Phase 12: Diff
            Some(Diff {
                dir1,
                dir2,
                shallow,
                json,
            }) => crate::cmd::diff::run_diff(dir1, dir2, *shallow, *json),

            // Phase 3: Shell integration
            Some(InstallHook { shell }) => {
                crate::cmd::install_hook::run_install_hook(shell.as_deref())
            }
            Some(UninstallHook) => crate::cmd::uninstall_hook::run_uninstall_hook(),
            Some(Completion { shell }) => crate::cmd::completion::run_completion(shell),

            // Phase 14: Config
            Some(Config { edit, get, set }) => {
                crate::cmd::config::run_config(*edit, get.as_deref(), set.as_deref())
            }

            // Daemon management
            Some(Daemon { action }) => crate::cmd::daemon_mgmt::run_daemon(action),
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
