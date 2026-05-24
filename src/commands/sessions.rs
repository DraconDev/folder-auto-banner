//! Sessions command — list all sessions
use anyhow::Result;

pub fn run_sessions() -> Result<()> {
    println!("💾 Sessions:");
    println!("💡 No sessions saved yet. Use 'fm save-session <name>'.");
    Ok(())
}
