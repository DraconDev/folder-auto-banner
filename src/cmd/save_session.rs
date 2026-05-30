//! Save session command — save current workspace state
//!
//! Saves: cwd, git branch, timestamp to session file

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::utils;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub name: String,
    pub cwd: PathBuf,
    pub timestamp: String,
    pub git_branch: Option<String>,
    pub description: Option<String>,
}

pub fn run_save_session(name: &str, description: Option<&str>) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Get git branch if in a repo
    let git_info = crate::git::get_git_info(&cwd).ok();
    let git_branch = git_info.as_ref().and_then(|i| i.branch.clone());

    let session = Session {
        name: name.to_string(),
        cwd: cwd.clone(),
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        git_branch,
        description: description.map(|s| s.to_string()),
    };

    // Get sessions directory
    let proj_dirs = directories::ProjectDirs::from("com", "cfm", "cfm")
        .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
    let sessions_dir = proj_dirs.data_dir().join("sessions");
    fs::create_dir_all(&sessions_dir)?;

    // Save session
    let session_file = sessions_dir.join(format!("{}.json", utils::sanitize_filename(name)));
    let content = serde_json::to_string_pretty(&session)?;
    fs::write(&session_file, content)?;

    println!("💾 Saved session: {}", name);
    println!("   Directory: {}", cwd.display());
    if let Some(ref branch) = session.git_branch {
        println!("   Branch: {}", branch);
    }
    if let Some(ref desc) = session.description {
        println!("   Note: {}", desc);
    }

    Ok(())
}
