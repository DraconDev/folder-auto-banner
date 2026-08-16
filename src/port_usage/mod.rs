//! Port usage detection — finds listening ports for a project
//!
//! Uses `ss` to get listening ports with PIDs, then checks if those processes
//! have their working directory in the project folder via `/proc/<pid>/cwd`.
//! Timeout: 500ms, Cache: 10 seconds

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::utils;

const PORT_TIMEOUT: Duration = Duration::from_millis(500);

// Cache of the latest `ss -tlnp` output for a short window so that a burst
// of warm requests (or banner recomputes) does not all shell out to `ss`
// independently. The cache lives for at most 2 seconds, which is short
// enough to keep port changes reflected in banners within the existing
// `cached_check!` 10s window.
static SS_OUTPUT_CACHE: OnceLock<Mutex<Option<(Instant, String)>>> = OnceLock::new();
fn ss_output_cache() -> &'static Mutex<Option<(Instant, String)>> {
    SS_OUTPUT_CACHE.get_or_init(|| Mutex::new(None))
}
const SS_OUTPUT_TTL: Duration = Duration::from_secs(2);

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
    let output = ss_output_cached()?;
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

        // Extract PIDs from the last field (e.g.,
        // "users:("node",pid=12345,fd=10)") — SO_REUSEPORT sockets can list
        // several processes, so take all of them and match any cwd.
        let process_info = parts.last().unwrap_or(&"");
        if let Some(pid) = extract_pids(process_info)
            .iter()
            .find(|pid| pid_cwd_matches(**pid, project_path))
        {
            if !ports.contains(&port) {
                ports.push(port);
            }
        }
    }

    ports.sort();
    Ok(ports)
}

/// Get the latest `ss -tlnp` output, refreshing it at most every
/// `SS_OUTPUT_TTL`. This dedupes shell-outs when many banner recomputes
/// happen in a short window (e.g. the warm-burst of child prewarm requests
/// after opening a large parent directory).
fn ss_output_cached() -> Result<String> {
    {
        let cache = ss_output_cache()
            .lock()
            .map_err(|e| anyhow::anyhow!("ss output cache poisoned: {}", e))?;
        if let Some((at, ref out)) = *cache {
            if at.elapsed() < SS_OUTPUT_TTL {
                return Ok(out.clone());
            }
        }
    }
    let fresh = utils::run_with_timeout_stdout("ss", &["-tlnp"], PORT_TIMEOUT)?;
    if let Ok(mut cache) = ss_output_cache().lock() {
        *cache = Some((Instant::now(), fresh.clone()));
    }
    Ok(fresh)
}

/// Extract all PIDs from an ss process info string like
/// `users:("node",pid=12345,fd=10)` — SO_REUSEPORT sockets emit one entry per
/// process, e.g. `users:(("node",pid=111,fd=7),("node",pid=222,fd=7))`, and
/// the previous first-match-only parse missed every process after the first.
fn extract_pids(info: &str) -> Vec<u32> {
    let mut pids = Vec::new();
    let mut rest = info;
    while let Some(pid_start) = rest.find("pid=") {
        let pid_str = &rest[pid_start + 4..];
        let pid_end = pid_str
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(pid_str.len());
        if pid_end > 0 {
            if let Ok(pid) = pid_str[..pid_end].parse::<u32>() {
                if !pids.contains(&pid) {
                    pids.push(pid);
                }
            }
        }
        rest = &pid_str[pid_end..];
    }
    pids
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

/// Fallback: use `lsof +D <path>` to list open files under the project dir
/// and report the LISTEN ports of any process whose cwd is inside it.
fn try_lsof_with_dir(project_path: &Path) -> Result<Vec<u16>> {
    let path_str = project_path.to_string_lossy().to_string();

    // NOTE: no `-a` here on purpose. `lsof -a -i +D dir` would AND the
    // network-file and directory selections, which never matches (socket
    // files have no path under dir). Instead we take the union and filter by
    // pid cwd below, so only processes actually running inside the project
    // count — not every network file on the system.
    let output = utils::run_with_timeout_stdout(
        "lsof",
        &["-i", "-P", "-n", "-F", "pcftn", "+D", &path_str],
        PORT_TIMEOUT,
    )?;

    let mut ports = Vec::new();
    let mut current_pid: Option<u32> = None;
    let mut current_conns: Vec<u16> = Vec::new();

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        let field = &line[..1];
        let value = &line[1..];

        match field {
            "p" => {
                // New process record: flush the previous one if its cwd is
                // inside the project dir.
                if let Some(pid) = current_pid {
                    if pid_cwd_matches(pid, project_path) {
                        for port in &current_conns {
                            if !ports.contains(port) {
                                ports.push(*port);
                            }
                        }
                    }
                }
                current_conns.clear();
                current_pid = value.parse().ok();
            }
            "n" => {
                // Skip `local->remote` entries: those are client connections
                // whose remote port is not project-relevant. Remaining entries
                // are LISTEN sockets (`*:3000`, `[::]:8080`, `127.0.0.1:631`).
                if value.contains("->") {
                    continue;
                }
                if let Some(port_str) = value.rsplit(':').next() {
                    if let Ok(port) = port_str.parse::<u16>() {
                        if !current_conns.contains(&port) {
                            current_conns.push(port);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Flush the last process record.
    if let Some(pid) = current_pid {
        if pid_cwd_matches(pid, project_path) {
            for port in &current_conns {
                if !ports.contains(port) {
                    ports.push(*port);
                }
            }
        }
    }

    ports.sort();
    Ok(ports)
}
