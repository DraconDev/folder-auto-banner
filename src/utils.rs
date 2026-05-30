//! Shared utilities — deduplicated functions used across multiple commands

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Duration;

// === File Operations ===

/// Recursively copy a directory (follows symlink guards to prevent loops)
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    let mut visited: std::collections::HashSet<std::path::PathBuf> = std::collections::HashSet::new();
    copy_dir_recursive_inner(src, dst, &mut visited)
}

fn copy_dir_recursive_inner(
    src: &Path,
    dst: &Path,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
) -> Result<()> {
    let canonical = src.canonicalize().unwrap_or_else(|_| src.to_path_buf());
    if !visited.insert(canonical) {
        return Err(anyhow::anyhow!("Symlink loop detected at {}", src.display()));
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive_inner(&src_path, &dst_path, visited)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Recursively delete a directory or file
pub fn delete_recursive(path: &Path) -> Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            delete_recursive(&entry?.path())?;
        }
        std::fs::remove_dir(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Generate a unique filename by appending a counter
pub fn generate_unique_name(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path.extension().map(|e| e.to_string_lossy().to_string());
    let parent = path.parent().unwrap_or(Path::new("."));

    let mut counter = 1;
    loop {
        let new_name = match ext.as_ref() {
            Some(ext) => format!(
                "{stem} ({counter}).{ext}",
                stem = stem,
                counter = counter,
                ext = ext
            ),
            None => format!("{} ({})", stem, counter),
        };
        let new_path = parent.join(&new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

/// Sanitize a string for use as a filename
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

// === Output ===

/// Print a summary of file operation results
pub fn print_summary(action: &str, moved: usize, skipped: usize, overwritten: usize) {
    println!();
    if moved > 0 {
        print!("✅ {} {} file(s)", action, moved);
        if overwritten > 0 {
            print!(", {} overwritten", overwritten);
        }
        if skipped > 0 {
            print!(", {} skipped", skipped);
        }
        println!();
    } else if skipped > 0 {
        println!("⚠️  {} skipped", skipped);
    } else {
        println!("📋 Nothing to do");
    }
}

// === Process Execution ===

/// Output from a command run with timeout
pub struct CommandOutput {
    pub status: std::process::ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

/// Run a command with a timeout
pub fn run_with_timeout(
    cmd: &str,
    args: &[&str],
    cwd: &Path,
    timeout: Duration,
) -> Result<CommandOutput> {
    let mut command = std::process::Command::new(cmd);
    command.args(args);
    command.current_dir(cwd);
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());

    let start = std::time::Instant::now();
    let mut child = command.spawn()?;

    loop {
        if let Some(_status) = child.try_wait()? {
            let output = child.wait_with_output()?;
            return Ok(CommandOutput {
                status: output.status,
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(CommandOutput {
                status: std::process::ExitStatus::default(),
                stdout: String::new(),
                stderr: "timeout".to_string(),
            });
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Run a command with a timeout, returning just stdout as a String
pub fn run_with_timeout_stdout(
    cmd: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<String> {
    let mut command = std::process::Command::new(cmd);
    command.args(args);
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::null());

    let start = std::time::Instant::now();
    let mut child = command.spawn()?;

    loop {
        if let Some(_output) = child.try_wait()? {
            let result = child.wait_with_output()?;
            return Ok(String::from_utf8_lossy(&result.stdout).to_string());
        }

        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(String::new());
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}

// === Constants ===

/// Directories to skip when scanning (shared between todo_scanner and code_metrics)
pub const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "dist",
    "build",
    "vendor",
    ".next",
    "__pycache__",
    ".venv",
    "venv",
];

/// Binary file extensions to skip (shared between todo_scanner and code_metrics)
pub const BINARY_EXTS: &[&str] = &[
    "exe", "bin", "o", "so", "dll", "dylib", "a", "lib", "obj", "pdb", "png", "jpg", "jpeg",
    "gif", "webp", "ico", "svg", "bmp", "tiff", "mp3", "mp4", "avi", "mkv", "mov", "webm",
    "flac", "wav", "ogg", "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "tgz", "woff", "woff2",
    "ttf", "eot", "pdf", "doc", "docx", "xls", "xlsx", "sqlite", "sqlite3", "db",
    "lock", // Cargo.lock, package-lock.json etc.
];
