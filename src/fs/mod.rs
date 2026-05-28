//! Filesystem utilities — directory metadata, file types, project detection
//! 
//! Fast, parallel directory walking using ignore crate.

use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

/// Project type detection
#[derive(Debug, Clone, PartialEq)]
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
    /// Detect project type from directory contents
    pub fn detect(path: &Path) -> Self {
        let dir = match std::fs::read_dir(path) {
            Ok(d) => d,
            Err(_) => return ProjectType::Generic,
        };

        for entry in dir.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_lowercase();

            match name_str.as_str() {
                "cargo.toml" => return ProjectType::Rust,
                "package.json" => return ProjectType::Node,
                "pyproject.toml" | "setup.py" | "requirements.txt" | "Pipfile" => return ProjectType::Python,
                "go.mod" => return ProjectType::Go,
                "gemfile" => return ProjectType::Ruby,
                "pom.xml" | "build.gradle" => return ProjectType::Java,
                "CMakeLists.txt" => return ProjectType::CMake,
                _ => {}
            }
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct DirSummary {
    pub total_items: usize,
    pub total_size: u64,
    pub files: usize,
    pub dirs: usize,
    pub top_items: Vec<DirEntry>,
    pub project_type: ProjectType,
    pub last_modified: Option<DateTime<Utc>>,
}

impl DirSummary {
    /// Scan a directory and gather metadata
    pub fn scan(path: &Path) -> Result<Self> {
        let project_type = ProjectType::detect(path);
        let mut total_size: u64 = 0;
        let mut files = 0;
        let mut dirs = 0;
        let mut top_items = Vec::new();
        let mut last_modified: Option<DateTime<Utc>> = None;

        let walker = WalkBuilder::new(path)
            .max_depth(Some(1))  // Only immediate directory
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

            // Try to get metadata — for symlinks, follow the link
            let metadata = entry.metadata().ok().or_else(|| {
                if is_symlink {
                    std::fs::metadata(entry.path()).ok()
                } else {
                    None
                }
            });

            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            total_size += size;

            // Get modified time
            let modified = metadata.as_ref()
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
                    let perms_str = if read_only { "r--r--r--".to_string() } else { "rw-rw-rw-".to_string() };
                    (perms_str, false, "?".to_string(), "?".to_string())
                }
            } else {
                ("----------".to_string(), false, "?".to_string(), "?".to_string())
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
        top_items.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        Ok(DirSummary {
            total_items: files + dirs,
            total_size,
            files,
            dirs,
            top_items,
            project_type,
            last_modified,
        })
    }

    /// Get top N items
    pub fn top_items(&self, n: usize) -> &[DirEntry] {
        &self.top_items[..n.min(self.top_items.len())]
    }

    /// Get remaining count
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
        let after_dot = &s[dot+1..];
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

/// Format exact date/time — ISO-style: "2024-05-27"
pub fn format_exact_time(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d").to_string()
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
    pub fn by_type(&self) -> Vec<(String, usize)> {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for entry in &self.top_items {
            let ext = if let Some(dot) = entry.name.rfind('.') {
                entry.name[dot+1..].to_lowercase()
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
