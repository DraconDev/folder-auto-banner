//! Code metrics — data types only.
//!
//! The scanning logic lives in `project_insights::scan_insights`, which
//! computes TODO counts, LOC and file counts in a single bounded walk.
//! The standalone `scan_metrics` duplicate was removed so the two copies
//! cannot drift (see audit finding).

use serde::{Deserialize, Serialize};

/// Code metrics result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub total_loc: usize,
    pub by_extension: Vec<(String, usize)>,
    pub file_count: usize,
}
