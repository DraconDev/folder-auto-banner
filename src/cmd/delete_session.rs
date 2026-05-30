//! Delete session command — remove a saved session

use anyhow::Result;
use std::fs;

use crate::state::Session;

pub fn run_delete_session(name: &str) -> Result<()> {
    if name.is_empty() {
        println!("❌ Session name required");
        return Ok(());
    }

    let sessions_dir = Session::sessions_dir()?;
    let session_file = sessions_dir.join(format!("{}.json", crate::utils::sanitize_filename(name)));

    if !session_file.exists() {
        println!("❌ Session '{}' not found", name);
        println!("💡 Use 'fm sessions' to see available sessions");
        return Ok(());
    }

    fs::remove_file(&session_file)?;
    println!("🗑️  Deleted session: {}", name);

    Ok(())
}
