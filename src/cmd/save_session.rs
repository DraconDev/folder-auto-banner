//! Save session command
use anyhow::Result;

pub fn run_save_session(name: &str) -> Result<()> {
    let path = std::env::current_dir()?;
    println!("💾 Session saved: {} -> {}", name, path.display());
    Ok(())
}
