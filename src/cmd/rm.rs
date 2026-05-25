//! Remove command — safe file removal with options
//! 
//! Usage: fm rm [options] <files>...

use anyhow::Result;
use std::path::{PathBuf, Path};
use std::fs;

pub fn run_rm(
    paths: &[PathBuf],
    recursive: bool,
    force: bool,
    verbose: bool,
) -> Result<()> {
    if paths.is_empty() {
        println!("❌ No files specified");
        return Ok(());
    }

    let mut removed = 0;
    let mut skipped = 0;

    for path in paths {
        if !path.exists() {
            if !force {
                eprintln!("❌ Not found: {}", path.display());
            }
            skipped += 1;
            continue;
        }

        // Check if it's a directory without -r flag
        if path.is_dir() && !recursive {
            eprintln!("❌ Is a directory (use -r): {}", path.display());
            skipped += 1;
            continue;
        }

        // Check if it's a protected path
        if is_protected_path(path) {
            eprintln!("❌ Refusing to remove protected path: {}", path.display());
            skipped += 1;
            continue;
        }

        match if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        } {
            Ok(_) => {
                if verbose {
                    println!("✓ Removed: {}", path.display());
                }
                removed += 1;
            }
            Err(e) => {
                eprintln!("❌ Failed to remove {}: {}", path.display(), e);
                skipped += 1;
            }
        }
    }

    // Summary
    println!();
    if removed > 0 {
        print!("✅ Removed {} file(s)", removed);
        if skipped > 0 {
            print!(", {} skipped", skipped);
        }
        println!();
    } else if skipped > 0 {
        println!("⚠️  {} file(s) skipped", skipped);
    } else {
        println!("📋 Nothing to do");
    }

    Ok(())
}

/// Check if path is protected (home, root, etc.)
fn is_protected_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    
    // Check for dangerous paths
    let dangerous = [
        "/",
        "/home",
        "/root",
        "/usr",
        "/bin",
        "/sbin",
        "/etc",
        "/var",
        "/sys",
        "/proc",
        "/dev",
    ];
    
    for d in &dangerous {
        if path_str == d || path_str.starts_with(&format!("{}/", d)) {
            return true;
        }
    }
    
    false
}