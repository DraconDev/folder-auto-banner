//! Pin command — bookmark a directory
//! 
//! Saves directory paths to ~/.local/share/cfm/pins.json

use anyhow::Result;
use std::path::Path;

use crate::state::{PinsState, Pin};

pub fn run_pin(name: &str) -> Result<()> {
    // Validate name (no spaces, no special chars except - and _)
    if name.is_empty() {
        println!("❌ Pin name cannot be empty");
        return Ok(());
    }
    
    if name.contains(' ') || name.contains('/') || name.contains('\\') {
        println!("❌ Pin name cannot contain spaces, / or \\");
        return Ok(());
    }

    // Get current directory
    let path = std::env::current_dir()?;
    let path_str = path.to_string_lossy().to_string();

    // Load existing pins or create new
    let mut state = PinsState::load().unwrap_or_default();

    // Check if pin already exists
    let existing_idx = state.pins.iter().position(|p| p.name == name);
    
    if let Some(idx) = existing_idx {
        // Update existing pin
        state.pins[idx].path = path.clone();
        state.pins[idx].last_accessed = Some(chrono::Utc::now());
        state.pins[idx].access_count += 1;
        println!("📌 Updated pin: {} -> {}", name, path_str);
    } else {
        // Create new pin
        let pin = Pin {
            name: name.to_string(),
            path,
            created_at: chrono::Utc::now(),
            last_accessed: Some(chrono::Utc::now()),
            access_count: 1,
        };
        state.pins.push(pin);
        println!("📌 Pinned: {} -> {}", name, path_str);
    }

    // Save pins
    state.save()?;

    Ok(())
}