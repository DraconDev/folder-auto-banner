use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::fs::DirSummary;
use crate::git::GitInfo;

// IPC Protocol
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Get cached banner data for a directory
    Banner { path: PathBuf },
    /// Pre-compute banner data for a directory (fire-and-forget)
    Warm { path: PathBuf },
    /// Get directory size (recursive)
    DirSize { path: PathBuf },
    /// Ping (health check)
    Ping,
    /// Shutdown daemon
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Banner(Box<BannerData>),
    DirSize { path: PathBuf, size: u64 },
    Pong,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannerData {
    #[serde(skip_serializing, default)]
    #[allow(dead_code)]
    pub path: PathBuf,
    pub summary: DirSummary,
    pub git_info: Option<GitInfo>,
    #[serde(skip_serializing, default)]
    #[allow(dead_code)]
    pub dir_sizes: HashMap<PathBuf, u64>,
    #[serde(skip_serializing, default)]
    #[allow(dead_code)]
    pub cached_at: DateTime<Utc>,
}
