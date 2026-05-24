//! Filesystem utilities — directory metadata, file types, project detection
//! 
//! Fast, parallel directory walking using ignore crate.

use anyhow::{Context, Result};
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
    pub size: u64,
    pub modified: Option<DateTime<Utc>>,
}

impl DirEntry {
    pub fn icon(&self) -> &'static str {
        if self.is_dir {
            "📂"
        } else {
            "📄"
        }
    }
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

            let metadata = entry.metadata().ok();
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

            top_items.push(DirEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_path_buf(),
                is_dir,
                is_file,
                is_symlink,
                size,
                modified,
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

/// Human-readable size
pub fn format_size(bytes: u64) -> String {
    use byte_unit::{Byte, UnitType};
    let byte = Byte::from_u64(bytes);
    let adjusted = byte.get_appropriate_unit(UnitType::Binary);
    format!("{}", adjusted)
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