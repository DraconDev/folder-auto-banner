//! Remove command — permanent delete with confirmation
use anyhow::Result;
use std::path::PathBuf;

pub fn run_rm(paths: &[PathBuf], force: bool, dry_run: bool) -> Result<()> {
    println!("🗑️  Remove: {} file(s) (force={}, dry_run={})", paths.len(), force, dry_run);
    for path in paths {
        println!("  - {}", path.display());
    }
    Ok(())
}
