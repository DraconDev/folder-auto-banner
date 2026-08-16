//! TODO/FIXME counter — data types only.
//!
//! The scanning logic lives in `project_insights::scan_insights`, which
//! computes TODO counts, LOC and file counts in a single bounded walk.
//! The standalone `scan_todos` duplicate was removed so the two copies
//! cannot drift (see audit finding).

use serde::{Deserialize, Serialize};

/// TODO scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoInfo {
    pub count: usize,
    pub by_pattern: Vec<(String, usize)>,
}
