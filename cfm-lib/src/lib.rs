//! cfm-lib — shared library for Contextual File Manager
//!
//! Contains the core modules shared between the `fm` CLI and `cfmd` daemon binaries.

pub mod build_status;
pub mod cache;
pub mod code_metrics;
pub mod daemon_types;
pub mod docker;
pub mod fs;
pub mod git;
pub mod icon;
pub mod port_usage;
pub mod state;
pub mod todo_scanner;
pub mod utils;
