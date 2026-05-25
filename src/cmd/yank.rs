//! Yank command — copy files to clipboard
//! 
//! Stores file paths in ~/.local/share/cfm/clipboard.json

use anyhow::Result;
use std::path::PathBuf;

use crate::state::{ClipboardState, ClipboardEntry};

pub fn run_yank(paths: &[PathBuf]) -> Result<()> {
    if paths.is_empty() {
        println!("📋 No files specified for yank");
        return Ok(());
    }

    // Load existing clipboard or create new
    let mut state = ClipboardState::load().unwrap_or_default();
    
    // Get current directory for relative path resolution
    let cwd = std::env::current_dir()?;
    
    // Resolve and validate all paths
    let mut valid_paths = Vec::new();
    for path in paths {
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            cwd.join(path)
        };
        
        if resolved.exists() {
            valid_paths.push(resolved);
        } else {
            eprintln!("⚠️  Skipping non-existent: {}", path.display());
        }
    }

    if valid_paths.is_empty() {
        println!("📋 No valid files to yank");
        return Ok(());
    }

    // Create new clipboard entry
    let entry = ClipboardEntry {
        paths: valid_paths.clone(),
        source_dir: cwd,
        timestamp: chrono::Utc::now(),
    };

    // Add to clipboard (prepend so newest is first)
    state.entries.insert(0, entry);
    state.current_index = 0;

    // Save clipboard
    state.save()?;

    // Print confirmation
    println!("📋 Yanked {} file(s):", valid_paths.len());
    for path in &valid_paths {
        if let Some(name) = path.file_name() {
            println!("  ✓ {}", name.to_string_lossy());
        } else {
            println!("  ✓ {}", path.display());
        }
    }

    Ok(())
}