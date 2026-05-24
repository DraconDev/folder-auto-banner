//! Root command — jump to git repo root
use anyhow::Result;

pub fn run_root(print_cd: bool) -> Result<()> {
    if print_cd {
        println!("cd $(git rev-parse --show-toplevel)");
    } else {
        println!("⬆️  Jump to repo root (not yet implemented)");
    }
    Ok(())
}
