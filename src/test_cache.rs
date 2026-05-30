//! Test results caching — stores last test run results
//!
//! When `cargo test` is run, save results to cache.
//! Banner reads from cache to show test status.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Cached test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub duration_ms: u64,
    pub timestamp: i64,
}

impl TestResults {
    /// Get cache file path
    fn cache_path() -> Option<PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("com", "cfm", "cfm")?;
        Some(proj_dirs.cache_dir().join("test_results.json"))
    }

    /// Load cached test results
    pub fn load() -> Option<Self> {
        let path = Self::cache_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let results: TestResults = serde_json::from_str(&content).ok()?;
        
        // Check if cache is expired (older than 1 hour)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        if now - results.timestamp > 3600 {
            return None; // Expired
        }
        
        Some(results)
    }

    /// Save test results to cache
    #[allow(dead_code)]
pub fn save(passed: usize, failed: usize, ignored: usize, duration_ms: u64) {
        let results = TestResults {
            passed,
            failed,
            ignored,
            duration_ms,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        };
        
        if let Some(path) = Self::cache_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(content) = serde_json::to_string_pretty(&results) {
                let _ = std::fs::write(&path, content);
            }
        }
    }

    /// Format duration as human-readable string
    #[allow(dead_code)]
pub fn format_duration(&self) -> String {
        if self.duration_ms < 1000 {
            format!("{}ms", self.duration_ms)
        } else {
            format!("{:.1}s", self.duration_ms as f64 / 1000.0)
        }
    }

    /// Format time ago as human-readable string
    pub fn format_time_ago(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let diff = now - self.timestamp;
        
        if diff < 60 {
            "just now".to_string()
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else {
            format!("{}d ago", diff / 86400)
        }
    }
}

use std::fs;
