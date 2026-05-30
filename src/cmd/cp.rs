//! Copy command — copy files with collision detection
//!
//! Usage: fm cp [options] <source> <dest>
//!        fm cp [options] <source>... <dest_dir>

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CollisionAction {
    Skip,
    Overwrite,
    Rename,
    Ask,
}

/// Options for the copy command
pub struct CpOptions<'a> {
    pub sources: &'a [PathBuf],
    pub dest: &'a Path,
    pub overwrite: bool,
    pub rename_on_collision: bool,
    pub verbose: bool,
    pub preserve: bool,
    pub dry_run: bool,
}

pub fn run_cp(opts: &CpOptions) -> Result<()> {
    if opts.sources.is_empty() {
        println!("❌ No source files specified");
        return Ok(());
    }

    if opts.dry_run {
        println!("🔍 Dry run — no files will be copied");
        println!();
    }

    let mut copied = 0;
    let mut skipped = 0;
    let mut overwritten = 0;

    // Determine if dest is a directory
    let dest_is_dir = opts.dest.is_dir() || opts.dest.to_string_lossy().ends_with('/');

    for source in opts.sources {
        if !source.exists() {
            eprintln!("⚠️  Source not found: {}", source.display());
            skipped += 1;
            continue;
        }

        // Check if source is a protected path
        if crate::utils::is_protected_path(source) {
            eprintln!("❌ Refusing to copy protected path: {}", source.display());
            skipped += 1;
            continue;
        }

        let dest_path = if dest_is_dir {
            let file_name = source
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid source: {}", source.display()))?;
            opts.dest.join(file_name)
        } else {
            opts.dest.to_path_buf()
        };

        // Check for collision
        if dest_path.exists() {
            if opts.overwrite {
                if opts.dry_run {
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
            } else if opts.rename_on_collision {
                // Generate new name
                let new_path = utils::generate_unique_name(&dest_path);
                if opts.dry_run {
                    println!(
                        "  Would copy: {} -> {}",
                        source.display(),
                        new_path.display()
                    );
                } else {
                    perform_copy(source, &new_path, opts.preserve, opts.verbose)?;
                }
                copied += 1;
                continue;
            } else {
                println!("⚠️  Skipping (exists): {}", dest_path.display());
                skipped += 1;
                continue;
            }
        }

        if opts.dry_run {
            println!(
                "  Would copy: {} -> {}",
                source.display(),
                dest_path.display()
            );
        } else {
            perform_copy(source, &dest_path, opts.preserve, opts.verbose)?;
        }
        copied += 1;
    }

    // Summary
    if opts.dry_run {
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


