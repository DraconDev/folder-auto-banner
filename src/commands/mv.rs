//! Move command — move files with split context dashboard
use anyhow::Result;
use std::path::PathBuf;

pub fn run_mv(
    sources: &[PathBuf],
    dest: &PathBuf,
    _overwrite: bool,
    _rename: bool,
    _skip: bool,
    _dry_run: bool,
) -> Result<()> {
    println!("📦 Moving {} file(s) to {}", sources.len(), dest.display());
    for src in sources {
        println!("  - {}", src.display());
    }
    println!("\n💡 Note: Split context dashboard pending.");
    Ok(())
}
