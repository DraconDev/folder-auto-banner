//! Stats command — deep directory analysis
//! 
//! Analyzes directory and provides:
//! - Total size
//! - File count by type
//! - Directory depth
//! - Largest files
//! - Language breakdown (for code projects)

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use crate::fs::format_size;

pub fn run_stats(path: Option<&Path>, json: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let path = path.unwrap_or(cwd.as_path());

    let mut stats = DirStats::default();
    analyze_dir(path, &mut stats, 0)?;

    if json {
        output_json(&stats, path);
    } else {
        output_rich(&stats, path);
    }

    Ok(())
}

#[derive(Default)]
struct DirStats {
    total_size: u64,
    total_files: usize,
    total_dirs: usize,
    max_depth: usize,
    by_extension: HashMap<String, FileCount>,
    largest_files: Vec<(PathBuf, u64)>,
    hidden_files: usize,
    binary_files: usize,
}

#[derive(Default)]
struct FileCount {
    count: usize,
    size: u64,
}

fn analyze_dir(path: &Path, stats: &mut DirStats, depth: usize) -> Result<()> {
    if depth > 100 {
        return Ok(()); // Safety limit
    }
    
    stats.max_depth = stats.max_depth.max(depth);

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("⚠️  Cannot read {}: {}", path.display(), e);
            return Ok(());
        }
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        let entry_path = entry.path();

        // Track hidden files
        if name_str.starts_with('.') {
            stats.hidden_files += 1;
        }

        if entry_path.is_dir() {
            stats.total_dirs += 1;
            analyze_dir(&entry_path, stats, depth + 1)?;
        } else if entry_path.is_file() {
            stats.total_files += 1;
            
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            
            // Check for binary first (needs the path)
            if is_likely_binary(&entry_path) {
                stats.binary_files += 1;
            }
            
            stats.total_size += size;

            // Track by extension
            let ext = get_extension(&name_str);
            let entry = stats.by_extension.entry(ext).or_default();
            entry.count += 1;
            entry.size += size;

            // Track largest files (after binary check)
            stats.largest_files.push((entry_path.clone(), size));
            stats.largest_files.sort_by(|a, b| b.1.cmp(&a.1));
            stats.largest_files.truncate(10);

            // Track binary separately (already counted above)
            // (removed duplicate tracking)
    }

    Ok(())
}

fn get_extension(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((_, ext)) => ext.to_lowercase(),
        None => "(no ext)".to_string(),
    }
}

fn is_likely_binary(path: &Path) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        use std::io::Read;
        let mut buffer = [0u8; 8192];
        if let Ok(bytes) = file.read(&mut buffer) {
            return buffer[..bytes].contains(&0);
        }
    }
    false
}

fn output_rich(stats: &DirStats, path: &Path) {
    println!("📊 Statistics: {}", path.display());
    println!("{}", "─".repeat(60));
    
    // Overview
    println!();
    println!("📁 OVERVIEW");
    println!("  Total size:    {}", format_size(stats.total_size));
    println!("  Files:         {}", stats.total_files);
    println!("  Directories:  {}", stats.total_dirs);
    println!("  Max depth:    {} levels", stats.max_depth);
    if stats.hidden_files > 0 {
        println!("  Hidden:       {}", stats.hidden_files);
    }
    
    // File types
    println!();
    println!("📋 FILE TYPES (by count)");
    let mut types: Vec<_> = stats.by_extension.iter().collect();
    types.sort_by(|a, b| b.1.count.cmp(&a.1.count));
    
    for (ext, fc) in types.iter().take(10) {
        let bar = make_bar(fc.count, stats.total_files, 20);
        println!("  {:15} {:>5} {:>8} {}", ext, fc.count, format_size(fc.size), bar);
    }
    
    // Largest files
    if !stats.largest_files.is_empty() {
        println!();
        println!("📦 LARGEST FILES");
        for (path, size) in &stats.largest_files {
            if let Some(name) = path.file_name() {
                println!("  {:>10}  {}", format_size(*size), name.to_string_lossy());
            }
        }
    }
    
    // Code breakdown (if project detected)
    let code_exts = ["rs", "ts", "js", "py", "go", "java", "c", "cpp", "h", "hpp", "rb", "rs"];
    let code_count: usize = code_exts.iter()
        .filter_map(|ext| stats.by_extension.get(*ext))
        .map(|fc| fc.count)
        .sum();
    
    if code_count > 0 {
        println!();
        println!("💻 CODE FILES: {} ({}%)", 
            code_count, 
            (code_count as f64 / stats.total_files as f64 * 100.0) as usize);
    }
    
    println!();
}

fn make_bar(count: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return "░".repeat(width);
    }
    let filled = (count as f64 / total as f64 * width as f64) as usize;
    let filled = filled.min(width);
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

fn output_json(stats: &DirStats, path: &Path) {
    println!("{{");
    println!("  \"path\": \"{}\",", path.display());
    println!("  \"total_size\": {},", stats.total_size);
    println!("  \"total_files\": {},", stats.total_files);
    println!("  \"total_dirs\": {},", stats.total_dirs);
    println!("  \"max_depth\": {},", stats.max_depth);
    println!("  \"hidden_files\": {},", stats.hidden_files);
    
    println!("  \"by_extension\": {{");
    let mut types: Vec<_> = stats.by_extension.iter().collect();
    types.sort_by(|a, b| b.1.count.cmp(&a.1.count));
    
    for (i, (ext, fc)) in types.iter().enumerate() {
        let comma = if i < types.len() - 1 { "," } else { "" };
        println!("    \"{}\": {{ \"count\": {}, \"size\": {} }}{}", ext, fc.count, fc.size, comma);
    }
    println!("  }}");
    
    println!("}}");
}