//! Filesystem utilities — directory metadata, file types, project detection
//!
//! Fast, parallel directory walking using ignore crate.

use anyhow::Result;
use chrono::{DateTime, Utc};
use ignore::WalkBuilder;
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
        while let Some(dir) = current {
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
    pub is_exec: bool,
    pub size: u64,
    pub modified: Option<DateTime<Utc>>,
    pub perms: String,
    pub owner: String,
    pub group: String,
    pub symlink_target: Option<String>,
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
        let mut top_items = Vec::new();
        let mut last_modified: Option<DateTime<Utc>> = None;

        let walker = WalkBuilder::new(path)
            .max_depth(Some(1)) // Only immediate directory
            .hidden(false)
            .ignore(false)
            .build();

        for entry in walker.flatten() {
            // Skip the root path itself
            if entry.path() == path {
                continue;
            }

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
                std::fs::symlink_metadata(entry.path()).ok()
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
                    let owner = resolve_uid(uid).unwrap_or_else(|| uid.to_string());
                    let group = resolve_gid(gid).unwrap_or_else(|| gid.to_string());

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
            } else {
                (
                    "----------".to_string(),
                    false,
                    "?".to_string(),
                    "?".to_string(),
                )
            };

            // Symlink target
            let symlink_target = if is_symlink {
                std::fs::read_link(entry.path())
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
            } else {
                None
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
            });
        }

        // Sort: directories first, then by name
        top_items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        // Run optional checks with caching
        let cache = crate::cache::Cache::new().ok();
        let cache_key = |feature: &str| crate::cache::cache_key(path, feature);

        let build_status = if check_build {
            if let Some(ref cache) = cache {
                let key = cache_key("build");
                if let Some(cached) = cache.get(&key, std::time::Duration::from_secs(30)) {
                    Some(cached)
                } else {
                    let result = crate::build_status::check_build(path, &project_type);
                    if let Some(ref r) = result {
                        if let Err(e) = cache.set(&key, r.clone()) {
                            tracing::warn!("Failed to cache build status: {}", e);
                        }
                    }
                    result
                }
            } else {
                crate::build_status::check_build(path, &project_type)
            }
        } else {
            None
        };

        let todo_info = if scan_todos {
            if let Some(ref cache) = cache {
                let key = cache_key("todos");
                if let Some(cached) = cache.get(&key, std::time::Duration::from_secs(60)) {
                    Some(cached)
                } else {
                    let result = crate::todo_scanner::scan_todos(path).ok();
                    if let Some(ref r) = result {
                        if let Err(e) = cache.set(&key, r.clone()) {
                            tracing::warn!("Failed to cache todo info: {}", e);
                        }
                    }
                    result
                }
            } else {
                crate::todo_scanner::scan_todos(path).ok()
            }
        } else {
            None
        };

        let code_metrics = if check_metrics {
            if let Some(ref cache) = cache {
                let key = cache_key("metrics");
                if let Some(cached) = cache.get(&key, std::time::Duration::from_secs(60)) {
                    Some(cached)
                } else {
                    let result = crate::code_metrics::scan_metrics(path).ok();
                    if let Some(ref r) = result {
                        if let Err(e) = cache.set(&key, r.clone()) {
                            tracing::warn!("Failed to cache code metrics: {}", e);
                        }
                    }
                    result
                }
            } else {
                crate::code_metrics::scan_metrics(path).ok()
            }
        } else {
            None
        };

        let port_info = if check_ports {
            if let Some(ref cache) = cache {
                let key = cache_key("ports");
                if let Some(cached) = cache.get(&key, std::time::Duration::from_secs(10)) {
                    Some(cached)
                } else {
                    let result = crate::port_usage::detect_ports(path).ok();
                    if let Some(ref r) = result {
                        if let Err(e) = cache.set(&key, r.clone()) {
                            tracing::warn!("Failed to cache port info: {}", e);
                        }
                    }
                    result
                }
            } else {
                crate::port_usage::detect_ports(path).ok()
            }
        } else {
            None
        };

        let docker_info = if check_docker {
            if let Some(ref cache) = cache {
                let key = cache_key("docker");
                if let Some(cached) = cache.get(&key, std::time::Duration::from_secs(10)) {
                    Some(cached)
                } else {
                    let result = crate::docker::detect_docker(path).ok();
                    if let Some(ref r) = result {
                        if let Err(e) = cache.set(&key, r.clone()) {
                            tracing::warn!("Failed to cache docker info: {}", e);
                        }
                    }
                    result
                }
            } else {
                crate::docker::detect_docker(path).ok()
            }
        } else {
            None
        };

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
#[allow(dead_code)]
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

/// Resolve uid to username from /etc/passwd
#[cfg(unix)]
fn resolve_uid(uid: u32) -> Option<String> {
    let content = std::fs::read_to_string("/etc/passwd").ok()?;
    for line in content.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            if let Ok(file_uid) = parts[2].parse::<u32>() {
                if file_uid == uid {
                    return Some(parts[0].to_string());
                }
            }
        }
    }
    Some(uid.to_string())
}

/// Resolve gid to group name from /etc/group
#[cfg(unix)]
fn resolve_gid(gid: u32) -> Option<String> {
    let content = std::fs::read_to_string("/etc/group").ok()?;
    for line in content.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            if let Ok(file_gid) = parts[2].parse::<u32>() {
                if file_gid == gid {
                    return Some(parts[0].to_string());
                }
            }
        }
    }
    Some(gid.to_string())
}
