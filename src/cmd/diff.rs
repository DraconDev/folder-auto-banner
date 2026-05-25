//! Diff command — compare two directories
//! 
//! Shows differences between two directories:
//! - Unique files to each
//! - Modified files (different content)
//! - Size differences

use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn run_diff(dir1: &Path, dir2: &Path, shallow: bool, json: bool) -> Result<()> {
    if !dir1.is_dir() {
        println!("❌ Not a directory: {}", dir1.display());
        return Ok(());
    }
    if !dir2.is_dir() {
        println!("❌ Not a directory: {}", dir2.display());
        return Ok(());
    }

    // Scan both directories
    let files1 = scan_dir(dir1, shallow);
    let files2 = scan_dir(dir2, shallow);
    
    let keys1: HashSet<_> = files1.keys().collect();
    let keys2: HashSet<_> = files2.keys().collect();
    
    // Find unique and common files
    let unique_to_1: Vec<_> = keys1.difference(&keys2).copied().collect();
    let unique_to_2: Vec<_> = keys2.difference(&keys1).copied().collect();
    let common: Vec<_> = keys1.intersection(&keys2).copied().collect();

    if json {
        output_json(dir1, dir2, &unique_to_1, &unique_to_2, &common, &files1, &files2);
    } else {
        output_rich(dir1, dir2, &unique_to_1, &unique_to_2, &common, &files1, &files2);
    }

    Ok(())
}

#[derive(Default)]
struct FileInfo {
    size: u64,
    path: String,
}

fn scan_dir(dir: &Path, shallow: bool) -> std::collections::HashMap<String, FileInfo> {
    let mut files = std::collections::HashMap::new();
    scan_dir_recursive(dir, dir, &mut files, 0, shallow);
    files
}

fn scan_dir_recursive(
    base: &Path,
    current: &Path,
    files: &mut std::collections::HashMap<String, FileInfo>,
    depth: usize,
    shallow: bool,
) {
    if depth > 50 || (shallow && depth > 1) {
        return;
    }

    let entries = match fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative = path.strip_prefix(base)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        
        if relative.is_empty() || relative.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            scan_dir_recursive(base, &path, files, depth + 1, shallow);
        } else if path.is_file() {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            files.insert(relative, FileInfo { size, path: path.to_string_lossy().to_string() });
        }
    }
}

fn output_rich(
    dir1: &Path,
    dir2: &Path,
    unique_to_1: &[&str],
    unique_to_2: &[&str],
    common: &[&str],
    files1: &std::collections::HashMap<String, FileInfo>,
    files2: &std::collections::HashMap<String, FileInfo>,
) {
    println!("🔍 Comparing directories");
    println!("  {}  →  {}", dir1.display(), dir2.display());
    println!("{}", "─".repeat(60));

    // Summary
    println!();
    println!("📊 SUMMARY");
    println!("  {} only:     {}", dir1.file_name().unwrap_or_default().to_string_lossy(), unique_to_1.len());
    println!("  {} only:     {}", dir2.file_name().unwrap_or_default().to_string_lossy(), unique_to_2.len());
    println!("  Common:     {}", common.len());

    // Unique to dir1
    if !unique_to_1.is_empty() {
        println!();
        println!("📁 Only in {}:", dir1.file_name().unwrap_or_default().to_string_lossy());
        for file in unique_to_1.iter().take(20) {
            let size = files1.get(*file).map(|f| format_size(f.size)).unwrap_or_default();
            println!("  + {} ({})", file, size);
        }
        if unique_to_1.len() > 20 {
            println!("  ... and {} more", unique_to_1.len() - 20);
        }
    }

    // Unique to dir2
    if !unique_to_2.is_empty() {
        println!();
        println!("📁 Only in {}:", dir2.file_name().unwrap_or_default().to_string_lossy());
        for file in unique_to_2.iter().take(20) {
            let size = files2.get(*file).map(|f| format_size(f.size)).unwrap_or_default();
            println!("  + {} ({})", file, size);
        }
        if unique_to_2.len() > 20 {
            println!("  ... and {} more", unique_to_2.len() - 20);
        }
    }

    // Check common files for size differences
    let size_diff: Vec<_> = common.iter()
        .filter(|f| {
            let s1 = files1.get(*f).map(|i| i.size).unwrap_or(0);
            let s2 = files2.get(*f).map(|i| i.size).unwrap_or(0);
            s1 != s2
        })
        .collect();

    if !size_diff.is_empty() {
        println!();
        println!("📏 Size differences:");
        for file in size_diff.iter().take(10) {
            let s1 = files1.get(*file).map(|i| i.size).unwrap_or(0);
            let s2 = files2.get(*file).map(|i| i.size).unwrap_or(0);
            let diff = s2 as i64 - s1 as i64;
            let sign = if diff > 0 { "+" } else { "" };
            println!("  {} {} → {} ({}{})", 
                file, 
                format_size(s1), 
                format_size(s2),
                sign,
                format_size(diff.unsigned_abs())
            );
        }
    }

    println!();
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn output_json(
    dir1: &Path,
    dir2: &Path,
    unique_to_1: &[&str],
    unique_to_2: &[&str],
    common: &[&str],
    _files1: &std::collections::HashMap<String, FileInfo>,
    _files2: &std::collections::HashMap<String, FileInfo>,
) {
    println!("{{");
    println!("  \"dir1\": \"{}\",", dir1.display());
    println!("  \"dir2\": \"{}\",", dir2.display());
    println!("  \"unique_to_dir1\": {:?},", unique_to_1);
    println!("  \"unique_to_dir2\": {:?},", unique_to_2);
    println!("  \"common_count\": {}", common.len());
    println!("}}");
}