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

    /// Output shell aliases for current project type
    Env {
        /// Directory to analyze (default: current directory)
        #[arg(value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,

        /// Output format (shell or json)
        #[arg(long, default_value = "shell")]
        format: Option<String>,
    },
}

impl Cli {
    pub fn run(&self) -> Result<()> {
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
                    verbose: *verbose,
                    sort: sort.as_deref(),
                    reverse: *reverse,
                    hidden: *hidden,
                    filter: filter.as_deref(),
                    max: *max,
                    group: *group,
                    tree: tree.clone(),
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
