//! Move command — move files with collision detection
//!
//! Usage: `fm mv [options] <source> <dest>`
//!        fm mv [options] <source>... <dest_dir>

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum CollisionAction {
    Skip,
    Overwrite,
    Rename,
    Ask,
}

pub fn run_mv(
    sources: &[PathBuf],
    dest: &Path,
    overwrite: bool,
    rename_on_collision: bool,
    verbose: bool,
    dry_run: bool,
) -> Result<()> {
    if sources.is_empty() {
        println!("❌ No source files specified");
        return Ok(());
    }

    if dry_run {
        println!("🔍 Dry run — no files will be moved");
        println!();
    }

    let mut moved = 0;
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
                let new_path = generate_unique_name(&dest_path);
                if dry_run {
                    println!(
                        "  Would move: {} -> {}",
                        source.display(),
                        new_path.display()
                    );
                } else {
                    perform_move(source, &new_path, verbose)?;
                }
                moved += 1;
                continue;
            } else {
                println!("⚠️  Skipping (exists): {}", dest_path.display());
                skipped += 1;
                continue;
            }
        }

        if dry_run {
            println!(
                "  Would move: {} -> {}",
                source.display(),
                dest_path.display()
            );
        } else {
            perform_move(source, &dest_path, verbose)?;
        }
        moved += 1;
    }

    // Summary
    if dry_run {
        println!();
        println!(
            "📋 Would move {} file(s), {} overwrite(s), {} skip(s)",
            moved, overwritten, skipped
        );
    } else {
        print_summary("Moved", moved, skipped, overwritten);
    }

    Ok(())
}

fn perform_move(source: &Path, dest: &Path, verbose: bool) -> Result<()> {
    // Try rename first (fast, atomic)
    match fs::rename(source, dest) {
        Ok(_) => {
            if verbose {
                println!(
                    "✓ {} -> {}",
                    source.file_name().unwrap_or_default().to_string_lossy(),
                    dest.file_name().unwrap_or_default().to_string_lossy()
                );
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            // Cross-device move: copy then delete with verification
            if source.is_dir() {
                copy_dir_recursive(source, dest)?;
                // Verify copy succeeded before deleting source
                verify_dir_copy(source, dest)?;
            } else {
                fs::copy(source, dest)?;
                // Verify file copy
                let src_meta = fs::metadata(source)?;
                let dst_meta = fs::metadata(dest)?;
                if src_meta.len() != dst_meta.len() {
                    fs::remove_file(dest)?;
                    return Err(anyhow::anyhow!(
                        "Copy verification failed: size mismatch ({} vs {})",
                        src_meta.len(),
                        dst_meta.len()
                    ));
                }
            }
            delete_recursive(source)?;
            if verbose {
                println!(
                    "✓ {} -> {} (cross-device)",
                    source.file_name().unwrap_or_default().to_string_lossy(),
                    dest.file_name().unwrap_or_default().to_string_lossy()
                );
            }
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

/// Verify that a directory copy is complete by checking item counts
fn verify_dir_copy(src: &Path, dst: &Path) -> Result<()> {
    let src_count = count_items(src)?;
    let dst_count = count_items(dst)?;
    if src_count != dst_count {
        return Err(anyhow::anyhow!(
            "Copy verification failed: source has {} items, destination has {}",
            src_count,
            dst_count
        ));
    }
    Ok(())
}

fn count_items(path: &Path) -> Result<usize> {
    let mut count = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        count += 1;
        if entry.file_type()?.is_dir() {
            count += count_items(&entry.path())?;
        }
    }
    Ok(count)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn delete_recursive(path: &Path) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            delete_recursive(&entry.path())?;
        }
        fs::remove_dir(path)?;
    } else {
        fs::remove_file(path)?;
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
            Some(ext) => format!(
                "{stem} ({counter}).{ext}",
                stem = stem,
                counter = counter,
                ext = ext
            ),
            None => format!("{} ({})", stem, counter),
        };
        let new_path = parent.join(&new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

fn print_summary(action: &str, moved: usize, skipped: usize, overwritten: usize) {
    println!();
    if moved > 0 {
        print!("✅ {} {} file(s)", action, moved);
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
