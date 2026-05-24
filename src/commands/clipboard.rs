//! Clipboard command — show/clear clipboard
use anyhow::Result;

pub fn run_clipboard(clear: bool) -> Result<()> {
    if clear {
        println!("🧹 Clipboard cleared");
    } else {
        println!("📋 Clipboard: empty (not yet implemented)");
    }
    Ok(())
}
