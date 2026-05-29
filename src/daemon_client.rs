use anyhow::Result;
use std::os::unix::net::UnixStream;
use std::path::Path;
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

fn send_and_recv(stream: &UnixStream, request: &Request) -> Result<Response> {
    serde_json::to_writer(stream, request)?;
    // Shutdown write end so daemon sees EOF on read
    stream.shutdown(std::net::Shutdown::Write)?;
    let response: Response = serde_json::from_reader(stream)?;
    Ok(response)
}

/// Try to get cached banner data from daemon
pub fn get_banner_cached(path: &Path) -> Option<BannerData> {
    let socket = socket_path().ok()?;
    let stream = UnixStream::connect(&socket).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .ok()?;

    let request = Request::Banner {
        path: path.to_path_buf(),
    };
    let response = send_and_recv(&stream, &request).ok()?;
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
    let stream = match UnixStream::connect(&socket) {
        Ok(s) => s,
        Err(_) => {
            // Socket file exists but nobody is listening — remove stale socket
            let _ = std::fs::remove_file(&socket);
            return false;
        }
    };
    stream.set_read_timeout(Some(Duration::from_secs(500))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(500))).ok();

    let request = Request::Ping;
    match send_and_recv(&stream, &request) {
        Ok(Response::Pong) => true,
        _ => {
            // Socket connected but daemon is unresponsive — remove stale socket
            let _ = std::fs::remove_file(&socket);
            false
        }
    }
}

/// Send shutdown signal to daemon
pub fn send_shutdown() {
    let Ok(socket) = socket_path() else {
        return;
    };
    if let Ok(stream) = UnixStream::connect(&socket) {
        stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(1))).ok();
        let request = Request::Shutdown;
        let _ = send_and_recv(&stream, &request);
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

    let daemon_bin = exe.parent().unwrap().join("cfmd");
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
            // Give daemon time to bind socket
            std::thread::sleep(Duration::from_millis(1000));
        }
        Err(e) => {
            tracing::warn!("Failed to start cfmd: {}", e);
        }
    }
}
