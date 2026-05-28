//! Port usage detection — finds listening ports for a project
//!
//! Uses `ss` to get listening ports with PIDs, then checks if those processes
//! have their working directory in the project folder via /proc/<pid>/cwd.
//! Timeout: 500ms, Cache: 10 seconds

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const PORT_TIMEOUT: Duration = Duration::from_millis(500);

/// Port scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub ports: Vec<u16>,
}

/// Detect listening ports for a project by checking process working directories
pub fn detect_ports(path: &Path) -> Result<PortInfo> {
    let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    // Try `ss -tlnp` to get listening ports with PIDs
    if let Ok(ports) = try_ss_with_cwd_check(&abs_path) {
        return Ok(PortInfo { ports });
    }

    // Fall back to `lsof -i -P -n +D <path>` (slower, but works on macOS)
    if let Ok(ports) = try_lsof_with_dir(&abs_path) {
        return Ok(PortInfo { ports });
    }

    Ok(PortInfo { ports: Vec::new() })
}

/// Use `ss -tlnp` to get listening ports, then check if PID's cwd matches project dir
fn try_ss_with_cwd_check(project_path: &Path) -> Result<Vec<u16>> {
    let output = run_with_timeout("ss", &["-tlnp"], PORT_TIMEOUT)?;

    let mut ports = Vec::new();

    for line in output.lines() {
        // Skip header line
        if line.starts_with("State") || line.starts_with("Recv-Q") {
            continue;
        }

        // Parse ss output: State Recv-Q Send-Q Local Address:Port Peer Address:Port Process
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        // Extract port from local address (e.g., "0.0.0.0:3000" or "[::]:8080")
        let addr = parts[3];
        let port_str = addr.rsplit(':').next().unwrap_or("");
        let port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        // Extract PID from the last field (e.g., "users:(\"node\",pid=12345,fd=10)")
        let process_info = parts.last().unwrap_or(&"");
        if let Some(pid) = extract_pid(process_info) {
            // Check if this process's cwd is in the project directory
            if pid_cwd_matches(pid, project_path) {
                if !ports.contains(&port) {
                    ports.push(port);
                }
            }
        }
    }

    ports.sort();
    Ok(ports)
}

/// Extract PID from ss process info string like `users:(\"node\",pid=12345,fd=10)`
fn extract_pid(info: &str) -> Option<u32> {
    let pid_start = info.find("pid=")?;
    let pid_str = &info[pid_start + 4..];
    let pid_end = pid_str.find(|c: char| !c.is_ascii_digit())?;
    pid_str[..pid_end].parse().ok()
}

/// Check if a process's working directory matches or is inside the project directory
fn pid_cwd_matches(pid: u32, project_path: &Path) -> bool {
    let cwd_link = format!("/proc/{}/cwd", pid);
    match std::fs::read_link(&cwd_link) {
        Ok(cwd) => {
            let cwd_abs = cwd.canonicalize().unwrap_or(cwd);
            cwd_abs == project_path || cwd_abs.starts_with(project_path)
        }
        Err(_) => false,
    }
}

/// Fallback: use `lsof +D <path>` to find processes with cwd in project dir
fn try_lsof_with_dir(project_path: &Path) -> Result<Vec<u16>> {
    let path_str = project_path.to_string_lossy().to_string();

    // lsof +D finds processes with cwd or open files in the directory
    let output = run_with_timeout(
        "lsof",
        &["-i", "-P", "-n", "-F", "pcftn", "+D", &path_str],
        PORT_TIMEOUT,
    )?;

    let mut ports = Vec::new();
    let mut _current_pid: Option<u32> = None;
    let mut _current_name: Option<String> = None;

    for line in output.lines() {
        let field = &line[..1.min(line.len())];
        let value = &line[1.min(line.len())..];

        match field {
            "p" => _current_pid = value.parse().ok(),
            "c" => _current_name = Some(value.to_string()),
            "n" => {
                // Network connection: "*:3000" or "127.0.0.1:8080"
                if let Some(port_str) = value.rsplit(':').next() {
                    if let Ok(port) = port_str.parse::<u16>() {
                        if !ports.contains(&port) {
                            ports.push(port);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    ports.sort();
    Ok(ports)
}

fn run_with_timeout(cmd: &str, args: &[&str], timeout: Duration) -> Result<String> {
    let mut command = Command::new(cmd);
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
