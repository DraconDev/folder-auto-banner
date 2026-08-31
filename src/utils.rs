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
    use std::io::Read;

    let mut command = std::process::Command::new(cmd);
    command.args(args);
    command.current_dir(cwd);
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());

    let start = std::time::Instant::now();
    let mut child = command.spawn()?;

    let mut stdout_pipe = child.stdout.take().unwrap();
    let mut stderr_pipe = child.stderr.take().unwrap();

    let stdout_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buf);
        buf
    });

    let stderr_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buf);
        buf
    });

    loop {
        if let Some(status) = child.try_wait()? {
            let stdout_bytes = stdout_handle.join().unwrap_or_default();
            let stderr_bytes = stderr_handle.join().unwrap_or_default();
            return Ok(CommandOutput {
                status,
                stdout: String::from_utf8_lossy(&stdout_bytes).to_string(),
                stderr: String::from_utf8_lossy(&stderr_bytes).to_string(),
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
    use std::io::Read;

    let mut command = std::process::Command::new(cmd);
    command.args(args);
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::null());

    let start = std::time::Instant::now();
    let mut child = command.spawn()?;

    let mut stdout_pipe = child.stdout.take().unwrap();
    let stdout_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buf);
        buf
    });

    loop {
        if let Some(_status) = child.try_wait()? {
            let stdout_bytes = stdout_handle.join().unwrap_or_default();
            return Ok(String::from_utf8_lossy(&stdout_bytes).to_string());
        }

        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(String::new());
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Return whether an environment flag is explicitly enabled with `1`.
pub fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| value == "1")
}

/// Resolve a feature flag for daemon scans. The config is the default, the
/// positive environment variable enables the feature, and the `FAB_NO_*`
/// variable always wins as an explicit disable.
pub fn feature_enabled(config_enabled: bool, enable_var: &str, disable_var: &str) -> bool {
    if env_flag(disable_var) {
        false
    } else if env_flag(enable_var) {
        true
    } else {
        config_enabled
    }
}

/// Resolve an optional direct-scan feature. Direct scans keep expensive probes
/// opt-in for interactive latency, while still honoring config disables and
/// the documented positive/negative environment overrides.
pub fn direct_feature_enabled(config_enabled: bool, enable_var: &str, disable_var: &str) -> bool {
    if env_flag(disable_var) {
        false
    } else {
        config_enabled && env_flag(enable_var)
    }
}

// === Constants ===

/// Directories to skip when scanning (shared between todo_scanner and code_metrics)
pub const SKIP_DIRS: &[&str] = &[
    // Package managers / build artifacts
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
    // SvelteKit / Vite scratch
    ".svelte-kit",
    ".vite",
    // Agent / AI tool directories (large, not source code)
    ".pi",
    ".pi-glla",
    ".opencode",
    ".claude",
    ".cursor",
    ".roo",
    ".windsurf",
    ".augment",
    ".amazonq",
    ".kiro",
    ".trae",
    ".deepseek",
    ".gemini",
    ".qwen",
    ".dracon",
    // Caches
    ".cache",
    ".trash",
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
        // Package managers / build artifacts
        assert!(SKIP_DIRS.contains(&"node_modules"));
        assert!(SKIP_DIRS.contains(&"target"));
        assert!(SKIP_DIRS.contains(&".git"));
        assert!(SKIP_DIRS.contains(&"__pycache__"));
        assert!(SKIP_DIRS.contains(&".svelte-kit"));
        // Agent / AI tool directories
        assert!(SKIP_DIRS.contains(&".pi"));
        assert!(SKIP_DIRS.contains(&".opencode"));
        assert!(SKIP_DIRS.contains(&".claude"));
        assert!(SKIP_DIRS.contains(&".cursor"));
        assert!(SKIP_DIRS.contains(&".dracon"));
        // Caches
        assert!(SKIP_DIRS.contains(&".cache"));
    }

    #[test]
    fn test_binary_exts_constant() {
        assert!(BINARY_EXTS.contains(&"png"));
        assert!(BINARY_EXTS.contains(&"jpg"));
        assert!(BINARY_EXTS.contains(&"mp4"));
        assert!(BINARY_EXTS.contains(&"lock"));
    }

    #[test]
    fn test_feature_flag_defaults_to_config() {
        assert!(feature_enabled(true, "FAB_TEST_MISSING_ENABLE", "FAB_TEST_MISSING_DISABLE"));
        assert!(!feature_enabled(false, "FAB_TEST_MISSING_ENABLE", "FAB_TEST_MISSING_DISABLE"));
        assert!(!direct_feature_enabled(true, "FAB_TEST_MISSING_ENABLE", "FAB_TEST_MISSING_DISABLE"));
    }
}
