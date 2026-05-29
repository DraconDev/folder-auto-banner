//! Load session command — restore a saved session
//!
//! Prints cd commands to restore saved workspace state

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub name: String,
    pub cwd: PathBuf,
    pub timestamp: String,
    pub git_branch: Option<String>,
    pub description: Option<String>,
}

pub fn run_load_session(name: &str) -> Result<()> {
    // Get sessions directory
    let proj_dirs = directories::ProjectDirs::from("com", "cfm", "cfm")
        .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
    let sessions_dir = proj_dirs.data_dir().join("sessions");

    let session_file = sessions_dir.join(format!("{}.json", sanitize_filename(name)));

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

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
