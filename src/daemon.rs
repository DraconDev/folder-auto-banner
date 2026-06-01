use anyhow::Result;
use inotify::{Inotify, WatchMask};
use std::collections::HashMap;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use cfm_lib::daemon_types::{BannerData, Request, Response};
use cfm_lib::fs::DirSummary;

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

        // Load persisted banner cache from disk
        let socket_dir =
            directories::ProjectDirs::from("com", "cfm", "cfm").map(|p| p.data_dir().to_path_buf());
        if let Some(ref dir) = socket_dir {
            let persisted = load_banner_cache(dir);
            let mut cache = self.cache.lock().unwrap_or_else(|e| {
                tracing::warn!("Cache mutex poisoned, recovering: {}", e);
                e.into_inner()
            });
            for (path, data) in persisted {
                cache.insert(
                    path,
                    CacheEntry {
                        data,
                        computed_at: Instant::now(),
                    },
                );
            }
            tracing::info!("Loaded {} banner caches from disk", cache.len());
        }

        // Start inotify watcher thread
        let cache_clone = self.cache.clone();
        let _watcher_handle = thread::spawn(move || {
            watch_loop(cache_clone);
        });

        // Start proactive scan of home directory in background
        let dir_sizes_clone = self.dir_sizes.clone();
        let cache_clone = self.cache.clone();
        let socket_dir =
            directories::ProjectDirs::from("com", "cfm", "cfm").map(|p| p.data_dir().to_path_buf());
        thread::spawn(move || {
            proactive_scan(dir_sizes_clone.clone(), cache_clone.clone());
            // Save to disk when done
            if let Some(dir) = socket_dir {
                let sizes = dir_sizes_clone.lock().unwrap_or_else(|e| {
                    tracing::warn!("Mutex poisoned, recovering");
                    e.into_inner()
                });
                save_size_cache(&dir, &sizes);
            }
        });

        let mut last_activity = Instant::now();
        let mut last_save = Instant::now();

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

                    // Check for signal-based shutdown request
                    #[cfg(unix)]
                    if SHUTDOWN_REQUESTED.load(std::sync::atomic::Ordering::SeqCst) {
                        tracing::info!("Shutdown signal received, shutting down gracefully");
                        break;
                    }

                    // Periodic save every 5 minutes
                    if last_save.elapsed() > Duration::from_secs(300) {
                        let socket_dir = directories::ProjectDirs::from("com", "cfm", "cfm")
                            .map(|p| p.data_dir().to_path_buf());
                        if let Some(dir) = socket_dir {
                            let cache = self.cache.lock().unwrap_or_else(|e| {
                                tracing::warn!("Mutex poisoned, recovering");
                                e.into_inner()
                            });
                            save_banner_cache(&dir, &cache);
                        }
                        last_save = Instant::now();
                    }

                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    tracing::error!("Accept error: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }

        // Cleanup — save banner cache to disk before exiting
        let socket_dir =
            directories::ProjectDirs::from("com", "cfm", "cfm").map(|p| p.data_dir().to_path_buf());
        if let Some(dir) = socket_dir {
            let cache = self.cache.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            save_banner_cache(&dir, &cache);
        }
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
    let mut failed_watches: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    loop {
        // Check for new directories to watch
        {
            let cache = cache.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            for path in cache.keys() {
                if !watched.contains_key(path) && !failed_watches.contains(path) {
                    // Skip non-existent directories and dead symlinks
                    if !path.exists() {
                        failed_watches.insert(path.clone());
                        continue;
                    }
                    // For symlinks, check if target exists
                    if let Ok(meta) = std::fs::symlink_metadata(path) {
                        if meta.is_symlink() && std::fs::metadata(path).is_err() {
                            failed_watches.insert(path.clone());
                            continue; // Dead symlink, skip
                        }
                    }
                    match inotify.watches().add(
                        path,
                        WatchMask::CREATE
                            | WatchMask::DELETE
                            | WatchMask::MODIFY
                            | WatchMask::MOVE
                            | WatchMask::CLOSE_WRITE,
                    ) {
                        Ok(wd) => {
                            tracing::info!("Watching: {}", path.display());
                            watched.insert(path.clone(), wd);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to watch {}: {}", path.display(), e);
                            failed_watches.insert(path.clone());
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
                        let mut cache_guard = cache.lock().unwrap_or_else(|e| {
                            tracing::warn!("Mutex poisoned, recovering");
                            e.into_inner()
                        });
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

        // Remove stale watchers for directories no longer in cache or no longer exist
        {
            let cache = cache.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            let to_remove: Vec<PathBuf> = watched
                .keys()
                .filter(|p| !cache.contains_key(*p) || !p.exists())
                .cloned()
                .collect();
            for path in to_remove {
                if let Some(wd) = watched.remove(&path) {
                    inotify.watches().remove(wd).ok();
                    tracing::info!("Stopped watching: {}", path.display());
                }
            }
            // Also clear failed watches for paths no longer in cache
            failed_watches.retain(|p| cache.contains_key(p));
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

    let mut stream = stream;

    // Length-prefixed bincode: read 4-byte LE length, then payload.
    // Bulk reads via read_exact avoid 1-byte-at-a-time I/O.
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let req_len = u32::from_le_bytes(len_bytes) as usize;
    let mut req_buf = vec![0u8; req_len];
    stream.read_exact(&mut req_buf)?;
    let request: Request = bincode::deserialize(&req_buf)?;
    tracing::debug!("Received request: {:?}", request);

    let response = match request {
        Request::Banner { path } => {
            let path = path.canonicalize().unwrap_or(path);

            // Check cache — if hit, inject sizes and refresh git status
            {
                let cache = cache.lock().unwrap_or_else(|e| {
                    tracing::warn!("Mutex poisoned, recovering");
                    e.into_inner()
                });
                if let Some(entry) = cache.get(&path) {
                    if entry.computed_at.elapsed() < CACHE_TTL {
                        let mut data = entry.data.clone();
                        drop(cache);
                        // Inject sizes from global cache
                        let global_sizes = dir_sizes.lock().unwrap_or_else(|e| {
                            tracing::warn!("Mutex poisoned, recovering");
                            e.into_inner()
                        });
                        for item in &mut data.summary.top_items {
                            if item.is_dir {
                                if let Some(&size) = global_sizes.get(&item.path) {
                                    item.size = size;
                                }
                            }
                        }
                        drop(global_sizes);
                        // Don't refresh git on cache hit — it's cached with TTL
                        use std::io::Read;
                        send_response(&mut stream, &Response::Banner(Box::new(data)))?;
                        // Keep stream alive while client reads
                        let mut discard = [0u8; 256];
                        loop {
                            match stream.read(&mut discard) {
                                Ok(0) => break,
                                Ok(_) => continue,
                                Err(_) => break,
                            }
                        }
                        return Ok(());
                    }
                }
            }

            // Cache miss — do full scan
            let mut data = compute_banner_data(&path)?;

            // Inject sizes from global cache
            let global_sizes = dir_sizes.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
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
                let mut cache = cache.lock().unwrap_or_else(|e| {
                    tracing::warn!("Mutex poisoned, recovering");
                    e.into_inner()
                });
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
        Request::Warm { path } => {
            let path = path.canonicalize().unwrap_or(path);
            let cache = cache.clone();
            // Pre-compute in background — don't block the client
            thread::spawn(move || {
                let cache_hit = {
                    let c = cache.lock().unwrap_or_else(|e| {
                        tracing::warn!("Mutex poisoned, recovering");
                        e.into_inner()
                    });
                    c.get(&path)
                        .map(|e| e.computed_at.elapsed() < CACHE_TTL)
                        .unwrap_or(false)
                };
                if !cache_hit {
                    if let Ok(data) = compute_banner_data(&path) {
                        let mut c = cache.lock().unwrap_or_else(|e| {
                            tracing::warn!("Mutex poisoned, recovering");
                            e.into_inner()
                        });
                        c.insert(
                            path,
                            CacheEntry {
                                data,
                                computed_at: Instant::now(),
                            },
                        );
                    }
                }
            });
            return Ok(()); // No response needed — fire and forget
        }
        Request::DirSize { path } => {
            let sizes = dir_sizes.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            let size = sizes
                .get(&path)
                .copied()
                .unwrap_or_else(|| compute_dir_size(&path));
            Response::DirSize { path, size }
        }
        Request::Ping => Response::Pong,
        Request::Shutdown => {
            tracing::info!("Shutdown requested");
            // Save banner cache before exiting
            let socket_dir = directories::ProjectDirs::from("com", "cfm", "cfm")
                .map(|p| p.data_dir().to_path_buf());
            if let Some(dir) = socket_dir {
                let c = cache.lock().unwrap_or_else(|e| {
                    tracing::warn!("Mutex poisoned, recovering");
                    e.into_inner()
                });
                save_banner_cache(&dir, &c);
            }
            std::process::exit(0);
        }
    };

    send_response(&mut stream, &response)?;

    // Keep stream alive while client reads. The client signals it's done by
    // closing the connection (dropping its UnixStream). Until then, the client
    // may still be reading the response.
    use std::io::Read;
    let mut discard = [0u8; 256];
    loop {
        match stream.read(&mut discard) {
            Ok(0) => break, // EOF — client closed
            Ok(_) => continue,
            Err(_) => break, // timeout or error
        }
    }
    Ok(())
}

fn send_response(stream: &mut UnixStream, response: &Response) -> Result<()> {
    use std::io::Write;
    // Length-prefixed bincode: 4-byte LE length, then payload.
    // Buffering in Vec<u8> avoids 1-byte-at-a-time I/O.
    let resp_bytes = bincode::serialize(response)?;
    let resp_len = resp_bytes.len() as u32;
    let mut header = [0u8; 4];
    header[..4].copy_from_slice(&resp_len.to_le_bytes());
    let mut combined = Vec::with_capacity(4 + resp_bytes.len());
    combined.extend_from_slice(&header);
    combined.extend_from_slice(&resp_bytes);
    match stream.write_all(&combined) {
        Ok(_) => {}
        Err(e) => {
            tracing::warn!("Failed to write response: {}", e);
            return Err(e.into());
        }
    }
    stream.flush()?;
    tracing::trace!("Sent bincode response: {} bytes payload", resp_bytes.len());
    Ok(())
}

fn compute_banner_data(path: &Path) -> Result<BannerData> {
    let summary = DirSummary::scan_with_options(path, false, true, true, true, true)?;

    // Build filter list for git status collection:
    // - For files at root: just the filename (e.g., "Cargo.toml")
    // - For directories: the directory name (e.g., "src") to match all children
    // This tells git2 to only collect statuses for files we'll actually display
    let filter_paths: Vec<String> = summary
        .top_items
        .iter()
        .map(|item| item.name.clone())
        .collect();

    // Use filtered git info — only collects statuses for top_items (much faster for large repos)
    let mut git_info = cfm_lib::git::get_git_info_filtered(path, &filter_paths).ok();

    // For directories, we need to also collect statuses for their immediate children
    // because git pathspec "src" matches src/*, but we display src/daemon.rs etc.
    // The filter already handles this via pathspec matching.
    // Additionally, filter file_statuses to only keep depth 0 or 1 entries.
    if let Some(ref mut gi) = git_info {
        if !gi.file_statuses.is_empty() {
            let keep: std::collections::HashSet<_> = summary
                .top_items
                .iter()
                .map(|item| item.name.clone())
                .collect();
            gi.file_statuses.retain(|path_str, _| {
                let components: Vec<_> = std::path::Path::new(path_str)
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect();
                match components.len() {
                    0 => false,
                    1 => keep.contains(&components[0]),
                    2 => keep.contains(&components[0]), // "src/daemon.rs"
                    _ => false,                         // "target/debug/build/..." — exclude
                }
            });
        }
    }

    // Return immediately — sizes come from global cache
    Ok(BannerData {
        path: path.to_path_buf(),
        summary,
        git_info,
        dir_sizes: HashMap::new(),
        cached_at: chrono::Utc::now(),
    })
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

const SIZE_CACHE_FILE: &str = "dir_sizes.json";
const BANNER_CACHE_FILE: &str = "banner_cache.json";

fn size_cache_path(socket_dir: &Path) -> PathBuf {
    socket_dir.join(SIZE_CACHE_FILE)
}

fn banner_cache_path(socket_dir: &Path) -> PathBuf {
    socket_dir.join(BANNER_CACHE_FILE)
}

fn load_size_cache(socket_dir: &Path) -> HashMap<PathBuf, u64> {
    let path = size_cache_path(socket_dir);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(map) = serde_json::from_str::<HashMap<String, u64>>(&data) {
            let result: HashMap<PathBuf, u64> = map
                .into_iter()
                .map(|(k, v)| (PathBuf::from(k), v))
                .collect();
            tracing::info!("Loaded {} cached directory sizes from disk", result.len());
            return result;
        }
    }
    HashMap::new()
}

fn save_size_cache(socket_dir: &Path, sizes: &HashMap<PathBuf, u64>) {
    let path = size_cache_path(socket_dir);
    let map: HashMap<String, u64> = sizes
        .iter()
        .map(|(k, v)| (k.to_string_lossy().to_string(), *v))
        .collect();
    if let Ok(data) = serde_json::to_string(&map) {
        if std::fs::write(&path, data).is_ok() {
            tracing::info!("Saved {} directory sizes to disk", sizes.len());
        }
    }
}

fn load_banner_cache(socket_dir: &Path) -> HashMap<PathBuf, BannerData> {
    let path = banner_cache_path(socket_dir);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(map) = serde_json::from_str::<HashMap<String, BannerData>>(&data) {
            let result: HashMap<PathBuf, BannerData> = map
                .into_iter()
                .map(|(k, v)| (PathBuf::from(k), v))
                .collect();
            tracing::info!("Loaded {} cached banners from disk", result.len());
            return result;
        }
    }
    HashMap::new()
}

fn save_banner_cache(socket_dir: &Path, cache: &HashMap<PathBuf, CacheEntry>) {
    let path = banner_cache_path(socket_dir);
    let map: HashMap<String, &BannerData> = cache
        .iter()
        .map(|(k, v)| (k.to_string_lossy().to_string(), &v.data))
        .collect();
    if let Ok(data) = serde_json::to_string(&map) {
        if std::fs::write(&path, data).is_ok() {
            tracing::info!("Saved {} banner caches to disk", cache.len());
        }
    }
}

/// Proactively scan home directory and populate global size cache + banner cache
fn proactive_scan(
    dir_sizes: Arc<Mutex<HashMap<PathBuf, u64>>>,
    banner_cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
) {
    let home = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => return,
    };

    tracing::info!("Starting proactive scan of {}", home.display());

    // Find all directories in home (up to 2 levels deep for speed)
    let mut dirs_to_scan: Vec<PathBuf> = Vec::new();

    // Level 1: direct children of home
    if let Ok(entries) = std::fs::read_dir(&home) {
        for entry in entries.flatten() {
            // Use symlink_metadata to detect symlinks without following them
            let meta = match std::fs::symlink_metadata(entry.path()) {
                Ok(m) => m,
                Err(_) => continue,
            };

            if meta.is_symlink() {
                // For symlinks, try to resolve the target
                if let Ok(target_meta) = std::fs::metadata(entry.path()) {
                    if target_meta.is_dir() && !entry.file_name().to_string_lossy().starts_with('.')
                    {
                        dirs_to_scan.push(entry.path());
                    }
                }
                // Skip dead symlinks (target doesn't exist)
            } else if meta.is_dir() && !entry.file_name().to_string_lossy().starts_with('.') {
                dirs_to_scan.push(entry.path());
            }
        }
    }

    // Level 2: subdirectories of visible dirs (for projects like ~/Dev/project)
    let level1: Vec<PathBuf> = dirs_to_scan.clone();
    for dir in &level1 {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten().take(50) {
                let meta = match std::fs::symlink_metadata(entry.path()) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                if meta.is_symlink() {
                    if let Ok(target_meta) = std::fs::metadata(entry.path()) {
                        if target_meta.is_dir() {
                            dirs_to_scan.push(entry.path());
                        }
                    }
                } else if meta.is_dir() {
                    dirs_to_scan.push(entry.path());
                }
            }
        }
    }

    // Track where level 1+2 end (most likely navigation targets)
    let level2_end = dirs_to_scan.len();

    // Level 3: subdirectories of level 2 dirs (for projects like ~/Dev/project/src)
    let level2: Vec<PathBuf> = dirs_to_scan[level1.len()..].to_vec();
    for dir in &level2 {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten().take(20) {
                let meta = match std::fs::symlink_metadata(entry.path()) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                if meta.is_symlink() {
                    if let Ok(target_meta) = std::fs::metadata(entry.path()) {
                        if target_meta.is_dir() {
                            dirs_to_scan.push(entry.path());
                        }
                    }
                } else if meta.is_dir() {
                    dirs_to_scan.push(entry.path());
                }
            }
        }
    }

    // Also scan ALL hidden dirs in home
    if let Ok(entries) = std::fs::read_dir(&home) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                let meta = match std::fs::symlink_metadata(entry.path()) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let is_dir = if meta.is_symlink() {
                    std::fs::metadata(entry.path())
                        .map(|m| m.is_dir())
                        .unwrap_or(false)
                } else {
                    meta.is_dir()
                };

                if is_dir {
                    dirs_to_scan.push(entry.path());
                }
            }
        }
    }

    tracing::info!("Scanning {} directories", dirs_to_scan.len());

    // Run du in batches to avoid ARG_MAX limits
    // Lock per-batch to avoid blocking banner requests for the entire scan
    let batch_size = 50;
    let mut count = 0;

    for chunk in dirs_to_scan.chunks(batch_size) {
        let mut du_args: Vec<String> = vec!["-s".to_string(), "--bytes".to_string()];
        for dir in chunk {
            du_args.push(dir.to_string_lossy().to_string());
        }

        if let Ok(output) = std::process::Command::new("du").args(&du_args).output() {
            let mut batch_sizes = Vec::new();
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.splitn(2, '\t').collect();
                    if parts.len() >= 2 {
                        if let Ok(size) = parts[0].parse::<u64>() {
                            let dir_path = PathBuf::from(parts[1]);
                            batch_sizes.push((dir_path, size));
                            count += 1;
                        }
                    }
                }
            }
            // Brief lock to insert batch, then release
            let mut sizes = dir_sizes.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            for (path, size) in batch_sizes {
                sizes.insert(path, size);
            }
            drop(sizes);
        }
    }

    tracing::info!("Proactive scan complete: {} directory sizes cached", count);

    // Pre-compute banner data for level 1 + level 2 dirs (most likely navigation targets)
    let banner_targets: Vec<PathBuf> = dirs_to_scan[..level2_end.min(dirs_to_scan.len())].to_vec();
    tracing::info!(
        "Pre-computing banners for {} directories",
        banner_targets.len()
    );

    let mut banner_count = 0;
    for path in &banner_targets {
        // Check cache with brief lock, compute outside lock
        let should_compute = {
            let cache = banner_cache.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            !cache
                .get(path)
                .map(|e| e.computed_at.elapsed() < CACHE_TTL)
                .unwrap_or(false)
        };

        if !should_compute {
            continue;
        }

        // Compute banner data outside the lock (expensive operation)
        if let Ok(data) = compute_banner_data(path) {
            // Brief lock to insert
            let mut cache = banner_cache.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            cache.insert(
                path.clone(),
                CacheEntry {
                    data,
                    computed_at: Instant::now(),
                },
            );
            banner_count += 1;
        }
    }

    tracing::info!("Pre-computed {} banner caches", banner_count);

    // Save banner cache to disk
    let socket_dir =
        directories::ProjectDirs::from("com", "cfm", "cfm").map(|p| p.data_dir().to_path_buf());
    if let Some(dir) = socket_dir {
        let cache = banner_cache.lock().unwrap_or_else(|e| {
            tracing::warn!("Mutex poisoned, recovering");
            e.into_inner()
        });
        save_banner_cache(&dir, &cache);
    }
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

    // Install signal handler for graceful shutdown (SIGTERM/SIGINT)
    #[cfg(unix)]
    {
        let handler = signal_wrapper as *const () as libc::sighandler_t;
        unsafe {
            libc::signal(libc::SIGTERM, handler);
            libc::signal(libc::SIGINT, handler);
        }
    }

    tracing::info!("cfmd started with resource limits (nice=10, ionice=idle)");

    let daemon = Daemon::new()?;
    daemon.run()
}

// Global shutdown flag — set by signal handler, checked by daemon loop
#[cfg(unix)]
static SHUTDOWN_REQUESTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(unix)]
extern "C" fn signal_wrapper(_sig: libc::c_int) {
    SHUTDOWN_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        let path = directories::ProjectDirs::from("com", "cfm", "cfm")
            .unwrap()
            .data_dir()
            .join(SOCKET_NAME);
        assert!(path.to_string_lossy().contains("cfmd.sock"));
    }

    #[test]
    fn test_cache_entry_creation() {
        let summary = DirSummary::scan(Path::new("/tmp")).unwrap();
        let data = BannerData {
            path: PathBuf::from("/tmp"),
            summary,
            git_info: None,
            dir_sizes: HashMap::new(),
            cached_at: chrono::Utc::now(),
        };
        let entry = CacheEntry {
            data,
            computed_at: Instant::now(),
        };
        assert!(entry.computed_at.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn test_cache_ttl() {
        let summary = DirSummary::scan(Path::new("/tmp")).unwrap();
        let entry = CacheEntry {
            data: BannerData {
                path: PathBuf::from("/tmp"),
                summary,
                git_info: None,
                dir_sizes: HashMap::new(),
                cached_at: chrono::Utc::now(),
            },
            computed_at: Instant::now() - Duration::from_secs(600), // 10 minutes ago
        };
        assert!(entry.computed_at.elapsed() > CACHE_TTL);
    }

    #[test]
    fn test_request_serialization() {
        let request = Request::Banner {
            path: PathBuf::from("/tmp"),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Banner"));
        assert!(json.contains("/tmp"));
    }

    #[test]
    fn test_response_serialization() {
        let response = Response::Pong;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Pong"));
    }

    #[test]
    fn test_banner_data_serialization() {
        let summary = DirSummary::scan(Path::new("/tmp")).unwrap();
        let data = BannerData {
            path: PathBuf::from("/tmp"),
            summary,
            git_info: None,
            dir_sizes: HashMap::new(),
            cached_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("/tmp"));
    }

    #[test]
    fn test_daemon_new() {
        let daemon = Daemon::new();
        assert!(daemon.is_ok());
    }

    #[test]
    fn test_compute_dir_size() {
        let _size = compute_dir_size(Path::new("/tmp"));
        // Just verify it doesn't panic
    }
}
