//! Open command — open files with default application
use anyhow::Result;
use std::path::PathBuf;

pub fn run_open(paths: &[PathBuf], dry_run: bool) -> Result<()> {
    println!("🖥️  Open: {} file(s) (dry_run={})", paths.len(), dry_run);
    for path in paths {
        println!("  - {}", path.display());
    }
    Ok(())
}
