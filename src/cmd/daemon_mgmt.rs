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
                    anyhow::bail!("Failed to start daemon");
                }
            }
        }
        DaemonAction::Stop => {
            if daemon_client::is_daemon_running() {
                daemon_client::send_shutdown();
                // Wait for the daemon to actually exit. `send_shutdown` only
                // sets an in-process flag; the daemon still needs to drain
                // its accept loop and run cache-save + socket-unlink on the
                // way out. Poll for up to 5s.
                let mut exited = false;
                for _ in 0..50 {
                    if !daemon_client::is_daemon_running() {
                        exited = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                if exited {
                    println!("Daemon stopped");
                } else {
                    anyhow::bail!("Daemon did not exit within 5s of shutdown request");
                }
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
                // Wait for the old daemon to actually exit before spawning
                // a replacement, otherwise the second instance sees the
                // live socket and bails with "daemon already running".
                let mut exited = false;
                for _ in 0..50 {
                    if !daemon_client::is_daemon_running() {
                        exited = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                if !exited {
                    anyhow::bail!("Daemon did not exit within 5s of shutdown request");
                }
            }
            daemon_client::ensure_daemon_running();
            if daemon_client::is_daemon_running() {
                println!("Daemon restarted");
            } else {
                anyhow::bail!("Failed to restart daemon");
            }
        }
        DaemonAction::Warm { paths } => {
            // Default to the current directory so `f daemon warm` on its own
            // is useful.
            let targets: Vec<std::path::PathBuf> = if paths.is_empty() {
                vec![std::env::current_dir().unwrap_or_else(|_| ".".into())]
            } else {
                paths.clone()
            };
            // Make sure something is listening before warming, otherwise every
            // warm request is a silent no-op against a missing socket.
            if !daemon_client::is_daemon_running() {
                daemon_client::ensure_daemon_running();
            }
            if !daemon_client::is_daemon_running() {
                anyhow::bail!("Failed to start daemon — cannot warm");
            }
            daemon_client::warm_paths(&targets);
            // `warm_paths` is fire-and-forget (the daemon computes on its own
            // thread and writes the on-disk cache when done), so give it a
            // moment to land before reporting, otherwise the first `cd` right
            // after this command races the warm.
            std::thread::sleep(std::time::Duration::from_millis(
                300 * targets.len().min(10) as u64,
            ));
            for path in &targets {
                println!("Warming {}", path.display());
            }
            println!(
                "Queued {} path(s). The daemon computes them in the background; sizes fill in as they land.",
                targets.len()
            );
        }
        DaemonAction::ClearCache => {
            if daemon_client::is_daemon_running() {
                daemon_client::send_shutdown();
                std::thread::sleep(std::time::Duration::from_millis(200));
            }

            let data_dir = crate::state::get_data_dir()?;
            let mut cleared = Vec::new();

            // Per-path banner data cache (written by banner_data_cache.rs) is
            // served by the client disk fast path — it must be cleared too,
            // otherwise stale banners survive a "clear-cache".
            let banner_data_dir = data_dir.join("banner_data");
            if banner_data_dir.exists() {
                std::fs::remove_dir_all(&banner_data_dir)?;
                cleared.push(banner_data_dir);
            }

            for file_name in ["banner_cache.json", "dir_sizes.json", "fabd.sock"] {
                let path = data_dir.join(file_name);
                if path.exists() && std::fs::remove_file(&path).is_ok() {
                    cleared.push(path);
                }
            }

            let cache_dir = directories::ProjectDirs::from("com", "fab", "fab")
                .map(|project_dir| project_dir.cache_dir().to_path_buf());
            if let Some(cache_dir) = cache_dir {
                if cache_dir.exists() {
                    std::fs::remove_dir_all(&cache_dir)?;
                    cleared.push(cache_dir);
                }
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
