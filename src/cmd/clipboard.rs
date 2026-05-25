//! Clipboard command — show/clear clipboard
//! 
//! Displays current clipboard contents or clears it

use anyhow::Result;

use crate::state::ClipboardState;

pub fn run_clipboard(clear: bool) -> Result<()> {
    if clear {
        // Clear clipboard
        let state = ClipboardState::default();
        state.save()?;
        println!("🧹 Clipboard cleared");
        return Ok(());
    }

    // Show clipboard contents
    let state = ClipboardState::load().unwrap_or_default();
    
    if state.entries.is_empty() {
        println!("📋 Clipboard is empty");
        println!("💡 Use 'fm yank <files>' to copy files to clipboard");
        return Ok(());
    }

    println!("📋 Clipboard ({} entries):", state.entries.len());
    println!();

    for (i, entry) in state.entries.iter().enumerate() {
        let current_marker = if i == state.current_index { "▶" } else { " " };
        let time_str = entry.timestamp.format("%Y-%m-%d %H:%M").to_string();
        
        println!("{} Entry {} — {} file(s) — from {}", 
                 current_marker, i + 1, entry.paths.len(), time_str);
        
        for path in &entry.paths {
            println!("   • {}", path.display());
        }
        println!();
    }

    println!("💡 Use 'fm yank <files>' to add, 'fm paste' to copy, 'fm clipboard --clear' to clear");

    Ok(())
}