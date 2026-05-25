//! Trash command — move files to trash instead of deleting
//! 
//! Moves files to ~/.local/share/cfm/trash/ with metadata for recovery
//! 
//! Usage: fm trash [options] <files>...

use anyhow::Result;
use std::path::{PathBuf, Path};
use std::fs;
use serde::{Serialize, Deserialize};

const TRASH_DIR: &str = ".local/share/cfm/trash";
const TRASH_MANIFEST: &str = ".local/share/cfm/trash/manifest.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct TrashManifest {
    pub items: Vec<TrashItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrashItem {
    pub id: String,
    pub original_path: String,
    pub trash_path: String,
    pub deleted_at: String,
    pub original_size: u64,
}

pub fn run_trash(paths: &[PathBuf], verbose: bool) -> Result<()> {
    if paths.is_empty() {
        println!("❌ No files specified");
        return Ok(());
    }

    // Get trash directory
    let trash_base = get_trash_base()?;
    fs::create_dir_all(&trash_base)?;

    // Load manifest
    let mut manifest = load_manifest(&trash_base)?;

    let mut trashed = 0;
    let mut skipped = 0;

    for path in paths {
        if !path.exists() {
            eprintln!("⚠️  Not found: {}", path.display());
            skipped += 1;
            continue;
        }

        match trash_file(path, &trash_base, &mut manifest, verbose) {
            Ok(_) => trashed += 1,
            Err(e) => {
                eprintln!("❌ Failed to trash {}: {}", path.display(), e);
                skipped += 1;
            }
        }
    }

    // Save manifest
    save_manifest(&trash_base, &manifest)?;

    // Summary
    println!();
    if trashed > 0 {
        print!("🗑️  Moved {} file(s) to trash", trashed);
        if skipped > 0 {
            print!(", {} skipped", skipped);
        }
        println!();
        println!("💡 Use 'fm restore-trash' to recover files");
    } else if skipped > 0 {
        println!("⚠️  {} file(s) skipped", skipped);
    } else {
        println!("📋 Nothing to do");
    }

    Ok(())
}

fn get_trash_base() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("com", "cfm", "cfm")
        .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
    let data_dir = proj_dirs.data_dir();
    Ok(data_dir.join("trash"))
}

fn load_manifest(trash_base: &Path) -> Result<TrashManifest> {
    let manifest_path = trash_base.join("manifest.json");
    if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path)?;
        let manifest: TrashManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    } else {
        Ok(TrashManifest { items: Vec::new() })
    }
}

fn save_manifest(trash_base: &Path, manifest: &TrashManifest) -> Result<()> {
    let manifest_path = trash_base.join("manifest.json");
    let content = serde_json::to_string_pretty(manifest)?;
    fs::write(manifest_path, content)?;
    Ok(())
}

fn trash_file(path: &Path, trash_base: &Path, manifest: &mut TrashManifest, verbose: bool) -> Result<()> {
    let original_path = path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    let original_size = if path.is_file() {
        fs::metadata(path)?.len()
    } else {
        calculate_dir_size(path)
    };

    // Generate unique ID and trash path
    let id = generate_id();
    let trash_path = trash_base.join(&id);
    
    // Move file to trash
    let result: Result<(), anyhow::Error> = if path.is_dir() {
        match fs::rename(path, &trash_path) {
            Ok(_) => Ok(()),
            Err(_) => {
                // If rename fails (cross-device), copy and delete
                copy_dir_recursive(path, &trash_path)?;
                delete_recursive(path)?;
                Ok(())
            }
        }
    } else {
        match fs::rename(path, &trash_path) {
            Ok(_) => Ok(()),
            Err(_) => {
                fs::copy(path, &trash_path)?;
                fs::remove_file(path)?;
                Ok(())
            }
        }
    };
    result?;

    // Add to manifest
    manifest.items.push(TrashItem {
        id: id.clone(),
        original_path: original_path.to_string_lossy().to_string(),
        trash_path: trash_path.to_string_lossy().to_string(),
        deleted_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        original_size,
    });

    if verbose {
        println!("🗑️  Trashed: {} -> {}", path.display(), id);
    }

    Ok(())
}

fn calculate_dir_size(path: &Path) -> u64 {
    fs::read_dir(path)
        .map(|entries| {
            entries.filter_map(|e| e.ok())
                .map(|e| {
                    if e.path().is_dir() {
                        calculate_dir_size(&e.path())
                    } else {
                        e.metadata().map(|m| m.len()).unwrap_or(0)
                    }
                })
                .sum()
        })
        .unwrap_or(0)
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
            delete_recursive(&entry?.path())?;
        }
        fs::remove_dir(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}