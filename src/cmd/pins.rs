//! Pins command — list all pins
use anyhow::Result;

pub fn run_pins() -> Result<()> {
    println!("📌 Pins:");
    println!("💡 No pins yet. Use 'fm pin <name>' to create one.");
    Ok(())
}
