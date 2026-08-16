//! `f install` — set up shell wrappers for cd support.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Run the install command: write shell wrappers and add source lines to rc files.
pub fn run_install() -> Result<()> {
    let bin_dir = get_bin_dir()?;
    fs::create_dir_all(&bin_dir).context("Failed to create ~/.local/bin")?;

    // Write shell wrappers
    let zsh_path = bin_dir.join("fab-shell.zsh");
    let bash_path = bin_dir.join("fab-shell.bash");

    fs::write(&zsh_path, crate::shell_wrapper::ZSH_WRAPPER)
        .context("Failed to write fab-shell.zsh")?;
    fs::write(&bash_path, crate::shell_wrapper::BASH_WRAPPER)
        .context("Failed to write fab-shell.bash")?;

    println!("✅ Wrote shell wrappers to {}", bin_dir.display());

    // Add source lines to rc files
    let home = dirs_home()?;

    // zsh
    let zshrc = home.join(".zshrc");
    let zsh_relevant = ensure_source_line(&zshrc, &bin_dir, "fab-shell.zsh")?;

    // bash
    let bashrc = home.join(".bashrc");
    let bash_relevant = ensure_source_line(&bashrc, &bin_dir, "fab-shell.bash")?;

    // Always print the reload hint when at least one rc file was processed.
    // This is critical for users whose current shell was started BEFORE the
    // install — the source line in the rc file only affects NEW shells, so
    // the user must paste the source command into their current terminal to
    // activate folder navigation (`f N` → `cd`).
    if zsh_relevant || bash_relevant {
        print_reload_hint(&bin_dir, zsh_relevant, bash_relevant);
    } else {
        println!("\nℹ️  No ~/.zshrc or ~/.bashrc found. To use the shell function, add a source line to your shell config manually.");
    }

    Ok(())
}

/// Run the uninstall command: remove the source lines added by
/// `f install` from the rc files and delete the fab-shell wrappers.
/// Idempotent — safe to run even if nothing was installed.
pub fn run_uninstall() -> Result<()> {
    let home = dirs_home()?;
    let mut removed_any = false;

    // Strip our marker line and any fab-shell source line (quoted or
    // unquoted, zsh or bash) from both rc files.
    for name in [".zshrc", ".bashrc"] {
        let rc_path = home.join(name);
        if !rc_path.exists() {
            continue;
        }
        let content = fs::read_to_string(&rc_path)?;
        let kept: Vec<&str> = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                let is_marker = trimmed == "# f shell function (for cd support)";
                let is_source = (trimmed.starts_with("source ") || trimmed.starts_with("source '"))
                    && trimmed.contains("fab-shell.zsh")
                    || (trimmed.starts_with("source ") || trimmed.starts_with("source '"))
                        && trimmed.contains("fab-shell.bash");
                !is_marker && !is_source
            })
            .collect();
        if kept.len() != content.lines().count() {
            fs::write(&rc_path, format!("{}\n", kept.join("\n")))??;
            println!("🗑  Removed fab shell function lines from {}", rc_path.display());
            removed_any = true;
        }
    }

    // Remove the wrapper files.
    let bin_dir = get_bin_dir()?;
    for wrapper in ["fab-shell.zsh", "fab-shell.bash"] {
        let path = bin_dir.join(wrapper);
        if path.exists() {
            fs::remove_file(&path)?;
            println!("🗑  Removed {}", path.display());
            removed_any = true;
        }
    }

    if !removed_any {
        println!("ℹ️  Nothing to remove — fab shell function was not installed.");
    }

    Ok(())
}

/// Ensure `~/.zshrc` (or `~/.bashrc`) contains a `source .../fab-shell.{zsh,bash}`
/// line. Returns `true` if the rc file exists and was processed (whether or
/// not we added a new line).
fn ensure_source_line(rc_path: &Path, bin_dir: &Path, wrapper_name: &str) -> Result<bool> {
    if !rc_path.exists() {
        return Ok(false);
    }

    let plain = format!("source {}/{}", bin_dir.display(), wrapper_name);
    // Quote the path so a $HOME containing spaces doesn't break the source line.
    let source_line = format!("source '{}/{}'", bin_dir.display(), wrapper_name);
    if !file_contains(rc_path, &plain)? && !file_contains(rc_path, &source_line)? {
        fs::write(
            rc_path,
            format!(
                "{}\n# f shell function (for cd support)\n{}\n",
                fs::read_to_string(rc_path)?,
                source_line
            ),
        )
        .with_context(|| format!("Failed to append to {}", rc_path.display()))?;
        println!("✅ Added source line to {}", rc_path.display());
    } else {
        println!("ℹ️  {} already has source line", rc_path.display());
    }
    Ok(true)
}

/// Print a copy-pasteable hint telling the user how to activate the shell
/// function in their CURRENT terminal. The source line in the rc file only
/// affects new shells.
fn print_reload_hint(bin_dir: &Path, include_zsh: bool, include_bash: bool) {
    println!();
    println!("🔄 Activate the shell function in your CURRENT terminal:");
    if include_zsh {
        println!("    source {}/fab-shell.zsh", bin_dir.display());
    }
    if include_bash {
        println!("    source {}/fab-shell.bash", bin_dir.display());
    }
    println!();
    println!("(The source line in your rc file only affects new shells.)");
}

/// Get the ~/.local/bin directory.
fn get_bin_dir() -> Result<PathBuf> {
    let home = dirs_home()?;
    Ok(home.join(".local").join("bin"))
}

/// Get home directory.
fn dirs_home() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME env var not set")?;
    Ok(PathBuf::from(home))
}

/// Check if a file contains a given line.
fn file_contains(path: &Path, needle: &str) -> Result<bool> {
    let content = fs::read_to_string(path).context(format!("Failed to read {}", path.display()))?;
    Ok(content.lines().any(|l| l.trim() == needle.trim()))
}
