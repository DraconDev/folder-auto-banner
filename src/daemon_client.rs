use anyhow::Result;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::daemon_types::{BannerData, Request, Response};

const SOCKET_NAME: &str = "cfmd.sock";

fn socket_path() -> Result<std::path::PathBuf> {
    let path = directories::ProjectDirs::from("com", "cfm", "cfm")
        .ok_or_else(|| anyhow::anyhow!("Cannot determine data directory"))?
        .data_dir()
        .join(SOCKET_NAME);
    Ok(path)
}

fn send_and_recv(stream: &mut UnixStream, request: &Request) -> Result<Response> {
    use std::io::{Read, Write};
    // Length-prefixed bincode: 4-byte LE length, then payload.
    // Buffering in Vec<u8> avoids 1-byte-at-a-time I/O (2500+ syscalls).
    let req_bytes = bincode::serialize(request)?;
    let req_len = req_bytes.len() as u32;
    // Write length prefix and payload in a single write_all call
    let mut header = [0u8; 4];
    header[..4].copy_from_slice(&req_len.to_le_bytes());
    let mut combined = Vec::with_capacity(4 + req_bytes.len());
    combined.extend_from_slice(&header);
    combined.extend_from_slice(&req_bytes);
    stream.write_all(&combined)?;
    // Shutdown write end so daemon sees EOF on read and starts processing
    stream.shutdown(std::net::Shutdown::Write)?;
    // Read 4-byte length prefix, then payload (bulk read, not 1-byte)
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let resp_len = u32::from_le_bytes(len_bytes) as usize;
    let mut resp_bytes = vec![0u8; resp_len];
    stream.read_exact(&mut resp_bytes)?;
    let response: Response = bincode::deserialize(&resp_bytes)?;
    Ok(response)
}

/// Try to get cached banner data from daemon.
/// Auto-starts daemon if socket doesn't exist or is stale.
pub fn get_banner_cached(path: &Path) -> Option<BannerData> {
    let t0 = std::time::Instant::now();
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
    let response = send_and_recv(&mut stream, &request).ok()?;
    let t2 = std::time::Instant::now();
    if std::env::var("CFM_PROFILE").is_ok() {
        let payload_bytes = serde_json::to_string(&response)
            .map(|s| s.len())
            .unwrap_or(0);
        eprintln!(
            "[CFM_PROFILE] ipc: connect={:?} send_recv={:?} total={:?} payload={}B",
            t1 - t0,
            t2 - t1,
            t2 - t0,
            payload_bytes,
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

/// Fire-and-forget: warm multiple paths using a single connection (faster)
pub fn warm_paths(paths: &[PathBuf]) {
    let Ok(socket) = socket_path() else {
        return;
    };
    let Ok(stream) = UnixStream::connect(&socket) else {
        return;
    };
    stream.set_write_timeout(Some(Duration::from_secs(1))).ok();

    // Send all warm requests over the same connection
    for path in paths {
        let request = Request::Warm { path: path.clone() };
        if let Err(e) = serde_json::to_writer(&stream, &request) {
            tracing::warn!("Failed to send warm request: {}", e);
            break; // Connection broken, stop sending
        }
    }
}

/// Send shutdown signal to daemon
#[allow(dead_code)]
pub fn send_shutdown() {
    let Ok(socket) = socket_path() else {
        return;
    };
    if let Ok(mut stream) = UnixStream::connect(&socket) {
        stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(1))).ok();
        let request = Request::Shutdown;
        if let Err(e) = send_and_recv(&mut stream, &request) {
            tracing::warn!("Failed to send shutdown request: {}", e);
        }
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
    let daemon_bin = parent.join("cfmd");
    if !daemon_bin.exists() {
        tracing::warn!("cfmd binary not found at {}", daemon_bin.display());
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
            tracing::info!("Started cfmd daemon");
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
            tracing::warn!("Failed to start cfmd: {}", e);
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
        assert!(path.to_string_lossy().contains("cfmd.sock"));
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
