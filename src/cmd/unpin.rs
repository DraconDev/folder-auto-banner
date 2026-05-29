//! Unpin command — remove a pin

use anyhow::Result;

use crate::state::PinsState;

pub fn run_unpin(name: &str) -> Result<()> {
    if name.is_empty() {
        println!("❌ Pin name required");
        return Ok(());
    }

    // Load pins
    let mut state = PinsState::load().unwrap_or_default();

    // Find and remove pin
    let idx = state.pins.iter().position(|p| p.name == name);

    match idx {
        Some(i) => {
            let pin = state.pins.remove(i);
            state.save()?;
            println!("📌 Unpinned: {} (was: {})", name, pin.path.display());
        }
        None => {
            println!(
                "❌ Pin '{}' not found. Use 'fm pins' to see available pins.",
                name
            );
        }
    }

    Ok(())
}
