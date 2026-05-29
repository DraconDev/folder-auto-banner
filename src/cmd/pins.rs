//! Pins command — list all pinned directories

use anyhow::Result;

use crate::state::PinsState;

pub fn run_pins() -> Result<()> {
    // Load pins
    let state = PinsState::load().unwrap_or_default();

    if state.pins.is_empty() {
        println!("📌 No pins saved");
        println!("💡 Use 'fm pin <name>' to bookmark the current directory");
        return Ok(());
    }

    println!("📌 Pins ({} total):", state.pins.len());
    println!();

    for pin in &state.pins {
        let created = pin.created_at.format("%Y-%m-%d").to_string();
        let last_accessed = pin
            .last_accessed
            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "never".to_string());

        println!("  📍 {} -> {}", pin.name, pin.path.display());
        println!("     Created: {}, Last used: {}", created, last_accessed);

        if pin.access_count > 1 {
            println!("     Used {} times", pin.access_count);
        }
        println!();
    }

    println!("💡 Use 'fm jump <name>' to cd to a pin, 'fm pin <name>' to add/change a pin");

    Ok(())
}
