//! Diff command — compare two directories
use anyhow::Result;
use std::path::Path;

pub fn run_diff(dir1: &Path, dir2: &Path, shallow: bool, json: bool) -> Result<()> {
    if json {
        println!("{{\"dir1\":\"{}\",\"dir2\":\"{}\",\"pending\":true}}", dir1.display(), dir2.display());
    } else {
        println!("🔍 Comparing {} vs {} (shallow={})", dir1.display(), dir2.display(), shallow);
        println!("💡 Directory comparison pending implementation.");
    }
    Ok(())
}
