//! Root command — jump to git repo root
use anyhow::Result;
use std::path::Path;

pub fn run_root(print_cd: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let root = find_git_root(&cwd)?;

    if print_cd {
        println!("cd '{}'", root.display());
    } else {
        println!("📁 {}", root.display());
    }
    Ok(())
}

/// Find the git repo root by walking up directories
fn find_git_root(path: &Path) -> Result<std::path::PathBuf> {
    let mut current = Some(path.to_path_buf());
    while let Some(dir) = current {
        if dir.join(".git").exists() {
            return Ok(dir);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    // Fallback: use git command
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()?;
    if output.status.success() {
        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(std::path::PathBuf::from(root));
    }
    Err(anyhow::anyhow!("Not in a git repository"))
}
