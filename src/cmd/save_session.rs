//! Save session command — save current workspace state
//!
//! Saves: cwd, git branch, timestamp to session file

use anyhow::Result;
use std::fs;

use crate::state::Session;

pub fn run_save_session(name: &str, description: Option<&str>) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Get git branch if in a repo
    let git_info = crate::git::get_git_info(&cwd).ok();
    let git_branch = git_info.as_ref().and_then(|i| i.branch.clone());

    let mut session = Session::new(name, cwd);
    session.git_branch = git_branch;
    session.description = description.map(|s| s.to_string());

    // Get sessions directory
    let sessions_dir = Session::sessions_dir()?;
    fs::create_dir_all(&sessions_dir)?;

    // Save session
    let session_file = session.file_path()?;
    let content = serde_json::to_string_pretty(&session)?;
    fs::write(&session_file, content)?;

    println!("💾 Saved session: {}", name);
    println!("   Directory: {}", session.cwd.display());
    if let Some(ref branch) = session.git_branch {
        println!("   Branch: {}", branch);
    }
    if let Some(ref desc) = session.description {
        println!("   Note: {}", desc);
    }

    Ok(())
}
