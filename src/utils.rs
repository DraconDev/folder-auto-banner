//! Shared utilities — deduplicated functions used across multiple commands

use anyhow::Result;
use std::path::Path;
use std::time::Duration;

// === File Operations ===

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
pub fn run_with_timeout_stdout(cmd: &str, args: &[&str], timeout: Duration) -> Result<String> {
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
    "exe", "bin", "o", "so", "dll", "dylib", "a", "lib", "obj", "pdb", "png", "jpg", "jpeg", "gif",
    "webp", "ico", "svg", "bmp", "tiff", "mp3", "mp4", "avi", "mkv", "mov", "webm", "flac", "wav",
    "ogg", "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "tgz", "woff", "woff2", "ttf", "eot",
    "pdf", "doc", "docx", "xls", "xlsx", "sqlite", "sqlite3", "db",
    "lock", // Cargo.lock, package-lock.json etc.
];

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
