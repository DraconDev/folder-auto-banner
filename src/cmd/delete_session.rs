//! Delete session command — remove a saved session

use anyhow::Result;
use std::fs;

use crate::utils;

pub fn run_delete_session(name: &str) -> Result<()> {
    if name.is_empty() {
        println!("❌ Session name required");
        return Ok(());
    }

    // Get sessions directory
    let proj_dirs = directories::ProjectDirs::from("com", "cfm", "cfm")
        .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
    let sessions_dir = proj_dirs.data_dir().join("sessions");

    let session_file = sessions_dir.join(format!("{}.json", utils::sanitize_filename(name)));

    if !session_file.exists() {
        println!("❌ Session '{}' not found", name);
        println!("💡 Use 'fm sessions' to see available sessions");
        return Ok(());
    }

    fs::remove_file(&session_file)?;
    println!("🗑️  Deleted session: {}", name);

    Ok(())
}
