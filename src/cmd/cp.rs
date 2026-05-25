//! Copy command — copy files with collision detection
//! 
//! Usage: fm cp [options] <source> <dest>
//!        fm cp [options] <source>... <dest_dir>

use anyhow::Result;
use std::path::{PathBuf, Path};
use std::fs;
use std::io;

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
) -> Result<()> {
    if sources.is_empty() {
        println!("❌ No source files specified");
        return Ok(());
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
            let file_name = source.file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid source: {}", source.display()))?;
            dest.join(file_name)
        } else {
            dest.to_path_buf()
        };

        // Check for collision
        if dest_path.exists() {
            if overwrite {
                // Remove destination first
                if dest_path.is_dir() {
                    fs::remove_dir_all(&dest_path)?;
                } else {
                    fs::remove_file(&dest_path)?;
                }
                overwritten += 1;
            } else if rename_on_collision {
                // Generate new name
                let new_path = generate_unique_name(&dest_path);
                perform_copy(source, &new_path, preserve, verbose)?;
                copied += 1;
                continue;
            } else {
                println!("⚠️  Skipping (exists): {}", dest_path.display());
                skipped += 1;
                continue;
            }
        }

        perform_copy(source, &dest_path, preserve, verbose)?;
        copied += 1;
    }

    // Summary
    print_summary("Copied", copied, skipped, overwritten);

    Ok(())
}

fn perform_copy(source: &Path, dest: &Path, preserve: bool, verbose: bool) -> Result<()> {
    if source.is_dir() {
        copy_dir_recursive(source, dest, preserve)?;
    } else {
        if preserve {
            // Preserve metadata
            let meta = fs::metadata(source)?;
            fs::copy(source, dest)?;
            // Note: Can't easily preserve all metadata in std
        } else {
            fs::copy(source, dest)?;
        }
    }

    if verbose {
        println!("✓ {} -> {}", 
            source.file_name().unwrap_or_default().to_string_lossy(),
            dest.file_name().unwrap_or_default().to_string_lossy());
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path, preserve: bool) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path, preserve)?;
        } else {
            if preserve {
                fs::copy(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    }
    Ok(())
}

fn generate_unique_name(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path.extension().map(|e| e.to_string_lossy().to_string());
    let parent = path.parent().unwrap_or(Path::new("."));

    let mut counter = 1;
    loop {
        let new_name = match ext.as_ref() {
            Some(ext) => format!("{stem} ({counter}).{ext}", stem = stem, counter = counter, ext = ext),
            None => format!("{} ({})", stem, counter),
        };
        let new_path = parent.join(&new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

fn print_summary(action: &str, copied: usize, skipped: usize, overwritten: usize) {
    println!();
    if copied > 0 {
        print!("✅ {} {} file(s)", action, copied);
        if overwritten > 0 {
            print!(", {} overwritten", overwritten);
        }
        if skipped > 0 {
            print!(", {} skipped", skipped);
        }
        println!();
    } else if skipped > 0 {
        println!("⚠️  {} skipped", skipped);
    } else {
        println!("📋 Nothing to do");
    }
}