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
    let mut installed_any = false;

    // zsh
    let zshrc = home.join(".zshrc");
    if zshrc.exists() {
        let source_line = format!("source {}/fab-shell.zsh", bin_dir.display());
        if !file_contains(&zshrc, &source_line)? {
            fs::write(
                &zshrc,
                format!(
                    "{}\n# f shell function (for cd support)\n{}\n",
                    fs::read_to_string(&zshrc)?,
                    source_line
                ),
            )
            .context("Failed to append to .zshrc")?;
            println!("✅ Added source line to ~/.zshrc");
            installed_any = true;
        } else {
            println!("ℹ️  ~/.zshrc already has source line");
        }
    }

    // bash
    let bashrc = home.join(".bashrc");
    if bashrc.exists() {
        let source_line = format!("source {}/fab-shell.bash", bin_dir.display());
        if !file_contains(&bashrc, &source_line)? {
            fs::write(
                &bashrc,
                format!(
                    "{}\n# f shell function (for cd support)\n{}\n",
                    fs::read_to_string(&bashrc)?,
                    source_line
                ),
            )
            .context("Failed to append to .bashrc")?;
            println!("✅ Added source line to ~/.bashrc");
            installed_any = true;
        } else {
            println!("ℹ️  ~/.bashrc already has source line");
        }
    }

    if installed_any {
        println!("\n🔄 Run `source ~/.zshrc` or open a new terminal for changes to take effect.");
    } else {
        println!("\n✅ Shell wrappers already installed. No changes needed.");
    }

    Ok(())
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
    let content = fs::read_to_string(path).context(format!(
        "Failed to read {}",
        path.display()
    ))?;
    Ok(content.lines().any(|l| l.trim() == needle.trim()))
}
