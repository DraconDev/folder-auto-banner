//! State persistence — clipboard, pins, sessions
//!
//! Stores state in `~/.local/share/fab/` via the directories crate.

use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Get the fab data directory (~/.local/share/fab/)
pub fn get_data_dir() -> Result<PathBuf> {
    let proj_dirs =
        ProjectDirs::from("com", "fab", "fab").context("Failed to determine data directory")?;
    let data_dir = proj_dirs.data_dir().to_path_buf();

    // Never follow a symlink supplied at the cache root. The directory holds
    // local state and cache files, so redirecting it elsewhere would weaken
    // both the permission guarantee and the expected data boundary.
    if let Ok(metadata) = fs::symlink_metadata(&data_dir) {
        if metadata.file_type().is_symlink() {
            bail!("data directory is a symlink: {:?}", data_dir);
        }
        if !metadata.is_dir() {
            bail!("data directory is not a directory: {:?}", data_dir);
        }
    }

    // Ensure directory exists
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .context(format!("Failed to create data directory: {:?}", data_dir))?;
    }
    // The data dir holds per-path banner caches (listings, git status) —
    // restrict it to this user.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700));
    }

    Ok(data_dir)
}

// === Config ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    // Display preferences
    pub icons: bool,
    pub colors: bool,
    pub compact: bool,
    pub verbose: bool,
    pub max_display_items: usize,

    // Permission display
    pub permission: String, // "rwx", "octal", "disable"

    // Size display
    pub size: String, // "default", "short", "bytes"

    // Date display
    pub date: String, // "date", "relative"

    // File classification
    pub classify: bool, // append */=>@|

    // Symlink display
    pub no_symlink: bool,

    // Total size
    pub total_size: bool,

    // Columns to show
    pub columns: Vec<String>,

    // Columns to hide
    pub hide_columns: Vec<String>,

    // Sorting
    pub sort: String,
    pub reverse: bool,
    pub group_dirs: String, // "none", "first", "last"

    // Display options
    pub hyperlink: bool,          // terminal hyperlinks
    pub hidden: bool,             // show hidden files by default
    pub highlight_recent: String, // background highlight for recent files (e.g., "236", "22", "green")
    pub highlight_old: String,    // background highlight for old files (e.g., "", "none")

    // Features
    pub git_status: bool,
    pub build_status: bool,
    pub todo_count: bool,
    pub languages: bool,
    pub ports: bool,
    pub docker: bool,
    pub inline_preview: bool,   // show inline previews for directories
    pub mini_tree: bool,        // show mini tree on right side when there's space
    pub smart_truncation: bool, // smart truncation for big folders
    pub numbered: bool,         // show [N] numbers next to items for quick navigation
    pub zebra_rows: bool,       // alternating row background tint (disabled by default)
    pub open_command: String,   // command to open files (e.g., "micro", "nano", "vim")

    // Directories to ignore
    pub ignore_dirs: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            icons: true,
            colors: true,
            compact: false,
            verbose: false,
            max_display_items: 0,
            permission: "rwx".to_string(),
            size: "default".to_string(),
            date: "date".to_string(),
            classify: false,
            no_symlink: false,
            total_size: true,
            columns: vec![
                "permission".to_string(),
                "owner".to_string(),
                "group".to_string(),
                "size".to_string(),
                "contents".to_string(),
                "date".to_string(),
                "name".to_string(),
            ],
            hide_columns: vec![],
            sort: "name".to_string(),
            reverse: false,
            group_dirs: "last".to_string(),
            hyperlink: false,
            hidden: false,
            highlight_recent: "bold".to_string(), // bold text for recent files
            highlight_old: "".to_string(),        // no highlight
            git_status: true,
            build_status: false, // opt-in: build checks spawn subprocesses (cargo check ≈ 6.7s)
            todo_count: true,
            languages: true,
            ports: true,
            docker: true,
            inline_preview: false, // disabled by default, can be enabled in config
            mini_tree: false,      // disabled by default - conflicts with inline_preview
            smart_truncation: true, // enabled by default - shows most relevant items first
            numbered: false,       // disabled by default - enable to show [N] navigation numbers
            zebra_rows: false,     // disabled by default for clean pure-black background
            open_command: "micro".to_string(), // default file opener
            ignore_dirs: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".pi".to_string(),
                ".opencode".to_string(),
                ".pi-glla".to_string(),
                ".dracon".to_string(),
                ".svelte-kit".to_string(),
                ".claude".to_string(),
                ".cursor".to_string(),
                ".cache".to_string(),
            ],
        }
    }
}

impl Config {
    #[allow(dead_code)]
    pub fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "fab", "fab")
            .context("Failed to determine config directory")?;
        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if let Ok(path) = path {
            if path.exists() {
                let content = fs::read_to_string(&path).context("Failed to read config")?;
                return toml::from_str(&content).context("Failed to parse config TOML");
            }
        }
        Ok(Config::default())
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        // Atomic write (temp + rename): a torn write or crash mid-write
        // would leave an unparseable config that silently resets to default.
        let tmp = path.with_extension("toml.tmp");
        fs::write(&tmp, &content).context(format!("Failed to write config to {:?}", tmp))?;
        fs::rename(&tmp, &path).context(format!("Failed to write config to {:?}", path))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.icons);
        assert!(config.colors);
        assert!(!config.compact);
        assert_eq!(config.max_display_items, 0);
        assert_eq!(config.permission, "rwx");
        assert_eq!(config.size, "default");
        assert_eq!(config.date, "date");
        assert!(!config.classify);
        assert!(!config.no_symlink);
        assert!(config.total_size);
        assert!(config.git_status);
        assert!(!config.build_status); // opt-in feature, see Default
        assert!(config.todo_count);
        assert!(config.languages);
        assert!(config.ports);
        assert!(config.docker);
        assert!(config.ignore_dirs.contains(&"node_modules".to_string()));
        assert!(config.ignore_dirs.contains(&".pi".to_string()));
        assert!(config.ignore_dirs.contains(&".opencode".to_string()));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("icons = true"));
        assert!(toml_str.contains("colors = true"));
        assert!(toml_str.contains("compact = false"));

        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.icons, config.icons);
        assert_eq!(deserialized.colors, config.colors);
        assert_eq!(deserialized.compact, config.compact);
    }
}
