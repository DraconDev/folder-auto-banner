use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::fs::{DirSummary, DirEntry};
use crate::git::GitInfo;

// IPC Protocol
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Get cached banner data for a directory
    Banner { path: PathBuf },
    /// Get directory size (recursive)
    DirSize { path: PathBuf },
    /// Ping (health check)
    Ping,
    /// Shutdown daemon
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Banner(BannerData),
    DirSize { path: PathBuf, size: u64 },
    Pong,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannerData {
    pub path: PathBuf,
    pub summary: DirSummary,
    pub git_info: Option<GitInfo>,
    pub dir_sizes: HashMap<PathBuf, u64>,
    pub cached_at: DateTime<Utc>,
}
