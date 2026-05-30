//! Load session command — restore a saved session
//!
//! Prints cd commands to restore saved workspace state

use anyhow::Result;
use std::fs;

use crate::state::Session;

pub fn run_load_session(name: &str) -> Result<()> {
    let sessions_dir = Session::sessions_dir()?;
    let session_file = sessions_dir.join(format!("{}.json", crate::utils::sanitize_filename(name)));

    if !session_file.exists() {
        println!("❌ Session '{}' not found", name);
        println!("💡 Use 'fm sessions' to see available sessions");
        return Ok(());
    }

    let content = fs::read_to_string(&session_file)?;
    let session: Session = serde_json::from_str(&content)?;

    // Check if directory exists
    if !session.cwd.exists() {
        println!(
            "⚠️  Session directory no longer exists: {}",
            session.cwd.display()
        );
        println!("   Saving session would update the path");
    }

    // Print restore commands
    println!("# To restore session '{}', run:", name);
    println!("cd '{}'", session.cwd.display());
    println!();
    println!("# Or use this shell function:");
    println!("# cds() {{ cd '{}'; }}", session.cwd.display());

    Ok(())
}
