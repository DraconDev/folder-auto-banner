//! State persistence — clipboard, pins, sessions
//!
//! Stores state in `~/.local/share/fab/` via the directories crate.

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Get the fab data directory (~/.local/share/fab/)
pub fn get_data_dir() -> Result<PathBuf> {
    let proj_dirs =
        ProjectDirs::from("com", "fab", "fab").context("Failed to determine data directory")?;
    let data_dir = proj_dirs.data_dir().to_path_buf();

    // Ensure directory exists
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .context(format!("Failed to create data directory: {:?}", data_dir))?;
    }

    Ok(data_dir)
}

// === Clipboard ===

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct ClipboardEntry {
    pub paths: Vec<PathBuf>,
    pub source_dir: PathBuf,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct ClipboardState {
    pub entries: Vec<ClipboardEntry>,
    pub current_index: usize,
}

#[allow(dead_code)]
impl ClipboardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clipboard_path() -> Result<PathBuf> {
        Ok(get_data_dir()?.join("clipboard.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::clipboard_path();
        if let Ok(path) = path {
            if path.exists() {
                let content = fs::read_to_string(&path).context("Failed to read clipboard")?;
                return serde_json::from_str(&content).context("Failed to parse clipboard JSON");
            }
        }
        Ok(Self::new())
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::clipboard_path()?;
        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize clipboard")?;
        fs::write(&path, content).context(format!("Failed to write clipboard to {:?}", path))?;
        Ok(())
    }
}

// === Pins ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Pin {
    pub name: String,
    pub path: PathBuf,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
    pub access_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct PinsState {
    pub pins: Vec<Pin>,
}

#[allow(dead_code)]
impl PinsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pins_path() -> Result<PathBuf> {
        Ok(get_data_dir()?.join("pins.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::pins_path();
        if let Ok(path) = path {
            if path.exists() {
                let content = fs::read_to_string(&path).context("Failed to read pins")?;
                return serde_json::from_str(&content).context("Failed to parse pins JSON");
            }
        }
        Ok(Self::new())
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::pins_path()?;
        let content = serde_json::to_string_pretty(self).context("Failed to serialize pins")?;
        fs::write(&path, content).context(format!("Failed to write pins to {:?}", path))?;
        Ok(())
    }
}

// === Sessions ===

/// Canonical session type used across all session operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Session {
    pub name: String,
    #[serde(alias = "path")]
    pub cwd: PathBuf,
    #[serde(alias = "timestamp")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub git_branch: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl Session {
    /// Create a new session
    #[allow(dead_code)]
    pub fn new(name: &str, cwd: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            cwd,
            created_at: Some(chrono::Utc::now()),
            last_accessed: None,
            git_branch: None,
            description: None,
        }
    }

    /// Get the sessions directory
    #[allow(dead_code)]
    pub fn sessions_dir() -> Result<PathBuf> {
        Ok(get_data_dir()?.join("sessions"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionsState {
    pub sessions: Vec<Session>,
}

#[allow(dead_code)]
impl SessionsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sessions_path() -> Result<PathBuf> {
        Ok(get_data_dir()?.join("sessions.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::sessions_path();
        if let Ok(path) = path {
            if path.exists() {
                let content = fs::read_to_string(&path).context("Failed to read sessions")?;
                return serde_json::from_str(&content).context("Failed to parse sessions JSON");
            }
        }
        Ok(Self::new())
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::sessions_path()?;
        let content = serde_json::to_string_pretty(self).context("Failed to serialize sessions")?;
        fs::write(&path, content).context(format!("Failed to write sessions to {:?}", path))?;
        Ok(())
    }
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
    pub inline_preview: bool,  // show inline previews for directories
    pub mini_tree: bool,       // show mini tree on right side when there's space
    pub smart_truncation: bool, // smart truncation for big folders
    pub numbered: bool,         // show [N] numbers next to items for quick navigation
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
            max_display_items: 8,
            permission: "rwx".to_string(),
            size: "default".to_string(),
            date: "date".to_string(),
            classify: false,
            no_symlink: false,
            total_size: false,
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
            group_dirs: "first".to_string(),
            hyperlink: false,
            hidden: false,
            highlight_recent: "bold".to_string(), // bold text for recent files
            highlight_old: "".to_string(),        // no highlight
            git_status: true,
            build_status: true,
            todo_count: true,
            languages: true,
            ports: true,
            docker: true,
            inline_preview: false, // disabled by default, can be enabled in config
            mini_tree: false,       // disabled by default - conflicts with inline_preview
            smart_truncation: true, // enabled by default - shows most relevant items first
            numbered: false,        // disabled by default - enable to show [N] navigation numbers
            open_command: "micro".to_string(), // default file opener
            ignore_dirs: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                "dist".to_string(),
                "build".to_string(),
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
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&path, content).context(format!("Failed to write config to {:?}", path))?;
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
        assert_eq!(config.max_display_items, 8);
        assert_eq!(config.permission, "rwx");
        assert_eq!(config.size, "default");
        assert_eq!(config.date, "date");
        assert!(!config.classify);
        assert!(!config.no_symlink);
        assert!(!config.total_size);
        assert!(config.git_status);
        assert!(config.build_status);
        assert!(config.todo_count);
        assert!(config.languages);
        assert!(config.ports);
        assert!(config.docker);
        assert!(config.ignore_dirs.contains(&"node_modules".to_string()));
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

    #[test]
    fn test_clipboard_state_default() {
        let state = ClipboardState::default();
        assert!(state.entries.is_empty());
        assert_eq!(state.current_index, 0);
    }

    #[test]
    fn test_pins_state_default() {
        let state = PinsState::default();
        assert!(state.pins.is_empty());
    }

    #[test]
    fn test_sessions_state_default() {
        let state = SessionsState::default();
        assert!(state.sessions.is_empty());
    }
}
