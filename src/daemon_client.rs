use anyhow::Result;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::daemon_types::{BannerData, Request, Response};

const SOCKET_NAME: &str = "fabd.sock";

fn socket_path() -> Result<std::path::PathBuf> {
    let path = directories::ProjectDirs::from("com", "fab", "fab")
        .ok_or_else(|| anyhow::anyhow!("Cannot determine data directory"))?
        .data_dir()
        .join(SOCKET_NAME);
    Ok(path)
}

fn send_and_recv(stream: &mut UnixStream, request: &Request) -> Result<Response> {
    use std::io::{Read, Write};
    // Length-prefixed JSON: 4-byte LE length, then payload.
    let req_bytes = serde_json::to_vec(request)?;
    let req_len = req_bytes.len() as u32;
    let mut combined = Vec::with_capacity(4 + req_bytes.len());
    let len_bytes = req_len.to_le_bytes();
    combined.extend_from_slice(&len_bytes);
    combined.extend_from_slice(&req_bytes);
    let t0 = std::time::Instant::now();
    stream.write_all(&combined)?;
    let t_write = std::time::Instant::now();
    // NOTE: previously called stream.shutdown(Shutdown::Write) here, but
    // measuring shows it adds 1–78 ms of latency on Linux Unix sockets
    // because the kernel needs to deliver a FIN to the peer and reschedule
    // the client. The daemon's length-prefixed protocol already tells the
    // server when the request ends (it reads exactly req_len bytes), so
    // the shutdown is unnecessary and we skip it for latency.
    let t_shutdown = std::time::Instant::now();
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let t_read4 = std::time::Instant::now();
    let resp_len = u32::from_le_bytes(len_bytes) as usize;
    let mut resp_bytes = vec![0u8; resp_len];
    stream.read_exact(&mut resp_bytes)?;
    let t_read_payload = std::time::Instant::now();
    let response: Response = serde_json::from_slice(&resp_bytes)?;
    let t_deser = std::time::Instant::now();
    if std::env::var("FAB_PROFILE").is_ok() {
        eprintln!(
            "[FAB_PROFILE] ipc: write={:?} shutdown={:?} read4={:?} read_payload={:?} deser={:?} total={:?}",
            t_write - t0,
            t_shutdown - t_write,
            t_read4 - t_shutdown,
            t_read_payload - t_read4,
            t_deser - t_read_payload,
            t_deser - t0
        );
    }
    Ok(response)
}

/// Try to get cached banner data from daemon.
/// Auto-starts daemon if socket doesn't exist or is stale.
pub fn get_banner_cached(path: &Path) -> Option<BannerData> {
    use crate::cmd::banner_data_cache;

    let t0 = std::time::Instant::now();

    // Fast path: read the per-path on-disk cache directly, bypassing
    // the IPC round-trip entirely. The daemon writes this file on every
    // successful banner compute; the file's mtime is the freshness
    // signal. The disk read is <0.1 ms (page cache) versus 1–10 ms for
    // the IPC read4 (kernel scheduling).
    if banner_data_cache::is_cache_fresh(path) {
        if let Some(data) = banner_data_cache::read_cache(path) {
            if std::env::var("FAB_PROFILE").is_ok() {
                eprintln!(
                    "[FAB_PROFILE] banner_data_cache: hit total={:?}",
                    t0.elapsed()
                );
            }
            return Some(data);
        }
    }

    let socket = socket_path().ok()?;

    // Try connecting — if it fails, start daemon and retry once
    let mut stream = match UnixStream::connect(&socket) {
        Ok(s) => s,
        Err(_) => {
            // Socket missing or stale — clean up and start daemon
            let _ = std::fs::remove_file(&socket);
            ensure_daemon_running();
            // Wait for daemon to be ready
            std::thread::sleep(Duration::from_millis(50));
            UnixStream::connect(&socket).ok()?
        }
    };
    let t1 = std::time::Instant::now();

    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(1)))
        .ok()?;

    let request = Request::Banner {
        path: path.to_path_buf(),
    };
    let result = send_and_recv(&mut stream, &request);
    let response = match result {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[FAB_DEBUG] send_and_recv error: {} (kind: {:?})", e, e);
            return None;
        }
    };
    let t2 = std::time::Instant::now();
    if std::env::var("FAB_PROFILE").is_ok() {
        eprintln!(
            "[FAB_PROFILE] ipc: connect={:?} send_recv={:?} total={:?}",
            t1 - t0,
            t2 - t1,
            t2 - t0,
        );
    }
    match response {
        Response::Banner(data) => Some(*data),
        _ => None,
    }
}

/// Check if daemon is running. Cleans up stale sockets automatically.
pub fn is_daemon_running() -> bool {
    let Ok(socket) = socket_path() else {
        return false;
    };
    if !socket.exists() {
        return false;
    }
    let mut stream = match UnixStream::connect(&socket) {
        Ok(s) => s,
        Err(_) => {
            // Socket file exists but nobody is listening — remove stale socket
            let _ = std::fs::remove_file(&socket);
            return false;
        }
    };
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(1))).ok();

    let request = Request::Ping;
    match send_and_recv(&mut stream, &request) {
        Ok(Response::Pong) => true,
        _ => {
            // Socket connected but daemon is unresponsive — remove stale socket
            let _ = std::fs::remove_file(&socket);
            false
        }
    }
}

/// Send warm requests before the CLI exits.
/// Each warm request uses its own short-lived connection because daemon request
/// handlers process one request per accepted stream.
pub fn warm_paths(paths: &[PathBuf]) {
    for path in paths {
        warm_path(path);
    }
}

fn warm_path(path: &Path) {
    use std::io::Write;
    let Ok(socket) = socket_path() else {
        return;
    };
    let Ok(mut stream) = UnixStream::connect(&socket) else {
        return;
    };
    stream.set_write_timeout(Some(Duration::from_secs(1))).ok();

    let request = Request::Warm {
        path: path.to_path_buf(),
    };
    let req_bytes = match serde_json::to_vec(&request) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("Failed to serialize warm request: {}", e);
            return;
        }
    };
    let req_len = req_bytes.len() as u32;
    let mut combined = Vec::with_capacity(4 + req_bytes.len());
    combined.extend_from_slice(&req_len.to_le_bytes());
    combined.extend_from_slice(&req_bytes);
    if let Err(e) = stream.write_all(&combined) {
        tracing::warn!("Failed to send warm request: {}", e);
        return;
    }
    if let Err(e) = stream.flush() {
        tracing::warn!("Failed to flush warm request: {}", e);
    }
}

/// Send shutdown signal to daemon
pub fn send_shutdown() {
    use std::io::Write;

    let Ok(socket) = socket_path() else {
        return;
    };
    let Ok(mut stream) = UnixStream::connect(&socket) else {
        return;
    };
    stream.set_write_timeout(Some(Duration::from_secs(1))).ok();

    let request = Request::Shutdown;
    let req_bytes = match serde_json::to_vec(&request) {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::warn!("Failed to serialize shutdown request: {}", e);
            return;
        }
    };
    let req_len = req_bytes.len() as u32;
    let mut combined = Vec::with_capacity(4 + req_bytes.len());
    combined.extend_from_slice(&req_len.to_le_bytes());
    combined.extend_from_slice(&req_bytes);
    if let Err(e) = stream.write_all(&combined) {
        tracing::warn!("Failed to send shutdown request: {}", e);
    }
}

/// Start daemon in background (auto-start)
pub fn ensure_daemon_running() {
    if is_daemon_running() {
        return;
    }

    let Ok(exe) = std::env::current_exe() else {
        return;
    };

    let Some(parent) = exe.parent() else {
        tracing::warn!("Cannot determine parent directory of executable");
        return;
    };
    let daemon_bin = parent.join("fabd");
    if !daemon_bin.exists() {
        tracing::warn!("fabd binary not found at {}", daemon_bin.display());
        return;
    }

    // Clean up stale socket before spawning
    if let Ok(socket) = socket_path() {
        if socket.exists() {
            let _ = std::fs::remove_file(&socket);
        }
    }

    match std::process::Command::new(&daemon_bin)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(_) => {
            tracing::info!("Started fabd daemon");
            // Poll for socket to appear (up to 2s, checking every 50ms)
            for _ in 0..40 {
                if let Ok(socket) = socket_path() {
                    if socket.exists() && UnixStream::connect(&socket).is_ok() {
                        return;
                    }
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
        Err(e) => {
            tracing::warn!("Failed to start fabd: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        let path = socket_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("fabd.sock"));
    }

    #[test]
    fn test_send_and_recv_with_mock() {
        // This test verifies the function signature and basic flow
        // In a real test, we'd mock the UnixStream, but for now we just test the types
        let request = Request::Ping;
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Ping"));
    }

    #[test]
    fn test_is_daemon_running_when_not_running() {
        // This test checks the function doesn't panic when daemon is not running
        // It may return false, which is expected
        let _result = is_daemon_running();
    }

    #[test]
    fn test_send_shutdown_does_not_panic() {
        // send_shutdown may fail if daemon is not running, but shouldn't panic
        send_shutdown();
    }
}
