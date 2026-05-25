//! Open command — open files with default application
//! 
//! Cross-platform: xdg-open on Linux, open on macOS, start on Windows

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

pub fn run_open(paths: &[PathBuf], verbose: bool) -> Result<()> {
    if paths.is_empty() {
        println!("❌ No files specified");
        return Ok(());
    }

    let mut opened = 0;
    let mut failed = 0;

    for path in paths {
        if !path.exists() {
            eprintln!("❌ Not found: {}", path.display());
            failed += 1;
            continue;
        }

        match open_path(path) {
            Ok(_) => {
                if verbose {
                    println!("✓ Opened: {}", path.display());
                }
                opened += 1;
            }
            Err(e) => {
                eprintln!("❌ Failed to open {}: {}", path.display(), e);
                failed += 1;
            }
        }
    }

    if opened > 0 {
        println!("✅ Opened {} file(s)", opened);
    }
    if failed > 0 {
        println!("⚠️  {} failed", failed);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn open_path(path: &PathBuf) -> Result<()> {
    // Try xdg-open first, then fallback to sensible-browser or other common openers
    let openers = ["xdg-open", "gio open", "gnome-open", "kde-open"];
    
    for opener in &openers {
        let parts: Vec<&str> = opener.split_whitespace().collect();
        let mut cmd = Command::new(parts[0]);
        for arg in &parts[1..] {
            cmd.arg(arg);
        }
        cmd.arg(path);
        
        match cmd.spawn() {
            Ok(_) => return Ok(()),
            Err(_) => continue,
        }
    }
    
    Err(anyhow::anyhow!("No opener found. Install xdg-utils (xdg-open)"))
}

#[cfg(target_os = "macos")]
fn open_path(path: &PathBuf) -> Result<()> {
    Command::new("open")
        .arg(path)
        .spawn()?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn open_path(path: &PathBuf) -> Result<()> {
    Command::new("start")
        .arg("")
        .arg(path)
        .spawn()?;
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn open_path(path: &PathBuf) -> Result<()> {
    Err(anyhow::anyhow!("Unsupported platform for 'open' command"))
}