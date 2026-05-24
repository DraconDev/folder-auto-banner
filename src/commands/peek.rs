//! Peek command — syntax-highlighted file preview
use anyhow::Result;
use std::path::Path;

pub fn run_peek(file: &Path, lines: usize) -> Result<()> {
    println!("👁️  Peek: {} ({} lines)", file.display(), lines);
    // Read file content
    if let Ok(content) = std::fs::read_to_string(file) {
        for (i, line) in content.lines().take(lines).enumerate() {
            println!("{:4}│ {}", i + 1, line);
        }
    } else {
        println!("Could not read file");
    }
    Ok(())
}
