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
            if daemon_client::is_daemon_running() {
                daemon_client::send_shutdown();
                std::thread::sleep(std::time::Duration::from_millis(200));
            }

            let project_dir = directories::ProjectDirs::from("com", "fab", "fab")
                .ok_or_else(|| anyhow::anyhow!("Cannot determine data directory"))?;
            let data_dir = project_dir.data_dir();
            let mut cleared = Vec::new();

            for file_name in ["banner_cache.json", "dir_sizes.json", "fabd.sock"] {
                let path = data_dir.join(file_name);
                if path.exists() && std::fs::remove_file(&path).is_ok() {
                    cleared.push(path);
                }
            }

            let cache_dir = project_dir.cache_dir().to_path_buf();
            if cache_dir.exists() {
                std::fs::remove_dir_all(&cache_dir)?;
                cleared.push(cache_dir);
            }

            if cleared.is_empty() {
                println!("No cache files found");
            } else {
                println!("Cache cleared: {}", cleared.len());
                for path in cleared {
                    println!("  {}", path.display());
                }
            }
        }
    }
    Ok(())
}
