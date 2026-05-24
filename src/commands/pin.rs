//! Pin command — bookmark a directory
use anyhow::Result;

pub fn run_pin(name: &str) -> Result<()> {
    let path = std::env::current_dir()?;
    println!("📌 Pinned: {} -> {}", name, path.display());
    println!("💡 State persistence pending.");
    Ok(())
}
