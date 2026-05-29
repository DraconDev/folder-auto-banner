//! Sessions command — list all saved sessions

use anyhow::Result;
use std::fs;

#[derive(Debug, serde::Deserialize)]
struct Session {
    name: String,
    cwd: std::path::PathBuf,
    timestamp: String,
    git_branch: Option<String>,
    description: Option<String>,
}

pub fn run_sessions() -> Result<()> {
    // Get sessions directory
    let proj_dirs = directories::ProjectDirs::from("com", "cfm", "cfm")
        .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
    let sessions_dir = proj_dirs.data_dir().join("sessions");

    if !sessions_dir.exists() {
        println!("📋 No sessions saved");
        println!("💡 Use 'fm save-session <name>' to save the current directory");
        return Ok(());
    }

    // Read all session files
    let mut sessions: Vec<Session> = Vec::new();

    for entry in fs::read_dir(&sessions_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(session) = serde_json::from_str::<Session>(&content) {
                    sessions.push(session);
                }
            }
        }
    }

    if sessions.is_empty() {
        println!("📋 No sessions saved");
        println!("💡 Use 'fm save-session <name>' to save the current directory");
        return Ok(());
    }

    println!("📋 Sessions ({} total):", sessions.len());
    println!();

    // Sort by timestamp (newest first)
    sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    for session in &sessions {
        let exists = if session.cwd.exists() { "✓" } else { "✗" };
        let branch = session
            .git_branch
            .as_ref()
            .map(|b| format!(" [{}]", b))
            .unwrap_or_default();

        println!("  📁 {} {}{}", exists, session.name, branch);
        println!("     Path: {}", session.cwd.display());
        println!("     Saved: {}", session.timestamp);

        if let Some(ref desc) = session.description {
            println!("     Note: {}", desc);
        }
        println!();
    }

    println!("💡 Use 'fm load-session <name>' to see restore commands");
    println!("💡 Use 'fm delete-session <name>' to remove a session");

    Ok(())
}
