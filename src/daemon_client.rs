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

/// Try to get cached banner data from daemon
pub fn get_banner_cached(path: &Path) -> Option<BannerData> {
    let socket = socket_path().ok()?;
    let stream = UnixStream::connect(&socket).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .ok()?;

    let request = Request::Banner {
        path: path.to_path_buf(),
    };
    serde_json::to_writer(&stream, &request).ok()?;

    let response: Response = serde_json::from_reader(&stream).ok()?;
    match response {
        Response::Banner(data) => Some(*data),
        _ => None,
    }
}

/// Check if daemon is running
pub fn is_daemon_running() -> bool {
    let Ok(socket) = socket_path() else {
        return false;
    };
    if !socket.exists() {
        return false;
    }
    // Try to connect and ping
    let Ok(stream) = UnixStream::connect(&socket) else {
        return false;
    };
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .ok();
    stream
        .set_write_timeout(Some(Duration::from_millis(500)))
        .ok();

    let request = Request::Ping;
    serde_json::to_writer(&stream, &request).ok();
    let response: Result<Response, _> = serde_json::from_reader(&stream);
    matches!(response, Ok(Response::Pong))
}

/// Start daemon in background (auto-start)
pub fn ensure_daemon_running() {
    if is_daemon_running() {
        return;
    }

    // Start daemon in background
    let Ok(exe) = std::env::current_exe() else {
        return;
    };

    let daemon_bin = exe.parent().unwrap().join("cfmd");
    if !daemon_bin.exists() {
        tracing::warn!("cfmd binary not found at {}", daemon_bin.display());
        return;
    }

    match std::process::Command::new(&daemon_bin)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(_) => {
            tracing::info!("Started cfmd daemon");
            // Give daemon a moment to start
            std::thread::sleep(Duration::from_millis(100));
        }
        Err(e) => {
            tracing::warn!("Failed to start cfmd: {}", e);
        }
    }
}
