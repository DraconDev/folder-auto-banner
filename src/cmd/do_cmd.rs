//! Do command — act on piped file paths
use anyhow::Result;

pub fn run_do(action: Option<&str>, dry_run: bool) -> Result<()> {
    println!("🔧 Do: action={:?}, dry_run={}", action, dry_run);
    println!("💡 Reads paths from stdin and takes action based on extension.");
    Ok(())
}
