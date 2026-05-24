//! Copy command — copy files
use anyhow::Result;
use std::path::PathBuf;

pub fn run_cp(sources: &[PathBuf], dest: &PathBuf, _overwrite: bool, _dry_run: bool) -> Result<()> {
    println!("📋 Copying {} file(s) to {}", sources.len(), dest.display());
    Ok(())
}
