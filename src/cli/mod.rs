//! CLI module — argument parsing and command routing
//!
//! Simplified: just the banner and env commands.

use anyhow::Result;
use clap::{Parser, Subcommand, ValueHint};
use std::path::{Path, PathBuf};

/// cfm — Contextual File Manager
///
/// A directory listing with instant context.
#[derive(Parser, Debug)]
#[command(
    name = "f",
    about = "cfm — Contextual File Manager\nA directory listing with instant context.",
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

        /// Tree view with specified depth (0 = unlimited)
        #[arg(long)]
        tree: Option<Option<usize>>,
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
                filter,
                max,
                group,
                classify,
                tree,
            }) => {
                let p: Option<&Path> = path.as_ref().map(|p| p.as_path());
                crate::cmd::banner::run_banner(&crate::cmd::banner::BannerOptions {
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
                    filter: filter.as_deref(),
                    max: *max,
                    group: *group,
                    tree: *tree,
                })
            }
            None => {
                // `f` with no args = `f banner`
                let p: Option<&Path> = self.path.as_deref();
                crate::cmd::banner::run_banner(&crate::cmd::banner::BannerOptions {
                    path: p,
                    ..Default::default()
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
