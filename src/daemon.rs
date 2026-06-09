use anyhow::Result;
use inotify::{Inotify, WatchMask};
use std::collections::{HashMap, HashSet};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use folder_auto_banner::daemon_types::{BannerData, Request, Response};
#[cfg(test)]
use folder_auto_banner::fs::ProjectType;
use folder_auto_banner::fs::{DirEntry, DirSummary};

// Cache entry with TTL
#[derive(Clone)]
struct CacheEntry {
    data: BannerData,
    computed_at: Instant,
    root_mtime: Option<SystemTime>,
}

const CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes
const SIZE_CACHE_REFRESH_TIMEOUT: Duration = Duration::from_millis(1000);
const SOCKET_NAME: &str = "fabd.sock";
const IDLE_TIMEOUT: Duration = Duration::from_secs(600); // 10 minutes
const WATCH_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const ACTIVE_WATCH_DEPTH: usize = 3;
const MAX_ACTIVE_WATCH_DIRS: usize = 512;

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
struct ShallowSnapshot {
    total_items: usize,
    total_size: u64,
    files: usize,
    dirs: usize,
    project_type: ProjectType,
    last_modified: Option<SystemTime>,
    top_items: Vec<ShallowItem>,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
struct ShallowItem {
    name: String,
    is_dir: bool,
    is_file: bool,
    is_symlink: bool,
    size: u64,
    modified: Option<SystemTime>,
    symlink_valid: bool,
}

#[derive(Clone, Debug)]
struct WatchRegistration {
    owner: PathBuf,
    watched_path: PathBuf,
}

struct Daemon {
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    /// Global directory size cache — populated by proactive scan
    dir_sizes: Arc<Mutex<HashMap<PathBuf, u64>>>,
    /// Last observed mtime for each cached directory size
    dir_size_mtimes: Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
    socket_path: PathBuf,
}

impl Daemon {
    fn new() -> Result<Self> {
        let socket_dir = directories::ProjectDirs::from("com", "fab", "fab")
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
            dir_size_mtimes: Arc::new(Mutex::new(HashMap::new())),
            socket_path,
        })
    }

    fn run(&self) -> Result<()> {
        let listener = UnixListener::bind(&self.socket_path)?;
        listener.set_nonblocking(true)?;

        tracing::info!("fabd listening on {}", self.socket_path.display());

        // Start inotify watcher thread for active folders only.
        let cache_clone = self.cache.clone();
        let active_roots = Arc::new(Mutex::new(HashSet::new()));
        let active_roots_clone = active_roots.clone();
        let _watcher_handle = thread::spawn(move || {
            watch_loop(cache_clone, active_roots_clone);
        });

        // Load persisted banner cache after the watcher is ready so watched paths become
        // active immediately. Persisted entries are intentionally left in the cache for
        // fast startup, but active-folder watchers and a cheap root-mtime check catch
        // changes without forcing a full shallow scan on every cache hit.
        let socket_dir =
            directories::ProjectDirs::from("com", "fab", "fab").map(|p| p.data_dir().to_path_buf());
        if let Some(ref dir) = socket_dir {
            let persisted = load_banner_cache(dir);
            let mut cache = self.cache.lock().unwrap_or_else(|e| {
                tracing::warn!("Cache mutex poisoned, recovering: {}", e);
                e.into_inner()
            });
            for (path, data) in persisted {
                active_roots
                    .lock()
                    .unwrap_or_else(|e| {
                        tracing::warn!("Active roots mutex poisoned, recovering");
                        e.into_inner()
                    })
                    .insert(path.clone());
                cache.insert(
                    path.clone(),
                    CacheEntry {
                        data,
                        computed_at: Instant::now() - CACHE_TTL,
                        root_mtime: current_dir_mtime(&path),
                    },
                );
            }
            tracing::info!("Loaded {} banner caches from disk", cache.len());
        }

        let mut last_activity = Instant::now();
        let mut last_save = Instant::now();

        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    last_activity = Instant::now();
                    let cache = self.cache.clone();
                    let dir_sizes = self.dir_sizes.clone();
                    let dir_size_mtimes = self.dir_size_mtimes.clone();
                    let active_roots = active_roots.clone();
                    thread::spawn(move || {
                        if let Err(e) =
                            handle_client(stream, cache, dir_sizes, dir_size_mtimes, active_roots)
                        {
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
                        let socket_dir = directories::ProjectDirs::from("com", "fab", "fab")
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

                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    tracing::error!("Accept error: {}", e);
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }

        // Cleanup — save banner cache to disk before exiting
        let socket_dir =
            directories::ProjectDirs::from("com", "fab", "fab").map(|p| p.data_dir().to_path_buf());
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

/// inotify watcher loop — watches active folders and their shallow descendants.
///
/// The old daemon only watched cached root directories, which caught top-level
/// create/delete/move events but missed nested edits that change displayed
/// directory sizes. The freshness fix validated every cache hit with a shallow
/// scan, which made the daemon feel slow. This keeps cache hits fast by watching
/// active folders more aggressively: once a folder is requested, the daemon watches
/// the folder and a bounded set of descendant directories so nested changes
/// invalidate the cached banner without a full scan on every request.
fn watch_loop(
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    active_roots: Arc<Mutex<HashSet<PathBuf>>>,
) {
    let mut inotify = match Inotify::init() {
        Ok(i) => i,
        Err(e) => {
            tracing::error!("Failed to init inotify: {}", e);
            return;
        }
    };

    let mut watched: HashMap<inotify::WatchDescriptor, Vec<WatchRegistration>> = HashMap::new();
    let mut failed_watches: HashSet<PathBuf> = HashSet::new();
    let mut last_refresh = Instant::now() - WATCH_REFRESH_INTERVAL;

    loop {
        if last_refresh.elapsed() >= WATCH_REFRESH_INTERVAL {
            refresh_active_watchers(
                &mut inotify,
                &active_roots,
                &mut watched,
                &mut failed_watches,
            );
            last_refresh = Instant::now();
        }

        // Read inotify events (non-blocking)
        let mut buffer = [0u8; 8192];
        match inotify.read_events(&mut buffer) {
            Ok(events) => {
                for event in events {
                    let mut invalidated = Vec::new();
                    if let Some(registrations) = watched.get(&event.wd) {
                        for reg in registrations {
                            invalidated.push(reg.owner.clone());
                        }
                    }

                    if !invalidated.is_empty() {
                        let mut cache_guard = cache.lock().unwrap_or_else(|e| {
                            tracing::warn!("Mutex poisoned, recovering");
                            e.into_inner()
                        });
                        for path in &invalidated {
                            if cache_guard.remove(path).is_some() {
                                tracing::info!("Cache invalidated: {}", path.display());
                            }
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

        remove_inactive_watchers(
            &active_roots,
            &mut inotify,
            &mut watched,
            &mut failed_watches,
        );
        thread::sleep(Duration::from_millis(100));
    }
}

fn refresh_active_watchers(
    inotify: &mut Inotify,
    active_roots: &Arc<Mutex<HashSet<PathBuf>>>,
    watched: &mut HashMap<inotify::WatchDescriptor, Vec<WatchRegistration>>,
    failed_watches: &mut HashSet<PathBuf>,
) {
    let roots = active_roots
        .lock()
        .unwrap_or_else(|e| {
            tracing::warn!("Active roots mutex poisoned, recovering");
            e.into_inner()
        })
        .clone();

    let mut targets = Vec::new();
    for root in roots {
        if targets.len() >= MAX_ACTIVE_WATCH_DIRS {
            tracing::warn!(
                "Reached active watcher cap ({} dirs); skipping remaining active folders",
                MAX_ACTIVE_WATCH_DIRS
            );
            break;
        }
        collect_watch_targets(&root, 0, &mut targets, MAX_ACTIVE_WATCH_DIRS);
    }

    for target in targets {
        if watched
            .values()
            .any(|regs| regs.iter().any(|reg| reg.watched_path == target))
            || failed_watches.contains(&target)
        {
            continue;
        }

        if !can_watch_path(&target) {
            failed_watches.insert(target);
            continue;
        }

        match inotify.watches().add(
            &target,
            WatchMask::CREATE
                | WatchMask::DELETE
                | WatchMask::MODIFY
                | WatchMask::MOVE
                | WatchMask::CLOSE_WRITE
                | WatchMask::ATTRIB
                | WatchMask::DELETE_SELF
                | WatchMask::MOVE_SELF,
        ) {
            Ok(wd) => {
                let owner = find_owner_for_watch(&target, active_roots);
                let regs = watched.entry(wd).or_default();
                if !regs.iter().any(|reg| reg.owner == owner) {
                    regs.push(WatchRegistration {
                        owner,
                        watched_path: target.clone(),
                    });
                    tracing::debug!("Watching active path: {}", target.display());
                }
            }
            Err(e) => {
                tracing::debug!("Failed to watch {}: {}", target.display(), e);
                failed_watches.insert(target);
            }
        }
    }
}

fn collect_watch_targets(
    path: &Path,
    depth: usize,
    targets: &mut Vec<PathBuf>,
    max_targets: usize,
) {
    if targets.len() >= max_targets || depth > ACTIVE_WATCH_DEPTH {
        return;
    }

    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(_) => return,
    };

    let is_dir = if meta.is_symlink() {
        std::fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false)
    } else {
        meta.is_dir()
    };

    if !is_dir {
        return;
    }

    targets.push(path.to_path_buf());

    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        if targets.len() >= max_targets {
            return;
        }
        collect_watch_targets(&entry.path(), depth + 1, targets, max_targets);
    }
}

fn can_watch_path(path: &Path) -> bool {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(_) => return false,
    };

    if meta.is_symlink() {
        return std::fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false);
    }

    meta.is_dir()
}

fn find_owner_for_watch(path: &Path, active_roots: &Arc<Mutex<HashSet<PathBuf>>>) -> PathBuf {
    let roots = active_roots
        .lock()
        .unwrap_or_else(|e| {
            tracing::warn!("Active roots mutex poisoned, recovering");
            e.into_inner()
        })
        .clone();

    roots
        .into_iter()
        .filter(|root| path == root || path.starts_with(root))
        .max_by_key(|root| root.components().count())
        .unwrap_or_else(|| path.to_path_buf())
}

fn remove_inactive_watchers(
    active_roots: &Arc<Mutex<HashSet<PathBuf>>>,
    inotify: &mut Inotify,
    watched: &mut HashMap<inotify::WatchDescriptor, Vec<WatchRegistration>>,
    failed_watches: &mut HashSet<PathBuf>,
) {
    let roots = active_roots
        .lock()
        .unwrap_or_else(|e| {
            tracing::warn!("Active roots mutex poisoned, recovering");
            e.into_inner()
        })
        .clone();

    let to_remove: Vec<_> = watched
        .iter_mut()
        .filter_map(|(wd, regs)| {
            regs.retain(|reg| {
                roots.contains(&reg.owner)
                    && reg.owner.exists()
                    && (reg.watched_path.exists() || reg.watched_path == reg.owner)
            });
            if regs.is_empty() {
                Some(wd.clone())
            } else {
                None
            }
        })
        .collect();

    for wd in to_remove {
        if let Some(regs) = watched.remove(&wd) {
            if let Some(reg) = regs.first() {
                inotify.watches().remove(wd).ok();
                tracing::debug!("Stopped watching: {}", reg.watched_path.display());
            }
        }
    }

    failed_watches.retain(|path| is_path_under_any_root(path, &roots));
}

fn is_path_under_any_root(path: &Path, roots: &HashSet<PathBuf>) -> bool {
    roots
        .iter()
        .any(|root| path == root || path.starts_with(root))
}

fn handle_client(
    stream: UnixStream,
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    dir_sizes: Arc<Mutex<HashMap<PathBuf, u64>>>,
    dir_size_mtimes: Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
    active_roots: Arc<Mutex<HashSet<PathBuf>>>,
) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let mut stream = stream;

    let t_start = std::time::Instant::now();
    // Length-prefixed JSON: read 4-byte LE length, then payload.
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let t_read = std::time::Instant::now();
    let req_len = u32::from_le_bytes(len_bytes) as usize;
    let mut req_buf = vec![0u8; req_len];
    stream.read_exact(&mut req_buf)?;
    let request: Request = serde_json::from_slice(&req_buf)?;
    let t_parse = std::time::Instant::now();
    tracing::debug!(
        "Received request: {:?} (read={:?}, parse={:?})",
        request,
        t_read - t_start,
        t_parse - t_read
    );

    let response = match request {
        Request::Banner { path } => {
            let path = path.canonicalize().unwrap_or(path);

            // Check cache — if hit, do a cheap root-mtime check and refresh displayed
            // directory sizes only when their mtime changed.
            let cached_entry = {
                let cache = cache.lock().unwrap_or_else(|e| {
                    tracing::warn!("Mutex poisoned, recovering");
                    e.into_inner()
                });
                tracing::debug!("Cache lookup: path={:?}, entries={}", path, cache.len());
                cache
                    .get(&path)
                    .filter(|entry| entry.computed_at.elapsed() < CACHE_TTL)
                    .cloned()
            };

            if let Some(entry) = cached_entry {
                let t0 = std::time::Instant::now();
                let root_fresh = cache_entry_root_is_fresh(&entry, &path);
                let expired = entry.computed_at.elapsed() >= CACHE_TTL;
                let mut data = entry.data;
                let t1 = std::time::Instant::now();
                if expired || !root_fresh {
                    data = compute_banner_data(&path)?;
                    let mut cache = cache.lock().unwrap_or_else(|e| {
                        tracing::warn!("Mutex poisoned, recovering");
                        e.into_inner()
                    });
                    cache.insert(
                        path.clone(),
                        CacheEntry {
                            data: data.clone(),
                            computed_at: Instant::now(),
                            root_mtime: current_dir_mtime(&path),
                        },
                    );
                    active_roots
                        .lock()
                        .unwrap_or_else(|e| {
                            tracing::warn!("Active roots mutex poisoned, recovering");
                            e.into_inner()
                        })
                        .insert(path.clone());
                }
                refresh_displayed_dir_sizes(
                    &mut data.summary.top_items,
                    &dir_sizes,
                    &dir_size_mtimes,
                );
                data.summary.total_size = data.summary.top_items.iter().map(|item| item.size).sum();
                let t2 = std::time::Instant::now();
                let t3 = std::time::Instant::now();
                send_response(&mut stream, &Response::Banner(Box::new(data)))?;
                let t4 = std::time::Instant::now();
                tracing::debug!(
                    "Cache hit: clone={:?} root_check={:?} send={:?}",
                    t1 - t0,
                    t2 - t1,
                    t4 - t3
                );
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

            // Cache miss or stale shallow snapshot — do full scan
            let mut data = compute_banner_data(&path)?;

            refresh_displayed_dir_sizes(&mut data.summary.top_items, &dir_sizes, &dir_size_mtimes);
            data.summary.total_size = data.summary.top_items.iter().map(|item| item.size).sum();

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
                        root_mtime: current_dir_mtime(&path),
                    },
                );
                active_roots
                    .lock()
                    .unwrap_or_else(|e| {
                        tracing::warn!("Active roots mutex poisoned, recovering");
                        e.into_inner()
                    })
                    .insert(path.clone());
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
                            path.clone(),
                            CacheEntry {
                                data,
                                computed_at: Instant::now(),
                                root_mtime: current_dir_mtime(&path),
                            },
                        );
                        active_roots
                            .lock()
                            .unwrap_or_else(|e| {
                                tracing::warn!("Active roots mutex poisoned, recovering");
                                e.into_inner()
                            })
                            .insert(path);
                    }
                }
            });
            return Ok(()); // No response needed — fire and forget
        }
        Request::DirSize { path } => {
            let mut sizes = dir_sizes.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            let mut mtimes = dir_size_mtimes.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            let size = match sizes.get(&path).copied() {
                Some(size) if mtimes.get(&path).copied().flatten() == current_dir_mtime(&path) => {
                    size
                }
                _ => {
                    let size = compute_dir_size(&path);
                    sizes.insert(path.clone(), size);
                    mtimes.insert(path.clone(), current_dir_mtime(&path));
                    size
                }
            };
            Response::DirSize { path, size }
        }
        Request::Ping => Response::Pong,
        Request::Shutdown => {
            tracing::info!("Shutdown requested");
            // Save banner cache before exiting
            let socket_dir = directories::ProjectDirs::from("com", "fab", "fab")
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
    tracing::trace!("Sent response successfully");

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
    // Length-prefixed JSON: 4-byte LE length, then payload.
    // Using JSON instead of bincode because bincode validates UTF-8 on
    // String fields. JSON always produces valid UTF-8. The key insight:
    // to_vec on a Vec<u8> buffers the entire output, then a single
    // write_all sends it in one syscall — avoiding 1-byte-at-a-time I/O.
    let resp_bytes = serde_json::to_vec(response)?;
    let resp_len = resp_bytes.len() as u32;
    let mut combined = Vec::with_capacity(4 + resp_bytes.len());
    let len_bytes = resp_len.to_le_bytes();
    combined.extend_from_slice(&len_bytes);
    combined.extend_from_slice(&resp_bytes);
    stream.write_all(&combined)?;
    stream.flush()?;
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
    let mut git_info = folder_auto_banner::git::get_git_info_filtered(path, &filter_paths).ok();

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
    Ok(BannerData { summary, git_info })
}

fn cache_entry_root_is_fresh(entry: &CacheEntry, path: &Path) -> bool {
    entry.root_mtime == current_dir_mtime(path)
}

#[cfg(test)]
fn cached_snapshot_is_fresh(cached: &DirSummary, path: &Path) -> bool {
    let Ok(fresh) = shallow_snapshot(path) else {
        return false;
    };

    cached.total_items == fresh.total_items
        && cached.total_size == fresh.total_size
        && cached.files == fresh.files
        && cached.dirs == fresh.dirs
        && cached.project_type == fresh.project_type
        && cached.last_modified.map(datetime_to_system_time) == fresh.last_modified
        && cached.top_items.len() == fresh.top_items.len()
        && cached
            .top_items
            .iter()
            .zip(fresh.top_items.iter())
            .all(|(a, b)| {
                a.name == b.name
                    && a.is_dir == b.is_dir
                    && a.is_file == b.is_file
                    && a.is_symlink == b.is_symlink
                    && a.size == b.size
                    && a.modified.map(datetime_to_system_time) == b.modified
                    && a.symlink_valid == b.symlink_valid
            })
}

#[cfg(test)]
fn datetime_to_system_time(dt: chrono::DateTime<chrono::Utc>) -> SystemTime {
    let secs = dt.timestamp();
    if secs < 0 {
        return SystemTime::UNIX_EPOCH
            .checked_sub(Duration::new(
                secs.unsigned_abs(),
                dt.timestamp_subsec_nanos(),
            ))
            .unwrap_or(SystemTime::UNIX_EPOCH);
    }
    SystemTime::UNIX_EPOCH
        .checked_add(Duration::new(secs as u64, dt.timestamp_subsec_nanos()))
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

#[cfg(test)]
fn shallow_snapshot(path: &Path) -> Result<ShallowSnapshot> {
    let mut top_items = Vec::new();
    let mut total_size = 0;
    let mut files = 0;
    let mut dirs = 0;
    let mut last_modified: Option<SystemTime> = None;

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let symlink_meta = std::fs::symlink_metadata(entry.path()).ok();
        let Some(metadata) = symlink_meta else {
            continue;
        };

        let is_symlink = metadata.file_type().is_symlink();
        let is_dir = if is_symlink {
            std::fs::metadata(entry.path())
                .map(|m| m.is_dir())
                .unwrap_or(false)
        } else {
            metadata.is_dir()
        };
        let is_file = !is_symlink && metadata.is_file();

        if is_dir {
            dirs += 1;
        } else if is_file {
            files += 1;
        }

        let size = metadata.len();
        total_size += size;

        let modified = metadata.modified().ok();
        if let Some(mod_time) = modified {
            if last_modified.is_none() || mod_time > last_modified.unwrap() {
                last_modified = Some(mod_time);
            }
        }

        top_items.push(ShallowItem {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir,
            is_file,
            is_symlink,
            size,
            modified,
            symlink_valid: !is_symlink || std::fs::metadata(entry.path()).is_ok(),
        });
    }

    top_items.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(ShallowSnapshot {
        total_items: top_items.len(),
        total_size,
        files,
        dirs,
        project_type: ProjectType::detect(path),
        last_modified,
        top_items,
    })
}

fn refresh_displayed_dir_sizes(
    items: &mut [DirEntry],
    dir_sizes: &Arc<Mutex<HashMap<PathBuf, u64>>>,
    dir_size_mtimes: &Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
) {
    let sizes = dir_sizes.lock().unwrap_or_else(|e| {
        tracing::warn!("Mutex poisoned, recovering");
        e.into_inner()
    });
    let mtimes = dir_size_mtimes.lock().unwrap_or_else(|e| {
        tracing::warn!("Mutex poisoned, recovering");
        e.into_inner()
    });

    let mut to_compute = Vec::new();
    for item in items.iter_mut().filter(|item| item.is_dir) {
        let current_mtime = current_dir_mtime(&item.path);
        let cached_mtime = mtimes.get(&item.path).copied().flatten();
        let cached_size = sizes.get(&item.path).copied();

        if cached_size.is_none() || cached_mtime != current_mtime {
            to_compute.push((item.path.clone(), current_mtime));
        } else if let Some(size) = cached_size {
            item.size = size;
        }
    }

    drop(mtimes);
    drop(sizes);

    for (path, mtime) in to_compute {
        let size = compute_dir_size(&path);
        let mut sizes = dir_sizes.lock().unwrap_or_else(|e| {
            tracing::warn!("Mutex poisoned, recovering");
            e.into_inner()
        });
        sizes.insert(path.clone(), size);
        drop(sizes);

        let mut mtimes = dir_size_mtimes.lock().unwrap_or_else(|e| {
            tracing::warn!("Mutex poisoned, recovering");
            e.into_inner()
        });
        mtimes.insert(path, mtime);
    }

    let sizes = dir_sizes.lock().unwrap_or_else(|e| {
        tracing::warn!("Mutex poisoned, recovering");
        e.into_inner()
    });
    for item in items.iter_mut().filter(|item| item.is_dir) {
        if let Some(size) = sizes.get(&item.path) {
            item.size = *size;
        }
    }
}

fn current_dir_mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
}

fn compute_dir_size(path: &Path) -> u64 {
    // Use `du -s` which is much faster than recursive Rust, but keep it
    // bounded so a pathological directory cannot hang banner responses.
    let path_arg = path.to_string_lossy();
    if let Ok(stdout) = folder_auto_banner::utils::run_with_timeout_stdout(
        "du",
        &["-s", "--bytes", path_arg.as_ref()],
        SIZE_CACHE_REFRESH_TIMEOUT,
    ) {
        let stdout = stdout.trim();
        if !stdout.is_empty() {
            let size_str = stdout.split_whitespace().next().unwrap_or("0");
            if let Ok(size) = size_str.parse::<u64>() {
                return size;
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

    tracing::info!("fabd started with resource limits (nice=10, ionice=idle)");

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
        let path = directories::ProjectDirs::from("com", "fab", "fab")
            .unwrap()
            .data_dir()
            .join(SOCKET_NAME);
        assert!(path.to_string_lossy().contains("fabd.sock"));
    }

    #[test]
    fn test_cache_entry_creation() {
        let summary = DirSummary::scan(Path::new("/tmp")).unwrap();
        let data = BannerData {
            summary,
            git_info: None,
        };
        let entry = CacheEntry {
            data,
            computed_at: Instant::now(),
            root_mtime: None,
        };
        assert!(entry.computed_at.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn test_cache_ttl() {
        let summary = DirSummary::scan(Path::new("/tmp")).unwrap();
        let entry = CacheEntry {
            data: BannerData {
                summary,
                git_info: None,
            },
            computed_at: Instant::now() - Duration::from_secs(600), // 10 minutes ago
            root_mtime: None,
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
            summary,
            git_info: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("summary"));
    }

    #[test]
    fn test_daemon_new() {
        let daemon = Daemon::new();
        assert!(daemon.is_ok());
    }

    #[test]
    fn test_compute_dir_size() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("sample.txt"), "hello").unwrap();
        let size = compute_dir_size(tmp.path());
        assert!(size > 0);
    }

    #[test]
    fn test_cached_snapshot_is_fresh_detects_new_item() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("a.txt"), "a").unwrap();
        let summary = shallow_snapshot(tmp.path()).unwrap();

        assert!(cached_snapshot_is_fresh_from_snapshot(&summary, tmp.path()));

        std::fs::write(tmp.path().join("b.txt"), "b").unwrap();
        assert!(!cached_snapshot_is_fresh_from_snapshot(
            &summary,
            tmp.path()
        ));
    }

    #[test]
    fn test_cached_snapshot_is_fresh_detects_nested_change() {
        let tmp = tempfile::tempdir().unwrap();
        let child = tmp.path().join("child");
        std::fs::create_dir(&child).unwrap();
        std::fs::write(child.join("nested.txt"), "before").unwrap();
        let summary = shallow_snapshot(tmp.path()).unwrap();

        assert!(cached_snapshot_is_fresh_from_snapshot(&summary, tmp.path()));

        std::thread::sleep(Duration::from_millis(20));
        std::fs::write(child.join("nested-new.txt"), "after").unwrap();
        assert!(!cached_snapshot_is_fresh_from_snapshot(
            &summary,
            tmp.path()
        ));
    }

    fn cached_snapshot_is_fresh_from_snapshot(cached: &ShallowSnapshot, path: &Path) -> bool {
        let Ok(fresh) = shallow_snapshot(path) else {
            return false;
        };
        *cached == fresh
    }

    #[test]
    fn test_refresh_displayed_dir_sizes_updates_changed_directory_size() {
        let tmp = tempfile::tempdir().unwrap();
        let child = tmp.path().join("child");
        std::fs::create_dir(&child).unwrap();
        std::fs::write(child.join("one.txt"), "one").unwrap();

        let mut items = vec![DirEntry {
            name: "child".to_string(),
            path: child.clone(),
            is_dir: true,
            is_file: false,
            is_symlink: false,
            is_exec: true,
            size: 0,
            modified: None,
            perms: String::new(),
            owner: String::new(),
            group: String::new(),
            symlink_target: None,
            symlink_valid: true,
        }];
        let dir_sizes = Arc::new(Mutex::new(HashMap::new()));
        let dir_size_mtimes = Arc::new(Mutex::new(HashMap::new()));

        refresh_displayed_dir_sizes(&mut items, &dir_sizes, &dir_size_mtimes);
        let first_size = items[0].size;
        assert!(first_size > 0);

        std::thread::sleep(Duration::from_millis(20));
        std::fs::write(child.join("two.txt"), "two").unwrap();
        refresh_displayed_dir_sizes(&mut items, &dir_sizes, &dir_size_mtimes);

        assert!(items[0].size > first_size);
    }
}
