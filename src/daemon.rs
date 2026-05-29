#![allow(dead_code)]

use anyhow::Result;
use inotify::{Inotify, WatchMask};
use std::collections::HashMap;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

mod build_status;
mod cache;
mod code_metrics;
mod daemon_types;
mod docker;
mod fs;
mod git;
mod icon;
mod port_usage;
mod state;
mod todo_scanner;

use daemon_types::{BannerData, Request, Response};
use fs::DirSummary;

// Cache entry with TTL
struct CacheEntry {
    data: BannerData,
    computed_at: Instant,
}

const CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes
const SOCKET_NAME: &str = "cfmd.sock";
const IDLE_TIMEOUT: Duration = Duration::from_secs(600); // 10 minutes

struct Daemon {
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    /// Global directory size cache — populated by proactive scan
    dir_sizes: Arc<Mutex<HashMap<PathBuf, u64>>>,
    socket_path: PathBuf,
}

impl Daemon {
    fn new() -> Result<Self> {
        let socket_dir = directories::ProjectDirs::from("com", "cfm", "cfm")
            .ok_or_else(|| anyhow::anyhow!("Cannot determine data directory"))?
            .data_dir()
            .to_path_buf();

        std::fs::create_dir_all(&socket_dir)?;

        let socket_path = socket_dir.join(SOCKET_NAME);

        // Remove stale socket
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        // Load persistent size cache from disk
        let dir_sizes = load_size_cache(&socket_dir);

        Ok(Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            dir_sizes: Arc::new(Mutex::new(dir_sizes)),
            socket_path,
        })
    }

    fn run(&self) -> Result<()> {
        let listener = UnixListener::bind(&self.socket_path)?;
        listener.set_nonblocking(true)?;

        tracing::info!("cfmd listening on {}", self.socket_path.display());

        // Start inotify watcher thread
        let cache_clone = self.cache.clone();
        let _watcher_handle = thread::spawn(move || {
            watch_loop(cache_clone);
        });

        // Start proactive scan of home directory in background
        let dir_sizes_clone = self.dir_sizes.clone();
        let socket_dir = directories::ProjectDirs::from("com", "cfm", "cfm")
            .map(|p| p.data_dir().to_path_buf());
        thread::spawn(move || {
            proactive_scan(dir_sizes_clone.clone());
            // Save to disk when done
            if let Some(dir) = socket_dir {
                let sizes = dir_sizes_clone.lock().unwrap();
                save_size_cache(&dir, &sizes);
            }
        });

        let mut last_activity = Instant::now();

        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    last_activity = Instant::now();
                    let cache = self.cache.clone();
                    let dir_sizes = self.dir_sizes.clone();
                    thread::spawn(move || {
                        if let Err(e) = handle_client(stream, cache, dir_sizes) {
                            tracing::error!("Client error: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No pending connections — check idle timeout
                    if last_activity.elapsed() > IDLE_TIMEOUT {
                        tracing::info!("Idle timeout, shutting down");
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    tracing::error!("Accept error: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }

        // Cleanup
        std::fs::remove_file(&self.socket_path).ok();
        Ok(())
    }
}

/// inotify watcher loop — monitors cached directories for changes
fn watch_loop(cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>) {
    let mut inotify = match Inotify::init() {
        Ok(i) => i,
        Err(e) => {
            tracing::error!("Failed to init inotify: {}", e);
            return;
        }
    };

    let mut watched: HashMap<PathBuf, inotify::WatchDescriptor> = HashMap::new();

    loop {
        // Check for new directories to watch
        {
            let cache = cache.lock().unwrap();
            for path in cache.keys() {
                if !watched.contains_key(path) {
                    match inotify.watches().add(
                        path,
                        WatchMask::CREATE | WatchMask::DELETE | WatchMask::MODIFY | WatchMask::MOVE,
                    ) {
                        Ok(wd) => {
                            tracing::info!("Watching: {}", path.display());
                            watched.insert(path.clone(), wd);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to watch {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        // Read inotify events (non-blocking)
        let mut buffer = [0u8; 4096];
        match inotify.read_events(&mut buffer) {
            Ok(events) => {
                for event in events {
                    // Find which cached directory this event belongs to
                    let mut invalidated = Vec::new();
                    for (path, wd) in &watched {
                        if event.wd == *wd {
                            invalidated.push(path.clone());
                        }
                    }

                    // Invalidate affected cache entries
                    if !invalidated.is_empty() {
                        let mut cache_guard = cache.lock().unwrap();
                        for path in &invalidated {
                            cache_guard.remove(path);
                            tracing::info!("Cache invalidated: {}", path.display());
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No events — continue
            }
            Err(_) => {
                // Error — continue loop
            }
        }

        // Remove stale watchers for directories no longer in cache
        {
            let cache = cache.lock().unwrap();
            let to_remove: Vec<PathBuf> = watched
                .keys()
                .filter(|p| !cache.contains_key(*p))
                .cloned()
                .collect();
            for path in to_remove {
                if let Some(wd) = watched.remove(&path) {
                    inotify.watches().remove(wd).ok();
                    tracing::info!("Stopped watching: {}", path.display());
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn handle_client(
    stream: UnixStream,
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    dir_sizes: Arc<Mutex<HashMap<PathBuf, u64>>>,
) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let reader = stream.try_clone()?;
    let mut writer = stream;

    let request: Request = serde_json::from_reader(&reader)?;

    let response = match request {
        Request::Banner { path } => {
            let path = path.canonicalize().unwrap_or(path);

            // Check cache
            {
                let cache = cache.lock().unwrap();
                if let Some(entry) = cache.get(&path) {
                    if entry.computed_at.elapsed() < CACHE_TTL {
                        return send_response(
                            &mut writer,
                            &Response::Banner(Box::new(entry.data.clone())),
                        );
                    }
                }
            }

            // Cache miss — compute (fast, without dir sizes)
            let mut data = compute_banner_data(&path)?;

            // Inject sizes from global cache
            let global_sizes = dir_sizes.lock().unwrap();
            for item in &mut data.summary.top_items {
                if item.is_dir {
                    if let Some(&size) = global_sizes.get(&item.path) {
                        item.size = size;
                    }
                }
            }
            drop(global_sizes);

            // Store in cache
            {
                let mut cache = cache.lock().unwrap();
                cache.insert(
                    path.clone(),
                    CacheEntry {
                        data: data.clone(),
                        computed_at: Instant::now(),
                    },
                );
            }

            Response::Banner(Box::new(data))
        }
        Request::DirSize { path } => {
            let sizes = dir_sizes.lock().unwrap();
            let size = sizes.get(&path).copied().unwrap_or_else(|| compute_dir_size(&path));
            Response::DirSize { path, size }
        }
        Request::Ping => Response::Pong,
        Request::Shutdown => {
            tracing::info!("Shutdown requested");
            std::process::exit(0);
        }
    };

    send_response(&mut writer, &response)
}

fn send_response(writer: &mut UnixStream, response: &Response) -> Result<()> {
    serde_json::to_writer(writer, response)?;
    Ok(())
}

fn compute_banner_data(path: &Path) -> Result<BannerData> {
    let summary = DirSummary::scan_with_options(path, false, true, true, true, true)?;
    let git_info = git::get_git_info(path).ok();

    // Return immediately with empty dir_sizes — compute in background
    Ok(BannerData {
        path: path.to_path_buf(),
        summary,
        git_info,
        dir_sizes: HashMap::new(),
        cached_at: chrono::Utc::now(),
    })
}

/// Compute directory sizes in background and update cache
fn compute_dir_sizes_background(
    path: PathBuf,
    items: Vec<crate::fs::DirEntry>,
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
) {
    thread::spawn(move || {
        tracing::info!(
            "Computing sizes for {} directories in {}",
            items.len(),
            path.display()
        );

        // Run du for all directories at once (much faster than one-by-one)
        let mut du_args: Vec<String> = vec!["-s".to_string(), "--bytes".to_string()];
        for item in &items {
            du_args.push(item.path.to_string_lossy().to_string());
        }

        let mut dir_sizes = HashMap::new();

        match std::process::Command::new("du").args(&du_args).output() {
            Ok(output) => {
                if !output.stderr.is_empty() {
                    tracing::debug!(
                        "du stderr: {}",
                        String::from_utf8_lossy(&output.stderr).trim()
                    );
                }
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line.splitn(2, '\t').collect();
                        if parts.len() >= 2 {
                            if let Ok(size) = parts[0].parse::<u64>() {
                                let dir_path = PathBuf::from(parts[1]);
                                dir_sizes.insert(dir_path, size);
                            }
                        }
                    }
                }
                tracing::info!(
                    "Computed sizes for {} directories in {}",
                    dir_sizes.len(),
                    path.display()
                );
            }
            Err(e) => {
                tracing::error!("Failed to run du: {}", e);
            }
        }

        // Update the cache entry with computed sizes
        let mut cache = cache.lock().unwrap();
        if let Some(entry) = cache.get_mut(&path) {
            entry.data.dir_sizes = dir_sizes;
        } else {
            tracing::warn!("Cache entry for {} expired before sizes were computed", path.display());
        }
    });
}

fn compute_dir_size(path: &Path) -> u64 {
    // Use `du -s` which is much faster than recursive Rust
    if let Ok(output) = std::process::Command::new("du")
        .args(["-s", "--bytes"])
        .arg(path)
        .output()
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            if let Some(size_str) = stdout.split_whitespace().next() {
                if let Ok(size) = size_str.parse::<u64>() {
                    return size;
                }
            }
        }
    }
    // Fallback: just the directory inode size
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Set resource limits: low CPU priority and idle IO priority
    #[cfg(unix)]
    {
        // nice: 10 = lower priority (range -20 to 19, higher = lower priority)
        unsafe {
            libc::nice(10);
        }
        // ionice: 3 = idle priority class
        let _ = std::process::Command::new("ionice")
            .args(["-c", "3", "-p", &std::process::id().to_string()])
            .output();
    }

    tracing::info!("cfmd started with resource limits (nice=10, ionice=idle)");

    let daemon = Daemon::new()?;
    daemon.run()
}
