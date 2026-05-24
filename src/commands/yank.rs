//! Yank command — copy files to clipboard
use anyhow::Result;
use std::path::PathBuf;

pub fn run_yank(paths: &[PathBuf]) -> Result<()> {
    println!("📋 Yank: {} file(s)", paths.len());
    for path in paths {
        println!("  - {}", path.display());
    }
    println!("\n💡 Note: Full implementation pending. Clipboard state not yet saved.");
    Ok(())
}
