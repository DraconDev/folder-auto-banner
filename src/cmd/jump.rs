//! Jump command — cd to a pinned directory
//! 
//! Prints cd command for shell to execute

use anyhow::Result;

use crate::state::PinsState;

pub fn run_jump(name: &str, _print_cd: bool) -> Result<()> {
    // Load pins
    let state = PinsState::load().unwrap_or_default();

    // Find pin
    let pin = state.pins.iter().find(|p| p.name == name)
        .ok_or_else(|| anyhow::anyhow!("Pin '{}' not found. Use 'fm pins' to see available pins.", name))?;

    // Print cd command (shell integration)
    println!("cd '{}'", pin.path.display());
    
    Ok(())
}