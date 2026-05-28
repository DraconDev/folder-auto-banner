//! Port usage detection — finds listening ports for a project
//!
//! Uses `ss` or `lsof` to find ports associated with the project directory.
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

/// Detect listening ports for a project
pub fn detect_ports(path: &Path) -> Result<PortInfo> {
    let dir_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // Try `ss` first (Linux)
    if let Ok(ports) = try_ss(&dir_name) {
        if !ports.is_empty() {
            return Ok(PortInfo { ports });
        }
    }

    // Fall back to `lsof`
    if let Ok(ports) = try_lsof(&dir_name) {
        return Ok(PortInfo { ports });
    }

    Ok(PortInfo { ports: Vec::new() })
}

fn try_ss(dir_name: &str) -> Result<Vec<u16>> {
    let output = run_with_timeout(
        "ss",
        &["-tlnp"],
        PORT_TIMEOUT,
    )?;

    let mut ports = Vec::new();
    for line in output.lines() {
        if line.contains(dir_name) {
            // Extract port from address like 0.0.0.0:3000 or [::]:8080
            if let Some(addr) = line.split_whitespace().nth(3) {
                if let Some(port_str) = addr.rsplit(':').next() {
                    if let Ok(port) = port_str.parse::<u16>() {
                        if !ports.contains(&port) {
                            ports.push(port);
                        }
                    }
                }
            }
        }
    }

    ports.sort();
    Ok(ports)
}

fn try_lsof(dir_name: &str) -> Result<Vec<u16>> {
    let output = run_with_timeout(
        "lsof",
        &["-i", "-P", "-n"],
        PORT_TIMEOUT,
    )?;

    let mut ports = Vec::new();
    for line in output.lines() {
        if line.contains(dir_name) {
            // Extract port from address like *:3000 or 127.0.0.1:8080
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(addr) = parts.get(8) {
                if let Some(port_str) = addr.rsplit(':').next() {
                    if let Ok(port) = port_str.parse::<u16>() {
                        if !ports.contains(&port) {
                            ports.push(port);
                        }
                    }
                }
            }
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
        if let Some(output) = child.try_wait()? {
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
