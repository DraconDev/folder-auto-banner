//! Trash command — move to system trash
use anyhow::Result;
use std::path::PathBuf;

pub fn run_trash(paths: &[PathBuf], force: bool, dry_run: bool) -> Result<()> {
    println!("🗑️  Trash: {} file(s) (force={}, dry_run={})", paths.len(), force, dry_run);
    Ok(())
}
