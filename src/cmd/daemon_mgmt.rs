//! Daemon management commands — start, stop, status, restart, clear-cache

use anyhow::Result;

use crate::cli::DaemonAction;
use crate::daemon_client;

pub fn run_daemon(action: &DaemonAction) -> Result<()> {
    match action {
        DaemonAction::Start => {
            if daemon_client::is_daemon_running() {
                println!("Daemon is already running");
            } else {
                daemon_client::ensure_daemon_running();
                if daemon_client::is_daemon_running() {
                    println!("Daemon started");
                } else {
                    println!("Failed to start daemon");
                }
            }
        }
        DaemonAction::Stop => {
            if daemon_client::is_daemon_running() {
                daemon_client::send_shutdown();
                println!("Daemon stopped");
            } else {
                println!("Daemon is not running");
            }
        }
        DaemonAction::Status => {
            if daemon_client::is_daemon_running() {
                println!("Daemon is running");
            } else {
                println!("Daemon is not running");
            }
        }
        DaemonAction::Restart => {
            if daemon_client::is_daemon_running() {
                daemon_client::send_shutdown();
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            daemon_client::ensure_daemon_running();
            if daemon_client::is_daemon_running() {
                println!("Daemon restarted");
            } else {
                println!("Failed to restart daemon");
            }
        }
        DaemonAction::ClearCache => {
            daemon_client::send_shutdown();
            let cache_dir = directories::ProjectDirs::from("com", "fab", "fab")
                .map(|p| p.cache_dir().to_path_buf());
            if let Some(dir) = cache_dir {
                if dir.exists() {
                    std::fs::remove_dir_all(&dir)?;
                    println!("Cache cleared: {}", dir.display());
                } else {
                    println!("No cache directory found");
                }
            }
        }
    }
    Ok(())
}
