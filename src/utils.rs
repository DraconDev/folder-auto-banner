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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello"), "hello");
        assert_eq!(sanitize_filename("hello world"), "hello_world");
        assert_eq!(sanitize_filename("file/name"), "file_name");
        assert_eq!(sanitize_filename("file\\name"), "file_name");
        assert_eq!(sanitize_filename("file:name"), "file_name");
        assert_eq!(sanitize_filename("file\"name"), "file_name");
        assert_eq!(sanitize_filename("hello-world"), "hello-world");
        assert_eq!(sanitize_filename("file_123"), "file_123");
        assert_eq!(sanitize_filename(""), "");
    }

    #[test]
    fn test_generate_unique_name_new_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.txt");
        let result = generate_unique_name(&path);
        assert_eq!(result, path);
    }

    #[test]
    fn test_generate_unique_name_existing_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("existing.txt");
        fs::write(&path, "content").unwrap();
        let result = generate_unique_name(&path);
        assert_eq!(result, tmp.path().join("existing (1).txt"));
    }

    #[test]
    fn test_generate_unique_name_multiple() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("file.txt");
        fs::write(&path, "content").unwrap();
        fs::write(tmp.path().join("file (1).txt"), "content").unwrap();
        let result = generate_unique_name(&path);
        assert_eq!(result, tmp.path().join("file (2).txt"));
    }

    #[test]
    fn test_generate_unique_name_no_extension() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("Makefile");
        fs::write(&path, "content").unwrap();
        let result = generate_unique_name(&path);
        assert_eq!(result, tmp.path().join("Makefile (1)"));
    }

    #[test]
    fn test_copy_dir_recursive() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("file1.txt"), "hello").unwrap();
        fs::write(src.join("sub/file2.txt"), "world").unwrap();

        copy_dir_recursive(&src, &dst).unwrap();

        assert!(dst.join("file1.txt").exists());
        assert!(dst.join("sub/file2.txt").exists());
        assert_eq!(fs::read_to_string(dst.join("file1.txt")).unwrap(), "hello");
        assert_eq!(fs::read_to_string(dst.join("sub/file2.txt")).unwrap(), "world");
    }

    #[test]
    fn test_delete_recursive_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("file.txt");
        fs::write(&path, "content").unwrap();
        assert!(path.exists());
        delete_recursive(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn test_delete_recursive_dir() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("dir");
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("file.txt"), "content").unwrap();
        fs::write(dir.join("sub/file2.txt"), "content").unwrap();
        assert!(dir.exists());
        delete_recursive(&dir).unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn test_copy_dir_recursive_symlink_loop() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir_all(&src).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&src, src.join("loop")).unwrap();
            let result = copy_dir_recursive(&src, &dst);
            assert!(result.is_err(), "Should detect symlink loop");
        }
    }

    #[test]
    fn test_run_with_timeout_success() {
        let tmp = TempDir::new().unwrap();
        let result = run_with_timeout("echo", &["hello"], tmp.path(), Duration::from_secs(5));
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.stdout.contains("hello"));
    }

    #[test]
    fn test_run_with_timeout_timeout() {
        let tmp = TempDir::new().unwrap();
        let result = run_with_timeout("sleep", &["10"], tmp.path(), Duration::from_millis(50));
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.stderr, "timeout");
    }

    #[test]
    fn test_run_with_timeout_stdout() {
        let result = run_with_timeout_stdout("echo", &["test"], Duration::from_secs(5));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test"));
    }

    #[test]
    fn test_skip_dirs_constant() {
        assert!(SKIP_DIRS.contains(&"node_modules"));
        assert!(SKIP_DIRS.contains(&"target"));
        assert!(SKIP_DIRS.contains(&".git"));
        assert!(SKIP_DIRS.contains(&"__pycache__"));
    }

    #[test]
    fn test_binary_exts_constant() {
        assert!(BINARY_EXTS.contains(&"png"));
        assert!(BINARY_EXTS.contains(&"jpg"));
        assert!(BINARY_EXTS.contains(&"mp4"));
        assert!(BINARY_EXTS.contains(&"lock"));
    }
}
