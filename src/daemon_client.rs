use anyhow::Result;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::daemon_types::{BannerData, Request, Response};

const SOCKET_NAME: &str = "cfmd.sock";

/// Magic byte prefix to select IPC format.
/// `b'J'` = JSON, `b'B'` = bincode
const MAGIC_JSON: u8 = b'J';
const MAGIC_BINCODE: u8 = b'B';

/// Returns true if bincode IPC should be used (opt-in via env var or default).
fn use_bincode() -> bool {
    match std::env::var("CFM_IPC").as_deref() {
        Ok("bincode") => true,
        Ok("json") => false,
        _ => true, // default: bincode (faster)
    }
}

fn socket_path() -> Result<std::path::PathBuf> {
    let path = directories::ProjectDirs::from("com", "cfm", "cfm")
        .ok_or_else(|| anyhow::anyhow!("Cannot determine data directory"))?
        .data_dir()
        .join(SOCKET_NAME);
    Ok(path)
}

/// Send a request and receive a response over the Unix socket.
/// Uses bincode by default (5-10x faster than JSON for the same data).
fn send_and_recv(stream: &mut UnixStream, request: &Request) -> Result<Response> {
    if use_bincode() {
        send_and_recv_bincode(stream, request)
    } else {
        send_and_recv_json(stream, request)
    }
}

fn send_and_recv_json(stream: &mut UnixStream, request: &Request) -> Result<Response> {
    // Magic byte to signal JSON
    use std::io::Write;
    stream.write_all(&[MAGIC_JSON])?;
    // Write request JSON directly to the stream
    let request_json = serde_json::to_string(request)?;
    stream.write_all(request_json.as_bytes())?;
    // Shutdown write end so daemon sees EOF on read
    stream.shutdown(std::net::Shutdown::Write)?;
    // Read response: JSON has no length prefix, so read until EOF
    let mut response_bytes = Vec::new();
    use std::io::Read;
    stream.read_to_end(&mut response_bytes)?;
    let response: Response = serde_json::from_slice(&response_bytes)?;
    Ok(response)
}

fn send_and_recv_bincode(stream: &mut UnixStream, request: &Request) -> Result<Response> {
    use std::io::{Read, Write};
    // Magic byte to signal bincode
    stream.write_all(&[MAGIC_BINCODE])?;
    stream.flush()?;
    // Length-prefixed: 4-byte little-endian length, then payload
    let bytes = bincode::serialize(request)?;
    let len = bytes.len() as u32;
    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(&bytes)?;
    stream.flush()?;
    // Shutdown write end so daemon sees EOF on read
    stream.shutdown(std::net::Shutdown::Write)?;
    // Read 4-byte length prefix, then payload
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let resp_len = u32::from_le_bytes(len_bytes) as usize;
    let mut resp_bytes = vec![0u8; resp_len];
    stream.read_exact(&mut resp_bytes)?;
    let response: Response = bincode::deserialize(&resp_bytes)?;
    Ok(response)
}

/// Fire-and-forget: send a request without waiting for response.
fn send_fire_and_forget(stream: &mut UnixStream, request: &Request) -> Result<()> {
    use std::io::Write;
    if use_bincode() {
        stream.write_all(&[MAGIC_BINCODE])?;
        let bytes = bincode::serialize(request)?;
        let len = bytes.len() as u32;
        stream.write_all(&len.to_le_bytes())?;
        stream.write_all(&bytes)?;
        stream.flush()?;
    } else {
        stream.write_all(&[MAGIC_JSON])?;
        let request_json = serde_json::to_string(request)?;
        stream.write_all(request_json.as_bytes())?;
        stream.flush()?;
    }
    Ok(())
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

    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(1)))
        .ok()?;

    let request = Request::Banner {
        path: path.to_path_buf(),
    };
    let response = send_and_recv(&mut stream, &request).ok()?;
    let t2 = std::time::Instant::now();
    if std::env::var("CFM_PROFILE").is_ok() {
        let payload_bytes = if use_bincode() {
            bincode::serialize(&response).map(|v| v.len()).unwrap_or(0)
        } else {
            serde_json::to_string(&response).map(|s| s.len()).unwrap_or(0)
        };
        eprintln!(
            "[CFM_PROFILE] ipc: connect={:?} send_recv={:?} total={:?} format={} payload={}B",
            t1 - t0,
            t2 - t1,
            t2 - t0,
            if use_bincode() { "bincode" } else { "json" },
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
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
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
    let Ok(mut stream) = UnixStream::connect(&socket) else {
        return;
    };
    stream.set_write_timeout(Some(Duration::from_secs(1))).ok();

    // Send all warm requests over the same connection
    for path in paths {
        let request = Request::Warm { path: path.clone() };
        if let Err(e) = send_fire_and_forget(&mut stream, &request) {
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
            // Poll for socket to appear (up to 1s, checking every 25ms)
            for _ in 0..40 {
                if let Ok(socket) = socket_path() {
                    if socket.exists() && UnixStream::connect(&socket).is_ok() {
                        return;
                    }
                }
                std::thread::sleep(Duration::from_millis(25));
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
    fn test_bincode_roundtrip() {
        let request = Request::Ping;
        let bytes = bincode::serialize(&request).unwrap();
        let deserialized: Request = bincode::deserialize(&bytes).unwrap();
        assert!(matches!(deserialized, Request::Ping));
    }

    #[test]
    fn test_use_bincode_default() {
        // SAFETY: test-only env manipulation
        unsafe {
            std::env::remove_var("CFM_IPC");
        }
        assert!(use_bincode());
    }

    #[test]
    fn test_use_bincode_explicit_json() {
        // SAFETY: test-only env manipulation
        unsafe {
            std::env::set_var("CFM_IPC", "json");
        }
        assert!(!use_bincode());
        unsafe {
            std::env::remove_var("CFM_IPC");
        }
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
