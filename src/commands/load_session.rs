//! Load session command
use anyhow::Result;

pub fn run_load_session(name: &str, print_cd: bool) -> Result<()> {
    if print_cd {
        println!("cd /some/session/path");
    } else {
        println!("📂 Loading session: {} (not yet implemented)", name);
    }
    Ok(())
}
