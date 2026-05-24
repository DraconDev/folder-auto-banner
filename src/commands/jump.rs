//! Jump command — jump to a pinned directory
use anyhow::Result;

pub fn run_jump(name: &str, print_cd: bool) -> Result<()> {
    if print_cd {
        println!("cd /some/pinned/path");
    } else {
        println!("⬆️  Jump to: {} (not yet implemented)", name);
    }
    Ok(())
}
