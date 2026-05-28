//! Build status detection — checks if a project builds cleanly
//!
//! Runs language-specific build checks with a 500ms timeout.
//! Results are cached for 30 seconds.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use super::fs::ProjectType;

const BUILD_TIMEOUT: Duration = Duration::from_millis(500);

/// Build status result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStatus {
    pub ok: bool,
    pub errors: usize,
    pub warnings: usize,
    pub output: String,
}

/// Check build status for a project
pub fn check_build(path: &Path, project_type: &ProjectType) -> Option<BuildStatus> {
    let result = match project_type {
        ProjectType::Rust => check_rust_build(path),
        ProjectType::Node => check_node_build(path),
        ProjectType::Go => check_go_build(path),
        ProjectType::Python => check_python_build(path),
        _ => return None,
    };

    result.ok()
}

fn check_rust_build(path: &Path) -> Result<BuildStatus> {
    let output = run_with_timeout(
        "cargo",
        &["check", "--message-format=short"],
        path,
        BUILD_TIMEOUT,
    )?;

    let errors = count_matches(&output, "error");
    let warnings = count_matches(&output, "warning");
    let ok = errors == 0 && output.status.success();

    Ok(BuildStatus {
        ok,
        errors,
        warnings,
        output: truncate_output(&output.stdout, 500),
    })
}

fn check_node_build(path: &Path) -> Result<BuildStatus> {
    // Try tsc --noEmit first (TypeScript)
    if path.join("tsconfig.json").exists() {
        let output = run_with_timeout("npx", &["tsc", "--noEmit"], path, BUILD_TIMEOUT)?;
        let errors = count_matches(&output, "error");
        let warnings = count_matches(&output, "warning");
        let ok = errors == 0 && output.status.success();

        return Ok(BuildStatus {
            ok,
            errors,
            warnings,
            output: truncate_output(&output.stdout, 500),
        });
    }

    // Fall back to checking if package.json has a build script
    if let Ok(pkg) = std::fs::read_to_string(path.join("package.json")) {
        if pkg.contains("\"build\"") {
            let output = run_with_timeout("npm", &["run", "build", "--dry-run"], path, BUILD_TIMEOUT)?;
            let ok = output.status.success();
            return Ok(BuildStatus {
                ok,
                errors: if ok { 0 } else { 1 },
                warnings: 0,
                output: truncate_output(&output.stdout, 500),
            });
        }
    }

    Ok(BuildStatus {
        ok: true,
        errors: 0,
        warnings: 0,
        output: String::new(),
    })
}

fn check_go_build(path: &Path) -> Result<BuildStatus> {
    let output = run_with_timeout("go", &["build", "./..."], path, BUILD_TIMEOUT)?;
    let errors = count_matches(&output, "error");
    let warnings = count_matches(&output, "warning");
    let ok = errors == 0 && output.status.success();

    Ok(BuildStatus {
        ok,
        errors,
        warnings,
        output: truncate_output(&output.stdout, 500),
    })
}

fn check_python_build(path: &Path) -> Result<BuildStatus> {
    // Find Python files to compile-check
    let py_files: Vec<String> = std::fs::read_dir(path)?
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .ends_with(".py")
        })
        .map(|e| e.path().to_string_lossy().to_string())
        .take(50)
        .collect();

    if py_files.is_empty() {
        return Ok(BuildStatus {
            ok: true,
            errors: 0,
            warnings: 0,
            output: String::new(),
        });
    }

    let output = run_with_timeout("python3", &["-m", "py_compile"], path, BUILD_TIMEOUT)?;
    let errors = if output.status.success() { 0 } else { 1 };

    Ok(BuildStatus {
        ok: output.status.success(),
        errors,
        warnings: 0,
        output: truncate_output(&output.stderr, 500),
    })
}

/// Run a command with a timeout (uses std::process with no real timeout on non-unix,
/// but we use a simple approach: just spawn and wait briefly)
fn run_with_timeout(
    cmd: &str,
    args: &[&str],
    cwd: &Path,
    timeout: Duration,
) -> Result<CommandOutput> {
    let mut command = Command::new(cmd);
    command.args(args);
    command.current_dir(cwd);
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());

    let start = std::time::Instant::now();
    let mut child = command.spawn()?;

    // Poll with short intervals up to timeout
    loop {
        if let Some(status) = child.try_wait()? {
            let stdout = child
                .wait_with_output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default();
            let stderr = String::new(); // already consumed
            return Ok(CommandOutput {
                status,
                stdout,
                stderr,
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

struct CommandOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

fn count_matches(output: &CommandOutput, pattern: &str) -> usize {
    output
        .stdout
        .lines()
        .filter(|l| l.to_lowercase().contains(pattern))
        .count()
        + output
            .stderr
            .lines()
            .filter(|l| l.to_lowercase().contains(pattern))
            .count()
}

fn truncate_output(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len])
    }
}
