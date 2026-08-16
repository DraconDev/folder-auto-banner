use anyhow::Result;
use inotify::{Inotify, WatchMask};
use std::collections::{HashMap, HashSet};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
const SIZE_CACHE_REFRESH_TIMEOUT: Duration = Duration::from_millis(750);
const BACKGROUND_SIZE_CACHE_REFRESH_TIMEOUT: Duration = Duration::from_secs(5);
const ACTIVE_SIZE_REFRESH_TIMEOUT: Duration = Duration::from_secs(10);
const ACTIVE_SIZE_REFRESH_INTERVAL: Duration = Duration::from_secs(30);
const ACTIVE_SIZE_REFRESH_ROOTS_PER_TICK: usize = 5;
const MAX_SIZE_COMPUTE_THREADS: usize = 16;
const SOCKET_NAME: &str = "fabd.sock";
const IDLE_TIMEOUT: Duration = Duration::from_secs(600); // 10 minutes
const WATCH_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const ACTIVE_WATCH_DEPTH: usize = 3;
const MAX_ACTIVE_WATCH_DIRS: usize = 2048;

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

#[derive(Clone, Copy)]
struct SizeComputation {
    size: u64,
    measured: bool,
}

type SizeComputeResult = (usize, u64, Option<SystemTime>, bool);

struct SizeRefreshGuard {
    in_flight: Arc<Mutex<HashSet<PathBuf>>>,
    path: PathBuf,
}

impl Drop for SizeRefreshGuard {
    fn drop(&mut self) {
        let mut in_flight = self.in_flight.lock().unwrap_or_else(|e| {
            tracing::warn!("In-flight size-refresh mutex poisoned, recovering");
            e.into_inner()
        });
        in_flight.remove(&self.path);
    }
}

#[derive(Clone)]
struct SizeRefreshContext {
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    dir_sizes: Arc<Mutex<HashMap<PathBuf, u64>>>,
    dir_size_mtimes: Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
    pending_size_refreshes: Arc<Mutex<Vec<PathBuf>>>,
    size_refresh_in_flight: Arc<Mutex<HashSet<PathBuf>>>,
    active_roots: Arc<Mutex<HashSet<PathBuf>>>,
    active_order: Arc<Mutex<Vec<PathBuf>>>,
}

struct Daemon {
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    /// Global directory size cache — populated by proactive scan
    dir_sizes: Arc<Mutex<HashMap<PathBuf, u64>>>,
    /// Last observed mtime for each cached directory size
    dir_size_mtimes: Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
    pending_size_refreshes: Arc<Mutex<Vec<PathBuf>>>,
    size_refresh_in_flight: Arc<Mutex<HashSet<PathBuf>>>,
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

        // Load persistent size cache from disk, including mtimes so cached sizes can be
        // validated without recomputing every directory on daemon restart.
        let (dir_sizes, dir_size_mtimes) = load_size_cache(&socket_dir);

        Ok(Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            dir_sizes: Arc::new(Mutex::new(dir_sizes)),
            dir_size_mtimes: Arc::new(Mutex::new(dir_size_mtimes)),
            pending_size_refreshes: Arc::new(Mutex::new(Vec::new())),
            size_refresh_in_flight: Arc::new(Mutex::new(HashSet::new())),
            socket_path,
        })
    }

    fn run(&self) -> Result<()> {
        let listener = UnixListener::bind(&self.socket_path)?;
        listener.set_nonblocking(true)?;

        tracing::info!("fabd listening on {}", self.socket_path.display());

        // Start inotify watcher thread for active folders only.
        let cache_clone = self.cache.clone();
        let dir_sizes_clone = self.dir_sizes.clone();
        let dir_size_mtimes_clone = self.dir_size_mtimes.clone();
        let active_roots = Arc::new(Mutex::new(HashSet::new()));
        let active_order = Arc::new(Mutex::new(Vec::new()));
        let active_roots_clone = active_roots.clone();
        let active_order_clone = active_order.clone();
        let _watcher_handle = thread::spawn(move || {
            watch_loop(
                cache_clone,
                dir_sizes_clone,
                dir_size_mtimes_clone,
                active_roots_clone,
                active_order_clone,
            );
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
                active_order
                    .lock()
                    .unwrap_or_else(|e| {
                        tracing::warn!("Active order mutex poisoned, recovering");
                        e.into_inner()
                    })
                    .insert(0, path.clone());
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

        let size_refresh_ctx = Arc::new(SizeRefreshContext {
            cache: self.cache.clone(),
            dir_sizes: self.dir_sizes.clone(),
            dir_size_mtimes: self.dir_size_mtimes.clone(),
            pending_size_refreshes: self.pending_size_refreshes.clone(),
            size_refresh_in_flight: self.size_refresh_in_flight.clone(),
            active_roots: active_roots.clone(),
            active_order: active_order.clone(),
        });
        let active_size_refresh_ctx = size_refresh_ctx.clone();
        thread::spawn(move || active_size_refresh_loop(active_size_refresh_ctx));

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
                    let active_order = active_order.clone();
                    let size_refresh_ctx = size_refresh_ctx.clone();
                    thread::spawn(move || {
                        if let Err(e) = handle_client(
                            stream,
                            cache,
                            dir_sizes,
                            dir_size_mtimes,
                            active_roots,
                            active_order,
                            size_refresh_ctx,
                        ) {
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
                            let dir_sizes = self.dir_sizes.lock().unwrap_or_else(|e| {
                                tracing::warn!("Mutex poisoned, recovering");
                                e.into_inner()
                            });
                            let dir_size_mtimes = self.dir_size_mtimes.lock().unwrap_or_else(|e| {
                                tracing::warn!("Mutex poisoned, recovering");
                                e.into_inner()
                            });
                            save_banner_cache(&dir, &cache);
                            save_size_cache(&dir, &dir_sizes, &dir_size_mtimes);
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

        // Cleanup — save caches to disk before exiting
        let socket_dir =
            directories::ProjectDirs::from("com", "fab", "fab").map(|p| p.data_dir().to_path_buf());
        if let Some(dir) = socket_dir {
            let cache = self.cache.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            let dir_sizes = self.dir_sizes.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            let dir_size_mtimes = self.dir_size_mtimes.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            save_banner_cache(&dir, &cache);
            save_size_cache(&dir, &dir_sizes, &dir_size_mtimes);
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
/// the folder and a bounded set of descendant files/directories so nested changes
/// invalidate the cached banner without a full scan on every request.
fn watch_loop(
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    dir_sizes: Arc<Mutex<HashMap<PathBuf, u64>>>,
    dir_size_mtimes: Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
    active_roots: Arc<Mutex<HashSet<PathBuf>>>,
    active_order: Arc<Mutex<Vec<PathBuf>>>,
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
    let mut last_cleanup = Instant::now() - WATCH_REFRESH_INTERVAL;
    let mut last_roots: HashSet<PathBuf> = HashSet::new();
    let mut last_order: Vec<PathBuf> = Vec::new();

    loop {
        let now = Instant::now();
        let mut roots_snapshot = if last_refresh.elapsed() >= WATCH_REFRESH_INTERVAL {
            Some(
                active_roots
                    .lock()
                    .unwrap_or_else(|e| {
                        tracing::warn!("Active roots mutex poisoned, recovering");
                        e.into_inner()
                    })
                    .clone(),
            )
        } else {
            None
        };
        let mut order_snapshot = if last_refresh.elapsed() >= WATCH_REFRESH_INTERVAL {
            Some(
                active_order
                    .lock()
                    .unwrap_or_else(|e| {
                        tracing::warn!("Active order mutex poisoned, recovering");
                        e.into_inner()
                    })
                    .clone(),
            )
        } else {
            None
        };

        if last_refresh.elapsed() >= WATCH_REFRESH_INTERVAL {
            let roots = roots_snapshot.take().unwrap_or_else(|| {
                active_roots
                    .lock()
                    .unwrap_or_else(|e| {
                        tracing::warn!("Active roots mutex poisoned, recovering");
                        e.into_inner()
                    })
                    .clone()
            });
            let order = order_snapshot.take().unwrap_or_else(|| {
                active_order
                    .lock()
                    .unwrap_or_else(|e| {
                        tracing::warn!("Active order mutex poisoned, recovering");
                        e.into_inner()
                    })
                    .clone()
            });

            if roots != last_roots || order != last_order {
                refresh_active_watchers(
                    &mut inotify,
                    &roots,
                    &order,
                    &mut watched,
                    &mut failed_watches,
                );
                last_roots = roots;
                last_order = order;
            }

            last_refresh = now;
        }

        if last_cleanup.elapsed() >= WATCH_REFRESH_INTERVAL {
            let roots = if let Some(roots) = roots_snapshot.take() {
                roots
            } else {
                active_roots
                    .lock()
                    .unwrap_or_else(|e| {
                        tracing::warn!("Active roots mutex poisoned, recovering");
                        e.into_inner()
                    })
                    .clone()
            };

            active_order
                .lock()
                .unwrap_or_else(|e| {
                    tracing::warn!("Active order mutex poisoned, recovering");
                    e.into_inner()
                })
                .retain(|path| roots.contains(path));

            remove_inactive_watchers(
                &roots,
                &active_order,
                &mut inotify,
                &mut watched,
                &mut failed_watches,
            );
            last_cleanup = now;
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
                        // Determine if this event is for the root directory
                        // itself or for a descendant. A root event
                        // (create/delete/rename of a direct child) means the
                        // item listing may have changed, so the banner cache
                        // must be invalidated. A descendant event on a file
                        // that has a content-probe extension (text files,
                        // images, archives) can affect the banner data
                        // (line count, dimensions, entry count), so we
                        // invalidate the banner cache for those events too.
                        // Other descendant events (e.g., metadata-only
                        // changes to binary files) only affect the size
                        // cache.
                        let is_root_event = watched
                            .get(&event.wd)
                            .and_then(|regs| regs.first())
                            .map(|r| r.watched_path == r.owner)
                            .unwrap_or(false);
                        let is_file_modify = event.mask.contains(inotify::EventMask::MODIFY)
                            || event.mask.contains(inotify::EventMask::CLOSE_WRITE);
                        // A MODIFY/CLOSE_WRITE on a file with a
                        // content-probe extension means the banner data
                        // (line count, image dimensions, archive entry
                        // count) may have changed. Invalidate the banner
                        // cache for these events.
                        let has_content_probe_ext = if is_file_modify
                            && !event.mask.contains(inotify::EventMask::ISDIR)
                        {
                            // Get the file name from the event. The
                            // event name is an optional relative path
                            // under the watched directory.
                            event
                                .name
                                .map(|n| {
                                    is_content_probe_ext(&n.to_string_lossy().to_ascii_lowercase())
                                })
                                .unwrap_or(false)
                        } else {
                            false
                        };
                        let invalidate_banner = is_root_event || has_content_probe_ext;

                        if invalidate_banner {
                            let mut cache_guard = cache.lock().unwrap_or_else(|e| {
                                tracing::warn!("Mutex poisoned, recovering");
                                e.into_inner()
                            });
                            // Only invalidate if the cache entry is older than
                            // 10 seconds. This prevents rapid-fire invalidations
                            // in active directories like /tmp.
                            const MIN_INVALIDATION_AGE: Duration = Duration::from_secs(10);
                            for path in &invalidated {
                                if let Some(entry) = cache_guard.get(path) {
                                    if entry.computed_at.elapsed() < MIN_INVALIDATION_AGE {
                                        continue;
                                    }
                                }
                                if cache_guard.remove(path).is_some() {
                                    prune_size_cache_for_root(&dir_sizes, &dir_size_mtimes, path);
                                    tracing::info!("Cache invalidated: {}", path.display());
                                }
                            }
                        } else {
                            for path in &invalidated {
                                prune_size_cache_for_root(&dir_sizes, &dir_size_mtimes, path);
                                tracing::debug!(
                                    "Size cache pruned for descendant event under: {}",
                                    path.display()
                                );
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

        thread::sleep(Duration::from_millis(100));
    }
}

fn refresh_active_watchers(
    inotify: &mut Inotify,
    roots: &HashSet<PathBuf>,
    active_order: &[PathBuf],
    watched: &mut HashMap<inotify::WatchDescriptor, Vec<WatchRegistration>>,
    failed_watches: &mut HashSet<PathBuf>,
) {
    let mut targets = Vec::new();
    for root in active_order {
        if targets.len() >= MAX_ACTIVE_WATCH_DIRS {
            tracing::warn!(
                "Reached active watcher cap ({} entries); skipping remaining active folders",
                MAX_ACTIVE_WATCH_DIRS
            );
            break;
        }
        collect_watch_targets(root, 0, &mut targets, MAX_ACTIVE_WATCH_DIRS);
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
                let owner = find_owner_for_watch(&target, roots);
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

    if meta.is_symlink() && !is_dir {
        return;
    }

    // Skip directories whose internal churn should not invalidate the cache.
    // This must be checked before pushing to targets, otherwise the skipped
    // directory itself is still watched and its child events invalidate the
    // cache.
    if is_dir && should_skip_dir(path) {
        return;
    }

    targets.push(path.to_path_buf());

    if !is_dir {
        return;
    }

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

/// Directories whose internal file churn should not invalidate the banner cache.
///
/// VCS internals (`.git`, `.hg`, `.svn`) and build/dependency caches (`target`,
/// `node_modules`, `.next`, `dist`, `build`) constantly create and delete
/// temporary files. If the watcher observed them, the cache would be invalidated
/// every time the daemon or any tool performed a git operation, a build, or
/// installed a dependency — even though none of those events change what the
/// banner should display.
fn should_skip_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    // Use the shared SKIP_DIRS constant so that agent/tool directories
    // (e.g. .pi, .opencode, .claude) are skipped consistently across
    // the daemon's watcher and size computation.
    folder_auto_banner::utils::SKIP_DIRS.contains(&name)
}

fn can_watch_path(path: &Path) -> bool {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(_) => return false,
    };

    if meta.is_symlink() {
        return std::fs::metadata(path).is_ok();
    }

    meta.is_file() || meta.is_dir()
}

fn find_owner_for_watch(path: &Path, active_roots: &HashSet<PathBuf>) -> PathBuf {
    let roots = active_roots;

    roots
        .iter()
        .filter(|root| path == root.as_path() || path.starts_with(root.as_path()))
        .max_by_key(|root| root.components().count())
        .cloned()
        .unwrap_or_else(|| path.to_path_buf())
}

fn remove_inactive_watchers(
    roots: &HashSet<PathBuf>,
    active_order: &Arc<Mutex<Vec<PathBuf>>>,
    inotify: &mut Inotify,
    watched: &mut HashMap<inotify::WatchDescriptor, Vec<WatchRegistration>>,
    failed_watches: &mut HashSet<PathBuf>,
) {
    active_order
        .lock()
        .unwrap_or_else(|e| {
            tracing::warn!("Active order mutex poisoned, recovering");
            e.into_inner()
        })
        .retain(|path| roots.contains(path));

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

    failed_watches.retain(|path| is_path_under_any_root(path, roots));
}

fn is_path_under_any_root(path: &Path, roots: &HashSet<PathBuf>) -> bool {
    roots
        .iter()
        .any(|root| path == root || path.starts_with(root))
}

fn touch_active_root(
    active_roots: &Arc<Mutex<HashSet<PathBuf>>>,
    active_order: &Arc<Mutex<Vec<PathBuf>>>,
    path: PathBuf,
) {
    active_roots
        .lock()
        .unwrap_or_else(|e| {
            tracing::warn!("Active roots mutex poisoned, recovering");
            e.into_inner()
        })
        .insert(path.clone());

    let mut order = active_order.lock().unwrap_or_else(|e| {
        tracing::warn!("Active order mutex poisoned, recovering");
        e.into_inner()
    });
    if let Some(pos) = order.iter().position(|p| p == &path) {
        order.remove(pos);
    }
    order.insert(0, path);
}

fn handle_client(
    stream: UnixStream,
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    dir_sizes: Arc<Mutex<HashMap<PathBuf, u64>>>,
    dir_size_mtimes: Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
    active_roots: Arc<Mutex<HashSet<PathBuf>>>,
    active_order: Arc<Mutex<Vec<PathBuf>>>,
    size_refresh_ctx: Arc<SizeRefreshContext>,
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

    let response = match &request {
        Request::Banner { path } => {
            let path = path.canonicalize().unwrap_or_else(|_| path.clone());
            touch_active_root(&active_roots, &active_order, path.clone());
            let _t_req_start = std::time::Instant::now();

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
                let _t_root_done = t1;
                if expired || !root_fresh {
                    tracing::debug!(
                        "Cache miss/recompute: path={} expired={} root_fresh={} root_mtime={:?} current_mtime={:?}",
                        path.display(),
                        expired,
                        root_fresh,
                        entry.root_mtime,
                        current_dir_mtime(&path),
                    );
                    // Re-compute and replace the outer `data` so the
                    // response below (and the disk cache write) reflects
                    // the freshly-computed banner, not the stale entry.
                    // (Pre-0.6.27 the inner `let data =` shadowed the
                    // outer `data` and the response used the old data.)
                    data = match compute_banner_data(&path) {
                        Ok(data) => data,
                        Err(e) => {
                            send_response(
                                &mut stream,
                                &Response::Error {
                                    message: e.to_string(),
                                },
                            )?;
                            return Ok(());
                        }
                    };
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
                    touch_active_root(&active_roots, &active_order, path.clone());
                }
                if apply_cached_displayed_dir_sizes(
                    &mut data.summary.top_items,
                    &dir_sizes,
                    &dir_size_mtimes,
                ) {
                    enqueue_size_refresh(&size_refresh_ctx, path.clone());
                    schedule_size_refresh(
                        size_refresh_ctx.clone(),
                        path.clone(),
                        data.clone(),
                        BACKGROUND_SIZE_CACHE_REFRESH_TIMEOUT,
                    );
                }
                data.summary.total_size = data.summary.top_items.iter().map(|item| item.size).sum();
                let t2 = std::time::Instant::now();
                let t3 = std::time::Instant::now();
                // Persist to the on-disk cache so the next client
                // call can skip the IPC. We do this on every banner
                // response (cache hit or miss) so the file's mtime
                // stays fresh and the client can rely on it.
                persist_banner_data_cache(&path, &data);
                send_response(&mut stream, &Response::Banner(Box::new(data)))?;
                let t4 = std::time::Instant::now();
                tracing::debug!(
                    "Cache hit: clone={:?} root_check={:?} send={:?} total={:?}",
                    t1 - t0,
                    t2 - t1,
                    t4 - t3,
                    t4 - _t_req_start,
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
            let mut data = match compute_banner_data(&path) {
                Ok(data) => data,
                Err(e) => {
                    send_response(
                        &mut stream,
                        &Response::Error {
                            message: e.to_string(),
                        },
                    )?;
                    return Ok(());
                }
            };

            // Store in cache immediately so follow-up navigation is fast. Size
            // refresh for large directories continues in the background and
            // replaces the cache entry when accurate sizes are ready.
            apply_cached_displayed_dir_sizes(
                &mut data.summary.top_items,
                &dir_sizes,
                &dir_size_mtimes,
            );
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
                touch_active_root(&active_roots, &active_order, path.clone());
            }
            // Persist the banner data to the per-path on-disk cache so
            // the client can skip the IPC round-trip on the next call
            // (the IPC `read4` has a 1–10 ms kernel-scheduling floor).
            persist_banner_data_cache(&path, &data);
            schedule_size_refresh(
                size_refresh_ctx,
                path.clone(),
                data.clone(),
                BACKGROUND_SIZE_CACHE_REFRESH_TIMEOUT,
            );
            Response::Banner(Box::new(data))
        }
        Request::Warm { path } => {
            let path = path.canonicalize().unwrap_or_else(|_| path.clone());
            let cache = cache.clone();
            let active_order = active_order.clone();
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
                    match compute_banner_data(&path) {
                        Ok(mut data) => {
                            apply_cached_displayed_dir_sizes(
                                &mut data.summary.top_items,
                                &dir_sizes,
                                &dir_size_mtimes,
                            );
                            let mut c = cache.lock().unwrap_or_else(|e| {
                                tracing::warn!("Mutex poisoned, recovering");
                                e.into_inner()
                            });
                            c.insert(
                                path.clone(),
                                CacheEntry {
                                    data: data.clone(),
                                    computed_at: Instant::now(),
                                    root_mtime: current_dir_mtime(&path),
                                },
                            );
                            touch_active_root(&active_roots, &active_order, path.clone());
                            drop(c);
                            schedule_size_refresh(
                                size_refresh_ctx,
                                path,
                                data,
                                BACKGROUND_SIZE_CACHE_REFRESH_TIMEOUT,
                            );
                        }
                        Err(e) => {
                            tracing::debug!("Warm request failed for {}: {}", path.display(), e);
                        }
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
            let size = match sizes.get(path).copied() {
                Some(size) if mtimes.get(path).copied().flatten() == current_dir_mtime(path) => {
                    size
                }
                _ => {
                    let computed = compute_dir_size_with_status(path, SIZE_CACHE_REFRESH_TIMEOUT);
                    sizes.insert(path.clone(), computed.size);
                    if computed.measured {
                        mtimes.insert(path.clone(), current_dir_mtime(path));
                    } else {
                        mtimes.insert(path.clone(), None);
                    }
                    computed.size
                }
            };
            Response::DirSize {
                path: path.clone(),
                size,
            }
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
                let sizes = dir_sizes.lock().unwrap_or_else(|e| {
                    tracing::warn!("Mutex poisoned, recovering");
                    e.into_inner()
                });
                let mtimes = dir_size_mtimes.lock().unwrap_or_else(|e| {
                    tracing::warn!("Mutex poisoned, recovering");
                    e.into_inner()
                });
                save_banner_cache(&dir, &c);
                save_size_cache(&dir, &sizes, &mtimes);
            }
            std::process::exit(0);
        }
    };

    if matches!(request, Request::Warm { .. }) {
        return Ok(());
    }

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

/// Write the per-path `BannerData` cache file. Called by the daemon
/// after every successful banner compute (cache miss and cache hit).
/// The file's mtime is the freshness signal that the client checks
/// before opening an IPC connection.
fn persist_banner_data_cache(path: &Path, data: &BannerData) {
    use folder_auto_banner::cmd::banner_data_cache;
    let _ = banner_data_cache::write_cache(path, data);
}

fn compute_banner_data(path: &Path) -> Result<BannerData> {
    let mut summary = DirSummary::scan_with_options(path, false, true, true, true, true, &[])?;

    // Build pathspecs for git status collection. Files use their exact
    // top-level name; directories use `dir/*` so native git status only
    // walks immediate children the banner displays or aggregates.
    let filter_paths = folder_auto_banner::git::status_filter_paths_for_items(&summary.top_items);
    // Cache git status for 60s. On a large repo (e.g. dracon-platform
    // with 15K commits and a 5.8 GB .git), the first git status call
    // can take 8+ seconds. The daemon's BannerCache (5 min) covers
    // the common case, but when it expires we don't want to re-pay
    // the full cost. The file cache survives daemon restarts.
    let cache = folder_auto_banner::cache::Cache::new().ok();
    let mut git_info: Option<folder_auto_banner::git::GitInfo> = None;
    if let Some(ref cache) = cache {
        let ck = folder_auto_banner::cache::cache_key(path, "git");
        if let Some(cached) = cache.get(&ck, std::time::Duration::from_secs(60)) {
            git_info = Some(cached);
        }
    }
    if git_info.is_none() {
        git_info = folder_auto_banner::git::get_git_info_filtered(path, &filter_paths).ok();
        if let (Some(ref mut gi), Some(ref cache)) = (&mut git_info, &cache) {
            // Trim to displayable paths BEFORE caching: the unfiltered map can
            // hold tens of thousands of deep untracked entries under large
            // trees (e.g. target/), which bloated the cached payload and IPC.
            let keep: HashSet<_> = summary
                .top_items
                .iter()
                .map(|item| item.name.clone())
                .collect();
            gi.file_statuses.retain(|path_str, _| {
                folder_auto_banner::git::is_displayed_git_status_path(path_str, &keep)
            });
            let _ = cache.set(&ck, gi.clone());
        }
    }

    if let Some(ref mut gi) = git_info {
        if !gi.file_statuses.is_empty() {
            let keep: HashSet<_> = summary
                .top_items
                .iter()
                .map(|item| item.name.clone())
                .collect();
            gi.file_statuses.retain(|path_str, _| {
                folder_auto_banner::git::is_displayed_git_status_path(path_str, &keep)
            });
        }
    }

    populate_content_probes(&mut summary.top_items);

    // Return immediately — sizes come from global cache
    Ok(BannerData { summary, git_info })
}

/// For each file in `items`, run the per-extension content probe and store
/// the result in `entry.content_probe`. Directories are left as `None`; the
/// client populates their child counts from a separate `count_items_in_dir`
/// cache path (or by reading the on-disk count, which is fast).
///
/// This is a sequential walk; in 0.6.25 the client did the same work on
/// every invocation, so the per-call cost is the same on a cold scan but
/// collapses to ~0 for every subsequent call within the cache TTL.
///
/// We probe every file. The probe function is cheap for files with
/// unrecognized extensions (just a `Path::extension` check), and for
/// recognized text files it does a `read_to_string` and counts lines.
/// That work is moved off the client (a short-lived per-`f` process)
/// and onto the daemon (a long-lived cache layer), so it happens at
/// most once per `CACHE_TTL` per directory instead of once per
/// invocation. The trade-off is that an in-place edit of a text file
/// won't update its cached line count until the next refresh, but this
/// is cosmetic and the refresh happens on the 5-minute TTL boundary.
fn populate_content_probes(items: &mut [DirEntry]) {
    use folder_auto_banner::cmd::file_metadata::get_file_contents;
    for entry in items.iter_mut() {
        if !entry.is_file {
            continue;
        }
        let probe = get_file_contents(entry);
        entry.content_probe = if probe.is_empty() {
            // Some("") makes the field appear in serialized output so the
            // client knows the probe was attempted (vs None which means
            // "not probed"). Either way the renderer treats it as empty.
            Some(String::new())
        } else {
            Some(probe)
        };
    }
}

fn cache_entry_root_is_fresh(entry: &CacheEntry, path: &Path) -> bool {
    entry.root_mtime == current_dir_mtime(path)
}

/// Returns true if the (lower-cased) file name has an extension that
/// the daemon runs a content probe on (text files, images, archives,
/// etc.). Used by the inotify watcher to decide whether a
/// MODIFY/CLOSE_WRITE event on a file should invalidate the banner
/// cache (because the line count, image dimensions, archive entry
/// count, etc. may have changed).
pub(crate) fn is_content_probe_ext(lower_name: &str) -> bool {
    lower_name.ends_with(".txt")
        || lower_name.ends_with(".md")
        || lower_name.ends_with(".json")
        || lower_name.ends_with(".js")
        || lower_name.ends_with(".ts")
        || lower_name.ends_with(".jsx")
        || lower_name.ends_with(".tsx")
        || lower_name.ends_with(".rs")
        || lower_name.ends_with(".py")
        || lower_name.ends_with(".go")
        || lower_name.ends_with(".java")
        || lower_name.ends_with(".rb")
        || lower_name.ends_with(".c")
        || lower_name.ends_with(".cpp")
        || lower_name.ends_with(".h")
        || lower_name.ends_with(".hpp")
        || lower_name.ends_with(".sh")
        || lower_name.ends_with(".yaml")
        || lower_name.ends_with(".yml")
        || lower_name.ends_with(".toml")
        || lower_name.ends_with(".xml")
        || lower_name.ends_with(".html")
        || lower_name.ends_with(".css")
        || lower_name.ends_with(".scss")
        || lower_name.ends_with(".png")
        || lower_name.ends_with(".jpg")
        || lower_name.ends_with(".jpeg")
        || lower_name.ends_with(".gif")
        || lower_name.ends_with(".webp")
        || lower_name.ends_with(".bmp")
        || lower_name.ends_with(".tiff")
        || lower_name.ends_with(".zip")
        || lower_name.ends_with(".tar")
        || lower_name.ends_with(".gz")
        || lower_name.ends_with(".mp4")
        || lower_name.ends_with(".mov")
        || lower_name.ends_with(".mkv")
        || lower_name.ends_with(".webm")
        || lower_name.ends_with(".m4v")
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

fn prune_size_cache_for_root(
    dir_sizes: &Arc<Mutex<HashMap<PathBuf, u64>>>,
    dir_size_mtimes: &Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
    root: &Path,
) {
    dir_sizes
        .lock()
        .unwrap_or_else(|e| {
            tracing::warn!("Mutex poisoned, recovering");
            e.into_inner()
        })
        .retain(|path, _| path != root && !path.starts_with(root));

    dir_size_mtimes
        .lock()
        .unwrap_or_else(|e| {
            tracing::warn!("Mutex poisoned, recovering");
            e.into_inner()
        })
        .retain(|path, _| path != root && !path.starts_with(root));
}

fn apply_cached_displayed_dir_sizes(
    items: &mut [DirEntry],
    dir_sizes: &Arc<Mutex<HashMap<PathBuf, u64>>>,
    dir_size_mtimes: &Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
) -> bool {
    let sizes = dir_sizes.lock().unwrap_or_else(|e| {
        tracing::warn!("Mutex poisoned, recovering");
        e.into_inner()
    });
    let mtimes = dir_size_mtimes.lock().unwrap_or_else(|e| {
        tracing::warn!("Mutex poisoned, recovering");
        e.into_inner()
    });
    let mut needs_refresh = false;
    for item in items.iter_mut().filter(|item| item.is_dir) {
        // Skip size computation for directories in SKIP_DIRS (e.g. .pi,
        // .opencode, node_modules). These are large agent/tool directories
        // whose sizes are not useful to display and expensive to compute.
        if should_skip_dir(&item.path) {
            continue;
        }
        let cached_mtime = mtimes.get(&item.path).copied().flatten();
        match sizes.get(&item.path).copied() {
            Some(size) if cached_dir_size_is_fresh(&item.path, size, cached_mtime) => {
                item.size = size;
            }
            _ => needs_refresh = true,
        }
    }
    needs_refresh
}

fn active_size_refresh_loop(ctx: Arc<SizeRefreshContext>) {
    loop {
        thread::sleep(ACTIVE_SIZE_REFRESH_INTERVAL);

        let roots: Vec<PathBuf> = {
            let mut pending = ctx.pending_size_refreshes.lock().unwrap_or_else(|e| {
                tracing::warn!("Pending size-refresh mutex poisoned, recovering");
                e.into_inner()
            });
            let mut out = Vec::with_capacity(ACTIVE_SIZE_REFRESH_ROOTS_PER_TICK);
            let mut remaining = Vec::new();
            for path in pending.drain(..) {
                if out.len() < ACTIVE_SIZE_REFRESH_ROOTS_PER_TICK && !out.contains(&path) {
                    out.push(path);
                } else if !remaining.contains(&path) {
                    remaining.push(path);
                }
            }
            *pending = remaining;
            drop(pending);
            if out.is_empty() {
                let order = ctx.active_order.lock().unwrap_or_else(|e| {
                    tracing::warn!("Active order mutex poisoned, recovering");
                    e.into_inner()
                });
                order
                    .iter()
                    .take(ACTIVE_SIZE_REFRESH_ROOTS_PER_TICK)
                    .cloned()
                    .collect()
            } else {
                out
            }
        };

        for path in roots {
            let data = {
                let cache = ctx.cache.lock().unwrap_or_else(|e| {
                    tracing::warn!("Cache mutex poisoned, recovering");
                    e.into_inner()
                });
                cache
                    .get(&path)
                    .filter(|entry| entry.computed_at.elapsed() < CACHE_TTL)
                    .map(|entry| entry.data.clone())
            };

            if let Some(mut data) = data {
                if apply_cached_displayed_dir_sizes(
                    &mut data.summary.top_items,
                    &ctx.dir_sizes,
                    &ctx.dir_size_mtimes,
                ) {
                    schedule_size_refresh(ctx.clone(), path, data, ACTIVE_SIZE_REFRESH_TIMEOUT);
                }
            }
        }
    }
}

fn enqueue_size_refresh(ctx: &Arc<SizeRefreshContext>, path: PathBuf) {
    let mut pending = ctx.pending_size_refreshes.lock().unwrap_or_else(|e| {
        tracing::warn!("Pending size-refresh mutex poisoned, recovering");
        e.into_inner()
    });
    if !pending.contains(&path) {
        pending.push(path);
    }
}

fn mark_size_refresh_in_flight(ctx: &Arc<SizeRefreshContext>, path: &Path) -> bool {
    let mut in_flight = ctx.size_refresh_in_flight.lock().unwrap_or_else(|e| {
        tracing::warn!("In-flight size-refresh mutex poisoned, recovering");
        e.into_inner()
    });
    in_flight.insert(path.to_path_buf())
}

fn schedule_size_refresh(
    ctx: Arc<SizeRefreshContext>,
    path: PathBuf,
    data: BannerData,
    timeout: Duration,
) {
    if !mark_size_refresh_in_flight(&ctx, &path) {
        return;
    }

    let computed_at = Instant::now();
    thread::spawn(move || {
        let _guard = SizeRefreshGuard {
            in_flight: ctx.size_refresh_in_flight.clone(),
            path: path.clone(),
        };
        let mut refreshed = data;
        refresh_displayed_dir_sizes(
            &mut refreshed.summary.top_items,
            &ctx.dir_sizes,
            &ctx.dir_size_mtimes,
            timeout,
        );
        refreshed.summary.total_size = refreshed
            .summary
            .top_items
            .iter()
            .map(|item| item.size)
            .sum();

        let mut c = ctx.cache.lock().unwrap_or_else(|e| {
            tracing::warn!("Mutex poisoned, recovering");
            e.into_inner()
        });
        let should_replace = c
            .get(&path)
            .map(|entry| entry.computed_at <= computed_at)
            .unwrap_or(true);
        if should_replace {
            c.insert(
                path.clone(),
                CacheEntry {
                    data: refreshed,
                    computed_at,
                    root_mtime: current_dir_mtime(&path),
                },
            );
            touch_active_root(&ctx.active_roots, &ctx.active_order, path);
        }
    });
}

fn refresh_displayed_dir_sizes(
    items: &mut [DirEntry],
    dir_sizes: &Arc<Mutex<HashMap<PathBuf, u64>>>,
    dir_size_mtimes: &Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
    timeout: Duration,
) {
    // First, set sizes from cache where valid, and collect jobs for stale/missing ones.
    let mut jobs: Vec<(usize, PathBuf, Option<SystemTime>)> = Vec::new();
    {
        let sizes = dir_sizes.lock().unwrap_or_else(|e| {
            tracing::warn!("Mutex poisoned, recovering");
            e.into_inner()
        });
        let mtimes = dir_size_mtimes.lock().unwrap_or_else(|e| {
            tracing::warn!("Mutex poisoned, recovering");
            e.into_inner()
        });
        for (idx, item) in items.iter_mut().enumerate() {
            if !item.is_dir {
                continue;
            }
            // Skip size computation for directories in SKIP_DIRS (e.g. .pi,
            // .opencode, node_modules). These are large agent/tool directories
            // whose sizes are not useful to display and expensive to compute.
            if should_skip_dir(&item.path) {
                continue;
            }
            let current_mtime = current_dir_mtime(&item.path);
            let cached_mtime = mtimes.get(&item.path).copied().flatten();
            let cached_size = sizes.get(&item.path).copied();
            if let Some(size) = cached_size {
                if cached_dir_size_is_fresh(&item.path, size, cached_mtime) {
                    item.size = size;
                    continue;
                }
            }
            jobs.push((idx, item.path.clone(), current_mtime));
        }
    } // drop locks

    if jobs.is_empty() {
        return;
    }

    // Compute sizes in parallel to keep banner latency bounded on large trees.
    let results = compute_sizes_parallel(jobs, timeout);

    // Update cache and items.
    let mut sizes = dir_sizes.lock().unwrap_or_else(|e| {
        tracing::warn!("Mutex poisoned, recovering");
        e.into_inner()
    });
    let mut mtimes = dir_size_mtimes.lock().unwrap_or_else(|e| {
        tracing::warn!("Mutex poisoned, recovering");
        e.into_inner()
    });
    for (idx, size, mtime_opt, measured) in results {
        let path = items[idx].path.clone();
        sizes.insert(path.clone(), size);
        if measured {
            if let Some(mt) = mtime_opt {
                mtimes.insert(path, Some(mt));
            }
        } else {
            mtimes.insert(path, None);
        }
        items[idx].size = size;
    }
    drop(sizes);
    drop(mtimes);
}

fn compute_sizes_parallel(
    jobs: Vec<(usize, PathBuf, Option<SystemTime>)>,
    timeout: Duration,
) -> Vec<SizeComputeResult> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    if jobs.is_empty() {
        return Vec::new();
    }
    let worker_count = jobs.len().min(MAX_SIZE_COMPUTE_THREADS);
    let results: std::sync::Mutex<Vec<SizeComputeResult>> =
        std::sync::Mutex::new(Vec::with_capacity(jobs.len()));
    let next = AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..worker_count {
            s.spawn(|| loop {
                let idx = next.fetch_add(1, Ordering::SeqCst);
                if idx >= jobs.len() {
                    break;
                }
                let (orig_idx, path, mtime) = &jobs[idx];
                let computed = compute_dir_size_with_status(path, timeout);
                if let Ok(mut r) = results.lock() {
                    r.push((*orig_idx, computed.size, *mtime, computed.measured));
                }
            });
        }
    });
    results.into_inner().unwrap_or_default()
}

fn current_dir_mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
}

fn compute_dir_size_with_status(path: &Path, timeout: Duration) -> SizeComputation {
    // Use `du -s -b` for logical byte sizes. It is much faster than
    // `du --bytes -x` on large workspace trees while producing the same logical
    // sizes for normal files, so displayed sizes can be populated from cache
    // instead of falling back to the 4 KiB directory inode size.
    let path_arg = path.to_string_lossy();
    if let Ok(stdout) = folder_auto_banner::utils::run_with_timeout_stdout(
        "du",
        &["-s", "-b", path_arg.as_ref()],
        timeout,
    ) {
        let stdout = stdout.trim();
        if !stdout.is_empty() {
            let size_str = stdout.split_whitespace().next().unwrap_or("0");
            if let Ok(size) = size_str.parse::<u64>() {
                return SizeComputation {
                    size,
                    measured: true,
                };
            }
        }
    }
    // Fallback: just the directory inode size. Do not mark the mtime as
    // authoritative, because a timeout should not prevent a later background
    // refresh from retrying once the daemon is idle.
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    SizeComputation {
        size,
        measured: false,
    }
}

fn cached_dir_size_is_fresh(
    path: &Path,
    cached_size: u64,
    cached_mtime: Option<SystemTime>,
) -> bool {
    if cached_mtime != current_dir_mtime(path) {
        return false;
    }

    // Treat the directory inode size as a placeholder rather than a measured
    // value. This catches old cache entries and short `du` timeouts that stored
    // `4096` with a matching mtime, which made stale entries look fresh forever.
    std::fs::metadata(path)
        .map(|metadata| {
            !(metadata.is_dir() && cached_size == metadata.len() && cached_size <= 4096)
        })
        .unwrap_or(false)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedSizeCache {
    sizes: HashMap<String, u64>,
    mtimes: HashMap<String, Option<u128>>,
}

const SIZE_CACHE_FILE: &str = "dir_sizes.json";
const BANNER_CACHE_FILE: &str = "banner_cache.json";

fn size_cache_path(socket_dir: &Path) -> PathBuf {
    socket_dir.join(SIZE_CACHE_FILE)
}

fn banner_cache_path(socket_dir: &Path) -> PathBuf {
    socket_dir.join(BANNER_CACHE_FILE)
}

fn system_time_to_nanos(time: Option<SystemTime>) -> Option<u128> {
    time.and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
}

fn nanos_to_system_time(nanos: u128) -> Option<SystemTime> {
    let secs = nanos / 1_000_000_000;
    let subsec_nanos = nanos % 1_000_000_000;
    if secs > u64::MAX as u128 {
        return None;
    }
    UNIX_EPOCH.checked_add(Duration::new(secs as u64, subsec_nanos as u32))
}

fn load_size_cache(
    socket_dir: &Path,
) -> (HashMap<PathBuf, u64>, HashMap<PathBuf, Option<SystemTime>>) {
    let path = size_cache_path(socket_dir);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(persisted) = serde_json::from_str::<PersistedSizeCache>(&data) {
            let sizes: HashMap<PathBuf, u64> = persisted
                .sizes
                .into_iter()
                .map(|(k, v)| (PathBuf::from(k), v))
                .collect();
            let mtimes: HashMap<PathBuf, Option<SystemTime>> = persisted
                .mtimes
                .into_iter()
                .map(|(k, v)| (PathBuf::from(k), v.and_then(nanos_to_system_time)))
                .collect();
            tracing::info!("Loaded {} cached directory sizes from disk", sizes.len());
            return (sizes, mtimes);
        }

        if let Ok(map) = serde_json::from_str::<HashMap<String, u64>>(&data) {
            let sizes: HashMap<PathBuf, u64> = map
                .into_iter()
                .map(|(k, v)| (PathBuf::from(k), v))
                .collect();
            tracing::info!("Loaded {} cached directory sizes from disk", sizes.len());
            return (sizes, HashMap::new());
        }
    }
    (HashMap::new(), HashMap::new())
}

fn save_size_cache(
    socket_dir: &Path,
    sizes: &HashMap<PathBuf, u64>,
    mtimes: &HashMap<PathBuf, Option<SystemTime>>,
) {
    let path = size_cache_path(socket_dir);
    let persisted = PersistedSizeCache {
        sizes: sizes
            .iter()
            .map(|(k, v)| (k.to_string_lossy().to_string(), *v))
            .collect(),
        mtimes: mtimes
            .iter()
            .map(|(k, v)| (k.to_string_lossy().to_string(), system_time_to_nanos(*v)))
            .collect(),
    };
    if let Ok(data) = serde_json::to_string(&persisted) {
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
    fn test_should_skip_dir() {
        // Agent/tool directories should be skipped
        assert!(should_skip_dir(Path::new("/home/user/project/.pi")));
        assert!(should_skip_dir(Path::new("/home/user/project/.opencode")));
        assert!(should_skip_dir(Path::new("/home/user/project/.claude")));
        assert!(should_skip_dir(Path::new("/home/user/project/.cursor")));
        // Build artifacts should be skipped
        assert!(should_skip_dir(Path::new("/home/user/project/target")));
        assert!(should_skip_dir(Path::new(
            "/home/user/project/node_modules"
        )));
        assert!(should_skip_dir(Path::new("/home/user/project/dist")));
        assert!(should_skip_dir(Path::new("/home/user/project/build")));
        assert!(should_skip_dir(Path::new("/home/user/project/.git")));
        // Source directories should NOT be skipped
        assert!(!should_skip_dir(Path::new("/home/user/project/src")));
        assert!(!should_skip_dir(Path::new("/home/user/project/tests")));
        assert!(!should_skip_dir(Path::new("/home/user/project/docs")));
    }

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
        let size = compute_dir_size_with_status(tmp.path(), Duration::from_secs(5)).size;
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
    fn test_cached_placeholder_dir_size_is_retried() {
        let tmp = tempfile::tempdir().unwrap();
        let child = tmp.path().join("child");
        std::fs::create_dir(&child).unwrap();
        std::fs::write(child.join("payload.bin"), vec![7; 100_000]).unwrap();

        let mut items = vec![DirEntry {
            name: "child".to_string(),
            path: child.clone(),
            is_dir: true,
            is_file: false,
            is_symlink: false,
            is_exec: true,
            size: 4096,
            modified: None,
            perms: String::new(),
            owner: String::new(),
            group: String::new(),
            symlink_target: None,
            symlink_valid: true,
            content_probe: None,
        }];
        let dir_sizes = Arc::new(Mutex::new(HashMap::new()));
        let dir_size_mtimes = Arc::new(Mutex::new(HashMap::new()));
        let placeholder_size = std::fs::metadata(&child).unwrap().len();
        dir_sizes
            .lock()
            .unwrap()
            .insert(child.clone(), placeholder_size);
        dir_size_mtimes
            .lock()
            .unwrap()
            .insert(child.clone(), current_dir_mtime(&child));

        refresh_displayed_dir_sizes(
            &mut items,
            &dir_sizes,
            &dir_size_mtimes,
            Duration::from_secs(5),
        );

        assert!(items[0].size > placeholder_size);
    }

    #[test]
    fn test_compute_dir_size_reports_fallback_as_unmeasured() {
        let computed = compute_dir_size_with_status(
            Path::new("/tmp/definitely-missing-folder-auto-banner-dir"),
            Duration::from_millis(1),
        );
        assert_eq!(computed.size, 0);
        assert!(!computed.measured);
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
            content_probe: None,
        }];
        let dir_sizes = Arc::new(Mutex::new(HashMap::new()));
        let dir_size_mtimes = Arc::new(Mutex::new(HashMap::new()));

        refresh_displayed_dir_sizes(
            &mut items,
            &dir_sizes,
            &dir_size_mtimes,
            Duration::from_secs(5),
        );
        let first_size = items[0].size;
        assert!(first_size > 0);

        std::thread::sleep(Duration::from_millis(20));
        std::fs::write(child.join("two.txt"), "two").unwrap();
        refresh_displayed_dir_sizes(
            &mut items,
            &dir_sizes,
            &dir_size_mtimes,
            Duration::from_secs(5),
        );

        assert!(items[0].size > first_size);
    }
}
