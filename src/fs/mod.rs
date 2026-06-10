//! Filesystem utilities — directory metadata, file types, project detection
//!
//! Fast, parallel directory walking using ignore crate.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Project type detection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    Ruby,
    Java,
    Cpp,
    CMake,
    Generic,
}

impl ProjectType {
    /// Detect project type from directory contents, walking up to ancestors
    pub fn detect(path: &Path) -> Self {
        let mut current = Some(path);
        let mut depth = 0;
        while let Some(dir) = current {
            if depth >= 10 {
                break;
            } // Limit ancestor traversal
            depth += 1;

            if let Some(project_type) = Self::detect_from_direct_markers(dir) {
                return project_type;
            }

            if let Ok(d) = std::fs::read_dir(dir) {
                for entry in d.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy().to_lowercase();

                    match name_str.as_str() {
                        "cargo.toml" => return ProjectType::Rust,
                        "package.json" => return ProjectType::Node,
                        "pyproject.toml" | "setup.py" | "requirements.txt" | "pipfile" => {
                            return ProjectType::Python
                        }
                        "go.mod" => return ProjectType::Go,
                        "gemfile" => return ProjectType::Ruby,
                        "pom.xml" | "build.gradle" => return ProjectType::Java,
                        "cmakelists.txt" => return ProjectType::CMake,
                        _ => {}
                    }
                }
            }
            current = dir.parent();
        }

        ProjectType::Generic
    }

    fn detect_from_direct_markers(path: &Path) -> Option<Self> {
        let has = |name: &str| path.join(name).exists();
        if has("Cargo.toml") {
            Some(ProjectType::Rust)
        } else if has("package.json") {
            Some(ProjectType::Node)
        } else if has("pyproject.toml") || has("setup.py") || has("requirements.txt") || has("pipfile")
        {
            Some(ProjectType::Python)
        } else if has("go.mod") {
            Some(ProjectType::Go)
        } else if has("gemfile") {
            Some(ProjectType::Ruby)
        } else if has("pom.xml") || has("build.gradle") {
            Some(ProjectType::Java)
        } else if has("CMakeLists.txt") {
            Some(ProjectType::CMake)
        } else {
            None
        }
    }

    /// Get icon for project type
    pub fn icon(&self) -> &'static str {
        match self {
            ProjectType::Rust => "🦀",
            ProjectType::Node => "📦",
            ProjectType::Python => "🐍",
            ProjectType::Go => "🐹",
            ProjectType::Ruby => "💎",
            ProjectType::Java => "☕",
            ProjectType::Cpp | ProjectType::CMake => "⚙️",
            ProjectType::Generic => "📂",
        }
    }

    /// Get label for project type
    pub fn label(&self) -> &'static str {
        match self {
            ProjectType::Rust => "Rust",
            ProjectType::Node => "Node.js",
            ProjectType::Python => "Python",
            ProjectType::Go => "Go",
            ProjectType::Ruby => "Ruby",
            ProjectType::Java => "Java",
            ProjectType::Cpp => "C++",
            ProjectType::CMake => "CMake",
            ProjectType::Generic => "Generic",
        }
    }
}

/// Directory entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_file: bool,
    pub is_symlink: bool,
    #[serde(default)]
    pub is_exec: bool,
    pub size: u64,
    pub modified: Option<DateTime<Utc>>,
    pub perms: String,
    pub owner: String,
    pub group: String,
    #[serde(default)]
    pub symlink_target: Option<String>,
    #[serde(default = "default_true")]
    pub symlink_valid: bool,
}

fn default_true() -> bool {
    true
}

/// Directory summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirSummary {
    pub total_items: usize,
    pub total_size: u64,
    pub files: usize,
    pub dirs: usize,
    pub top_items: Vec<DirEntry>,
    pub project_type: ProjectType,
    pub last_modified: Option<DateTime<Utc>>,
    pub build_status: Option<crate::build_status::BuildStatus>,
    pub todo_info: Option<crate::todo_scanner::TodoInfo>,
    pub code_metrics: Option<crate::code_metrics::CodeMetrics>,
    pub port_info: Option<crate::port_usage::PortInfo>,
    pub docker_info: Option<crate::docker::DockerInfo>,
}

impl DirSummary {
    /// Scan a directory and gather metadata
    #[allow(dead_code)]
    pub fn scan(path: &Path) -> Result<Self> {
        Self::scan_with_options(path, true, true, true, true, true)
    }

    /// Scan a directory with feature flags
    pub fn scan_with_options(
        path: &Path,
        check_build: bool,
        scan_todos: bool,
        check_ports: bool,
        check_docker: bool,
        check_metrics: bool,
    ) -> Result<Self> {
        let project_type = ProjectType::detect(path);
        let mut total_size: u64 = 0;
        let mut files = 0;
        let mut dirs = 0;

        // Pre-load uid/gid caches to avoid reading /etc/passwd and /etc/group per file
        let uid_cache = load_uid_cache();
        let gid_cache = load_gid_cache();
        let mut top_items = Vec::new();
        let mut last_modified: Option<DateTime<Utc>> = None;

        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => return Err(e.into()),
        };

        for entry in entries.flatten() {
            let file_type = entry.file_type();
            let is_dir = file_type.map(|ft| ft.is_dir()).unwrap_or(false);
            let is_file = file_type.map(|ft| ft.is_file()).unwrap_or(false);
            let is_symlink = file_type.map(|ft| ft.is_symlink()).unwrap_or(false);

            if is_dir {
                dirs += 1;
            } else if is_file {
                files += 1;
            }

            // Try to get metadata — for symlinks, use symlink_metadata to read the link itself
            let metadata = if is_symlink {
                // For symlinks, try symlink_metadata first (reads the link itself)
                // If that fails, the symlink might be broken but we can still read permissions
                std::fs::symlink_metadata(entry.path()).ok().or_else(|| {
                    // Fallback: try to read metadata from the parent directory
                    entry.metadata().ok()
                })
            } else {
                entry.metadata().ok()
            };

            // For symlinks, update is_dir based on the target
            let is_dir = if is_symlink {
                // Try to resolve the target to determine if it points to a dir
                std::fs::metadata(entry.path())
                    .map(|m| m.is_dir())
                    .unwrap_or(is_dir)
            } else {
                is_dir
            };

            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            total_size += size;

            // Get modified time
            let modified = metadata
                .as_ref()
                .and_then(|m| m.modified().ok())
                .map(DateTime::<Utc>::from);

            if let Some(mod_time) = modified {
                if last_modified.is_none() || mod_time > last_modified.unwrap() {
                    last_modified = Some(mod_time);
                }
            }

            // Unix permissions + owner/group
            let (perms, is_exec, owner, group) = if let Some(meta) = &metadata {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = meta.permissions().mode();
                    let perms_str = format_mode(mode);
                    let exec = mode & 0o111 != 0;

                    // Resolve uid/gid to names via /etc/passwd and /etc/group
                    use std::os::unix::fs::MetadataExt;
                    let uid = meta.uid();
                    let gid = meta.gid();
                    let owner = uid_cache
                        .get(&uid)
                        .cloned()
                        .unwrap_or_else(|| uid.to_string());
                    let group = gid_cache
                        .get(&gid)
                        .cloned()
                        .unwrap_or_else(|| gid.to_string());

                    (perms_str, exec, owner, group)
                }
                #[cfg(not(unix))]
                {
                    let read_only = meta.permissions().readonly();
                    let perms_str = if read_only {
                        "r--r--r--".to_string()
                    } else {
                        "rw-rw-rw-".to_string()
                    };
                    (perms_str, false, "?".to_string(), "?".to_string())
                }
            } else if is_symlink {
                // For broken symlinks, show symlink permissions (usually lrwxrwxrwx)
                #[cfg(unix)]
                {
                    (
                        "lrwxrwxrwx".to_string(),
                        false,
                        "?".to_string(),
                        "?".to_string(),
                    )
                }
                #[cfg(not(unix))]
                {
                    (
                        "----------".to_string(),
                        false,
                        "?".to_string(),
                        "?".to_string(),
                    )
                }
            } else {
                (
                    "----------".to_string(),
                    false,
                    "?".to_string(),
                    "?".to_string(),
                )
            };

            // Symlink target and validity
            let (symlink_target, symlink_valid) = if is_symlink {
                let target = std::fs::read_link(entry.path()).ok().map(|p| {
                    // Try to resolve to absolute path for display
                    if p.is_absolute() {
                        p.to_string_lossy().to_string()
                    } else {
                        // Resolve relative path from parent directory
                        let parent = entry.path().parent().unwrap_or_else(|| Path::new("."));
                        let absolute = parent.join(&p);
                        if let Ok(canonical) = absolute.canonicalize() {
                            canonical.to_string_lossy().to_string()
                        } else {
                            // Can't resolve (dead symlink), show relative path
                            p.to_string_lossy().to_string()
                        }
                    }
                });
                let valid = target.is_some() && std::fs::metadata(entry.path()).is_ok();
                (target, valid)
            } else {
                (None, true)
            };

            top_items.push(DirEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_path_buf(),
                is_dir,
                is_file,
                is_symlink,
                is_exec,
                size,
                modified,
                perms,
                owner,
                group,
                symlink_target,
                symlink_valid,
            });
        }

        // Sort: directories first, then by name.
        // `to_lowercase` allocates; cache the key once per item instead of
        // lowercasing on every comparison.
        top_items.sort_by_cached_key(|item| (!item.is_dir, item.name.to_lowercase()));

        // Run optional checks with caching
        let cache = crate::cache::Cache::new().ok();

        macro_rules! cached_check {
            ($enabled:expr, $cache:expr, $key:expr, $ttl:expr, $compute:expr) => {
                if $enabled {
                    if let Some(ref cache) = $cache {
                        let ck = crate::cache::cache_key(path, $key);
                        if let Some(cached) = cache.get(&ck, std::time::Duration::from_secs($ttl)) {
                            Some(cached)
                        } else {
                            let result = $compute;
                            if let Some(ref r) = result {
                                if let Err(e) = cache.set(&ck, r.clone()) {
                                    tracing::warn!("Failed to cache {}: {}", $key, e);
                                }
                            }
                            result
                        }
                    } else {
                        $compute
                    }
                } else {
                    None
                }
            };
        }

        let build_status = cached_check!(
            check_build,
            cache,
            "build",
            30,
            crate::build_status::check_build(path, &project_type)
        );
        let todo_info = cached_check!(
            scan_todos,
            cache,
            "todos",
            60,
            crate::todo_scanner::scan_todos(path).ok()
        );
        let code_metrics = cached_check!(
            check_metrics,
            cache,
            "metrics",
            60,
            crate::code_metrics::scan_metrics(path).ok()
        );
        let port_info = cached_check!(
            check_ports,
            cache,
            "ports",
            10,
            crate::port_usage::detect_ports(path).ok()
        );
        let docker_info = cached_check!(
            check_docker,
            cache,
            "docker",
            10,
            crate::docker::detect_docker(path).ok()
        );

        Ok(DirSummary {
            total_items: files + dirs,
            total_size,
            files,
            dirs,
            top_items,
            project_type,
            last_modified,
            build_status,
            todo_info,
            code_metrics,
            port_info,
            docker_info,
        })
    }

    /// Get top N items
    #[allow(dead_code)]
    pub fn top_items(&self, n: usize) -> &[DirEntry] {
        &self.top_items[..n.min(self.top_items.len())]
    }

    /// Get remaining count
    #[allow(dead_code)]
    pub fn remaining(&self, n: usize) -> usize {
        self.total_items.saturating_sub(n)
    }
}

/// Human-readable size — compact
#[allow(dead_code)]
pub fn format_size(bytes: u64) -> String {
    use byte_unit::{Byte, UnitType};
    let byte = Byte::from_u64(bytes);
    let adjusted = byte.get_appropriate_unit(UnitType::Binary);
    // Truncate to 1 decimal place for compactness
    let s = format!("{}", adjusted);
    if let Some(dot) = s.find('.') {
        let after_dot = &s[dot + 1..];
        if after_dot.len() > 1 {
            let truncated: String = s.chars().take(dot + 2).collect();
            return truncated;
        }
    }
    s
}

/// Human-readable size — very compact (like exa: 4.3k, 1.1k, 983)
pub fn format_size_compact(bytes: u64) -> String {
    if bytes == 0 {
        "0".to_string()
    } else if bytes < 1024 {
        format!("{}", bytes)
    } else if bytes < 1024 * 1024 {
        let kb = bytes as f64 / 1024.0;
        if kb >= 10.0 {
            format!("{:.0}k", kb)
        } else {
            format!("{:.1}k", kb)
        }
    } else if bytes < 1024 * 1024 * 1024 {
        let mb = bytes as f64 / (1024.0 * 1024.0);
        if mb >= 10.0 {
            format!("{:.0}M", mb)
        } else {
            format!("{:.1}M", mb)
        }
    } else {
        let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        if gb >= 10.0 {
            format!("{:.0}G", gb)
        } else {
            format!("{:.1}G", gb)
        }
    }
}

/// Format exact date/time — ISO-style with time: "2026-05-27 23:06:44"
pub fn format_exact_time(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format relative time
pub fn format_relative_time(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);

    if duration.num_days() > 365 {
        format!("{} year(s) ago", duration.num_days() / 365)
    } else if duration.num_days() > 30 {
        format!("{} month(s) ago", duration.num_days() / 30)
    } else if duration.num_days() > 0 {
        format!("{} day(s) ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hour(s) ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minute(s) ago", duration.num_minutes())
    } else {
        "just now".to_string()
    }
}

impl DirSummary {
    /// Get file type breakdown
    #[allow(dead_code)]
    pub fn by_type(&self) -> Vec<(String, usize)> {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for entry in &self.top_items {
            let ext = if let Some(dot) = entry.name.rfind('.') {
                entry.name[dot + 1..].to_lowercase()
            } else {
                "other".to_string()
            };
            *counts.entry(ext).or_insert(0) += 1;
        }
        counts.into_iter().collect()
    }
}

/// Format Unix file mode to drwxr-xrwx string (10 chars like `ls -l`)
#[cfg(unix)]
fn format_mode(mode: u32) -> String {
    // File type
    let ft = match mode & 0o170000 {
        0o040000 => 'd',
        0o120000 => 'l',
        0o010000 => 'p',
        0o020000 => 'c',
        0o060000 => 'b',
        0o140000 => 's',
        _ => '-',
    };
    let user_r = if mode & 0o400 != 0 { 'r' } else { '-' };
    let user_w = if mode & 0o200 != 0 { 'w' } else { '-' };
    let user_x = if mode & 0o100 != 0 { 'x' } else { '-' };
    let group_r = if mode & 0o040 != 0 { 'r' } else { '-' };
    let group_w = if mode & 0o020 != 0 { 'w' } else { '-' };
    let group_x = if mode & 0o010 != 0 { 'x' } else { '-' };
    let other_r = if mode & 0o004 != 0 { 'r' } else { '-' };
    let other_w = if mode & 0o002 != 0 { 'w' } else { '-' };
    let other_x = if mode & 0o001 != 0 { 'x' } else { '-' };
    format!(
        "{}{}{}{}{}{}{}{}{}{}",
        ft, user_r, user_w, user_x, group_r, group_w, group_x, other_r, other_w, other_x
    )
}

/// Load uid→username cache from /etc/passwd
#[cfg(unix)]
fn load_uid_cache() -> std::collections::HashMap<u32, String> {
    let mut cache = std::collections::HashMap::new();
    if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(uid) = parts[2].parse::<u32>() {
                    cache.insert(uid, parts[0].to_string());
                }
            }
        }
    }
    cache
}

/// Load gid→groupname cache from /etc/group
#[cfg(unix)]
fn load_gid_cache() -> std::collections::HashMap<u32, String> {
    let mut cache = std::collections::HashMap::new();
    if let Ok(content) = std::fs::read_to_string("/etc/group") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(gid) = parts[2].parse::<u32>() {
                    cache.insert(gid, parts[0].to_string());
                }
            }
        }
    }
    cache
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_format_size() {
        let result_0 = format_size(0);
        let result_1023 = format_size(1023);
        let result_1024 = format_size(1024);
        eprintln!("format_size(0) = {:?}", result_0);
        eprintln!("format_size(1023) = {:?}", result_1023);
        eprintln!("format_size(1024) = {:?}", result_1024);
        assert!(result_0.contains("0"));
        assert!(result_1023.contains("1023"));
        assert!(result_1024.contains("1"));
    }

    #[test]
    fn test_format_size_compact() {
        assert_eq!(format_size_compact(0), "0");
        assert_eq!(format_size_compact(500), "500");
        assert_eq!(format_size_compact(1024), "1.0k");
        assert_eq!(format_size_compact(10 * 1024), "10k");
        assert_eq!(format_size_compact(1024 * 1024), "1.0M");
        assert_eq!(format_size_compact(1024 * 1024 * 1024), "1.0G");
    }

    #[test]
    fn test_project_type_detect_rust() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(ProjectType::detect(tmp.path()), ProjectType::Rust);
    }

    #[test]
    fn test_project_type_detect_node() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("package.json"), "").unwrap();
        assert_eq!(ProjectType::detect(tmp.path()), ProjectType::Node);
    }

    #[test]
    fn test_project_type_detect_python() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("pyproject.toml"), "").unwrap();
        assert_eq!(ProjectType::detect(tmp.path()), ProjectType::Python);
    }

    #[test]
    fn test_project_type_detect_go() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("go.mod"), "").unwrap();
        assert_eq!(ProjectType::detect(tmp.path()), ProjectType::Go);
    }

    #[test]
    fn test_project_type_generic() {
        // Create temp dir in /var/tmp to avoid project marker detection from ancestors
        let tmp = tempfile::Builder::new().tempdir_in("/var/tmp").unwrap();
        assert_eq!(ProjectType::detect(tmp.path()), ProjectType::Generic);
    }

    #[test]
    fn test_project_type_icons() {
        assert_eq!(ProjectType::Rust.icon(), "🦀");
        assert_eq!(ProjectType::Node.icon(), "📦");
        assert_eq!(ProjectType::Python.icon(), "🐍");
        assert_eq!(ProjectType::Go.icon(), "🐹");
        assert_eq!(ProjectType::Generic.icon(), "📂");
    }

    #[test]
    fn test_project_type_labels() {
        assert_eq!(ProjectType::Rust.label(), "Rust");
        assert_eq!(ProjectType::Node.label(), "Node.js");
        assert_eq!(ProjectType::Python.label(), "Python");
    }

    #[test]
    fn test_format_exact_time() {
        let dt = chrono::Utc
            .with_ymd_and_hms(2024, 1, 15, 10, 30, 0)
            .unwrap();
        let formatted = format_exact_time(&dt);
        assert_eq!(formatted, "2024-01-15 10:30:00");
    }

    #[test]
    fn test_format_relative_time() {
        let now = chrono::Utc::now();
        let five_min_ago = now - chrono::Duration::minutes(5);
        let result = format_relative_time(&five_min_ago);
        assert!(result.contains("5"));
        assert!(result.contains("minute"));
    }

    #[test]
    fn test_dir_summary_by_type() {
        let summary = DirSummary {
            total_items: 3,
            total_size: 100,
            files: 3,
            dirs: 0,
            top_items: vec![
                DirEntry {
                    name: "test.rs".into(),
                    path: "".into(),
                    is_dir: false,
                    is_file: true,
                    is_symlink: false,
                    is_exec: false,
                    size: 50,
                    modified: None,
                    perms: String::new(),
                    owner: String::new(),
                    group: String::new(),
                    symlink_target: None,
                    symlink_valid: true,
                },
                DirEntry {
                    name: "main.rs".into(),
                    path: "".into(),
                    is_dir: false,
                    is_file: true,
                    is_symlink: false,
                    is_exec: false,
                    size: 30,
                    modified: None,
                    perms: String::new(),
                    owner: String::new(),
                    group: String::new(),
                    symlink_target: None,
                    symlink_valid: true,
                },
                DirEntry {
                    name: "readme.md".into(),
                    path: "".into(),
                    is_dir: false,
                    is_file: true,
                    is_symlink: false,
                    is_exec: false,
                    size: 20,
                    modified: None,
                    perms: String::new(),
                    owner: String::new(),
                    group: String::new(),
                    symlink_target: None,
                    symlink_valid: true,
                },
            ],
            project_type: ProjectType::Rust,
            last_modified: None,
            build_status: None,
            todo_info: None,
            code_metrics: None,
            port_info: None,
            docker_info: None,
        };

        let by_type = summary.by_type();
        assert_eq!(by_type.len(), 2); // rs and md
    }
}
