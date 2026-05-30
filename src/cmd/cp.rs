//! Copy command — copy files with collision detection
//!
//! Usage: fm cp [options] <source> <dest>
//!        fm cp [options] <source>... <dest_dir>

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils;

#[derive(Debug, Clone)]
pub enum CollisionAction {
    Skip,
    Overwrite,
    Rename,
    Ask,
}

pub fn run_cp(
    sources: &[PathBuf],
    dest: &Path,
    overwrite: bool,
    rename_on_collision: bool,
    verbose: bool,
    preserve: bool,
    dry_run: bool,
) -> Result<()> {
    if sources.is_empty() {
        println!("❌ No source files specified");
        return Ok(());
    }

    if dry_run {
        println!("🔍 Dry run — no files will be copied");
        println!();
    }

    let mut copied = 0;
    let mut skipped = 0;
    let mut overwritten = 0;

    // Determine if dest is a directory
    let dest_is_dir = dest.is_dir() || dest.to_string_lossy().ends_with('/');

    for source in sources {
        if !source.exists() {
            eprintln!("⚠️  Source not found: {}", source.display());
            skipped += 1;
            continue;
        }

        let dest_path = if dest_is_dir {
            let file_name = source
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid source: {}", source.display()))?;
            dest.join(file_name)
        } else {
            dest.to_path_buf()
        };

        // Check for collision
        if dest_path.exists() {
            if overwrite {
                if dry_run {
                    println!(
                        "  Would overwrite: {} -> {}",
                        source.display(),
                        dest_path.display()
                    );
                } else if dest_path.is_dir() {
                    fs::remove_dir_all(&dest_path)?;
                } else {
                    fs::remove_file(&dest_path)?;
                }
                overwritten += 1;
            } else if rename_on_collision {
                // Generate new name
                let new_path = utils::generate_unique_name(&dest_path);
                if dry_run {
                    println!(
                        "  Would copy: {} -> {}",
                        source.display(),
                        new_path.display()
                    );
                } else {
                    perform_copy(source, &new_path, preserve, verbose)?;
                }
                copied += 1;
                continue;
            } else {
                println!("⚠️  Skipping (exists): {}", dest_path.display());
                skipped += 1;
                continue;
            }
        }

        if dry_run {
            println!(
                "  Would copy: {} -> {}",
                source.display(),
                dest_path.display()
            );
        } else {
            perform_copy(source, &dest_path, preserve, verbose)?;
        }
        copied += 1;
    }

    // Summary
    if dry_run {
        println!();
        println!(
            "📋 Would copy {} file(s), {} overwrite(s), {} skip(s)",
            copied, overwritten, skipped
        );
    } else {
        utils::print_summary("Copied", copied, skipped, overwritten);
    }

    Ok(())
}

fn perform_copy(source: &Path, dest: &Path, preserve: bool, verbose: bool) -> Result<()> {
    if source.is_dir() {
        utils::copy_dir_recursive(source, dest)?;
    } else {
        if preserve {
            // Preserve metadata
            let _meta = fs::metadata(source)?;
            fs::copy(source, dest)?;
            // Note: Can't easily preserve all metadata in std
        } else {
            fs::copy(source, dest)?;
        }
    }

    if verbose {
        println!(
            "✓ {} -> {}",
            source.file_name().unwrap_or_default().to_string_lossy(),
            dest.file_name().unwrap_or_default().to_string_lossy()
        );
    }

    Ok(())
}


