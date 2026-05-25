//! Paste command — paste files from clipboard
//! 
//! Copies files from clipboard to current directory

use anyhow::Result;
use std::path::PathBuf;
use std::fs;

use crate::state::{ClipboardState, ClipboardEntry};

pub fn run_paste(move_files: bool, overwrite: bool) -> Result<()> {
    // Load clipboard
    let mut state = ClipboardState::load().unwrap_or_default();
    
    if state.entries.is_empty() {
        println!("📋 Clipboard is empty. Use 'fm yank <files>' first.");
        return Ok(());
    }

    // Get current entry
    let entry = match state.entries.get(state.current_index) {
        Some(e) => e,
        None => {
            println!("📋 Clipboard is empty");
            return Ok(());
        }
    };

    let cwd = std::env::current_dir()?;
    let mut copied = 0;
    let mut skipped = 0;

    for src_path in &entry.paths {
        let file_name = match src_path.file_name() {
            Some(n) => n,
            None => {
                eprintln!("⚠️  Could not get filename: {}", src_path.display());
                continue;
            }
        };
        
        let dest = cwd.join(file_name);
        
        // Check if destination exists
        if dest.exists() && !overwrite {
            println!("⚠️  Skipping (exists): {}", file_name.to_string_lossy());
            skipped += 1;
            continue;
        }

        // Copy or move
        let action = if move_files { "Moving" } else { "Copying" };
        
        match if move_files {
            fs::rename(src_path, &dest)
        } else {
            fs::copy(src_path, &dest).map(|_| ())
        } {
            Ok(_) => {
                println!("{}: {}", action, file_name.to_string_lossy());
                copied += 1;
            }
            Err(e) => {
                // If rename fails (cross-device), fall back to copy then delete
                if move_files && e.kind() == std::io::ErrorKind::CrossesDevices {
                    match fs::copy(src_path, &dest) {
                        Ok(_) => {
                            if let Err(de) = fs::remove_file(src_path) {
                                eprintln!("⚠️  Copied but failed to remove original: {}", de);
                            } else {
                                println!("{} (cross-device): {}", action, file_name.to_string_lossy());
                                copied += 1;
                            }
                        }
                        Err(ce) => {
                            eprintln!("❌ {} failed: {} -> {}", action, file_name.to_string_lossy(), ce);
                        }
                    }
                } else {
                    eprintln!("❌ {} failed: {} -> {}", action, file_name.to_string_lossy(), e);
                }
            }
        }
    }

    println!("\n✅ {} file(s) {}, {} skipped", copied, if move_files { "moved" } else { "copied" }, skipped);

    // If move, clear the clipboard after successful move
    if move_files && copied > 0 {
        // Remove the entry we just moved
        state.entries.remove(state.current_index);
        if state.current_index >= state.entries.len() && !state.entries.is_empty() {
            state.current_index = state.entries.len() - 1;
        }
        state.save()?;
    }

    Ok(())
}