use anyhow::Result;
#[cfg(target_os = "linux")]
use inotify::{Inotify, WatchMask};
use std::collections::{HashMap, HashSet};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use folder_auto_banner::daemon_types::{
    checked_frame_len, BannerData, Request, Response, MAX_IPC_FRAME_SIZE,
};
#[cfg(test)]
use folder_auto_banner::fs::ProjectType;
use folder_auto_banner::fs::{DirEntry, DirSummary};

// Cache entry with TTL
#[derive(Clone)]
struct CacheEntry {
    data: BannerData,
    computed_at: Instant,
    root_mtime: Option<SystemTime>,
    config_mtime: Option<SystemTime>,
    /// `false` when this entry came from a fast pass that skipped the expensive
    /// enrichment (TODO/code-metrics aggregation, port detection). The listing
    /// is complete either way; this only records that the extra context has not
    /// been filled in yet.
    enriched: bool,
}

/// Which features a compute pass should include.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ComputeMode {
    /// Listing, git, and cheap per-entry metadata only. Bounded to
    /// milliseconds no matter how large the tree below is.
    Fast,
    /// Everything, including the recursive TODO/code-metrics walk and port
    /// detection. On a container directory such as `~/Dev` these cost ~1.5s and
    /// ~0.5s, which is exactly why they are kept off the request path.
    Full,
}

const CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes
const BACKGROUND_SIZE_CACHE_REFRESH_TIMEOUT: Duration = Duration::from_secs(5);
const ACTIVE_SIZE_REFRESH_TIMEOUT: Duration = Duration::from_secs(10);
const ACTIVE_SIZE_REFRESH_INTERVAL: Duration = Duration::from_secs(30);
const ACTIVE_SIZE_REFRESH_ROOTS_PER_TICK: usize = 5;
const MAX_SIZE_COMPUTE_THREADS: usize = 16;
/// Hard ceiling on wall clock for one directory size measurement. A size is a
/// display detail; it must never be able to hold up a shell prompt.
const SIZE_SAMPLE_TIME_BUDGET: Duration = Duration::from_millis(120);
/// Hard ceiling on file entries summed during one size measurement.
const SIZE_SAMPLE_FILE_BUDGET: usize = 20_000;
/// Hard ceiling on directories `read_dir`'d during one size measurement.
const SIZE_SAMPLE_DIR_BUDGET: usize = 4_000;
const SOCKET_NAME: &str = "fabd.sock";
const IDLE_TIMEOUT: Duration = Duration::from_secs(600); // 10 minutes
#[cfg(target_os = "linux")]
const WATCH_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
#[cfg(target_os = "linux")]
const ACTIVE_WATCH_DEPTH: usize = 3;
#[cfg(target_os = "linux")]
const MAX_ACTIVE_WATCH_DIRS: usize = 2048;
#[cfg(target_os = "linux")]
const MAX_WATCH_CHILDREN_PER_DIR: usize = 500;
/// Maximum speculative `Warm` computations running concurrently. Warm work is
/// optional, so saturating this budget drops new warm requests rather than
/// queueing them: a dropped warm only defers a computation to the next visit.
const MAX_CONCURRENT_WARM: usize = 2;

/// RAII guard that releases a `Warm` concurrency slot on drop, including on
/// panic, so a failed warm cannot permanently leak capacity.
struct WarmSlotGuard(std::sync::Arc<std::sync::atomic::AtomicUsize>);

impl Drop for WarmSlotGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

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

#[cfg(target_os = "linux")]
#[derive(Clone, Debug)]
struct WatchRegistration {
    owner: PathBuf,
    watched_path: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SizeComputation {
    /// Size in bytes. Exact only when `measured`; otherwise a true lower
    /// bound over the portion of the tree that was sampled.
    size: u64,
    /// The whole subtree was walked within budget, so `size` is exact.
    measured: bool,
    /// The budget ran out first, so `size` is a lower bound. The UI renders
    /// these with a `≥` prefix instead of implying a precise total.
    sampled: bool,
}

type SizeComputeResult = (usize, u64, Option<SystemTime>, bool, bool);

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
    dir_size_sampled: Arc<Mutex<HashSet<PathBuf>>>,
    /// Bounds concurrent speculative `Warm` computations process-wide.
    warm_in_flight: Arc<std::sync::atomic::AtomicUsize>,
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
    /// Paths whose cached size is a sampled lower bound, not an exact total.
    /// Drives the `≥` marker in the UI. Deliberately not persisted: after a
    /// daemon restart the set is empty, so a previously sampled value is shown
    /// without its marker until the next refresh re-labels it. Showing a
    /// correct lower bound unlabelled is a cosmetic miss, never a wrong number.
    dir_size_sampled: Arc<Mutex<HashSet<PathBuf>>>,
    pending_size_refreshes: Arc<Mutex<Vec<PathBuf>>>,
    size_refresh_in_flight: Arc<Mutex<HashSet<PathBuf>>>,
    socket_path: PathBuf,
}

impl Daemon {
    fn new() -> Result<Self> {
        let socket_dir = folder_auto_banner::state::get_data_dir()?;

        let socket_path = socket_dir.join(SOCKET_NAME);

        // If a socket file exists, check whether a live daemon is behind it.
        // Blindly unlinking would orphan a running daemon's listener and let
        // two daemons run concurrently (split-brain caches + duplicate
        // inotify watchers).
        if socket_path.exists() {
            // Timeout (not blocking) connect: a wedged predecessor holding
            // the socket must not wedge this startup forever.
            if folder_auto_banner::daemon_client::connect_with_timeout(&socket_path).is_ok() {
                anyhow::bail!(
                    "daemon already running (socket {} is live)",
                    socket_path.display()
                );
            }
            // Stale socket from a crashed daemon — safe to remove.
            std::fs::remove_file(&socket_path)?;
        }

        // Load persistent size cache from disk, including mtimes so cached sizes can be
        // validated without recomputing every directory on daemon restart.
        let (dir_sizes, dir_size_mtimes) = load_size_cache(&socket_dir);

        Ok(Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            dir_sizes: Arc::new(Mutex::new(dir_sizes)),
            dir_size_mtimes: Arc::new(Mutex::new(dir_size_mtimes)),
            // Not persisted: an empty set only means previously sampled sizes
            // render without their `≥` marker until the next refresh relabels
            // them. The underlying number is still a correct lower bound.
            dir_size_sampled: Arc::new(Mutex::new(HashSet::new())),
            pending_size_refreshes: Arc::new(Mutex::new(Vec::new())),
            size_refresh_in_flight: Arc::new(Mutex::new(HashSet::new())),
            socket_path,
        })
    }

    fn run(&self) -> Result<()> {
        let listener = UnixListener::bind(&self.socket_path)?;
        listener.set_nonblocking(true)?;

        // Restrict the IPC socket to this user. The default bind mode
        // (0777 & umask) makes it world-connectable, and the socket carries
        // no authentication — any local user could query or shut down the
        // daemon.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ =
                std::fs::set_permissions(&self.socket_path, std::fs::Permissions::from_mode(0o600));
        }

        tracing::info!("fabd listening on {}", self.socket_path.display());

        // Start inotify watcher thread for active folders only.
        let cache_clone = self.cache.clone();
        let dir_sizes_clone = self.dir_sizes.clone();
        let dir_size_mtimes_clone = self.dir_size_mtimes.clone();
        let dir_size_sampled_clone = self.dir_size_sampled.clone();
        let active_roots = Arc::new(Mutex::new(HashSet::new()));
        let active_order = Arc::new(Mutex::new(Vec::new()));
        // The watcher thread is Linux-only (gated below). On other targets
        // the clones below are unused; gate the clones to the same target
        // rather than `let _ = ...` them, so the request-handler path that
        // calls `touch_active_root(&active_roots, ...)` keeps a clean view of
        // which threads share these arcs.
        #[cfg(target_os = "linux")]
        let (active_roots_clone, active_order_clone) = (active_roots.clone(), active_order.clone());
        #[cfg(target_os = "linux")]
        let _watcher_handle = thread::spawn(move || {
            watch_loop(
                cache_clone,
                dir_sizes_clone,
                dir_size_mtimes_clone,
                dir_size_sampled_clone,
                active_roots_clone,
                active_order_clone,
            );
        });
        // On non-Linux the daemon has no inotify watcher. The cache still
        // invalidates on the mtime checks in `cache_entry_is_fresh` and
        // `cached_dir_size_is_fresh`, so staleness is bounded by the 5-minute
        // banner TTL rather than by filesystem events. For Linux-only
        // desktop use this is a no-op gate; for a future macOS port this is
        // the line that becomes "FSEvents or equivalent".
        #[cfg(not(target_os = "linux"))]
        let _ = (
            cache_clone,
            dir_sizes_clone,
            dir_size_mtimes_clone,
            dir_size_sampled_clone,
        );

        // Load persisted banner cache after the watcher is ready so watched paths become
        // active immediately. Persisted entries are intentionally left in the cache for
        // fast startup, but active-folder watchers and a cheap root-mtime check catch
        // changes without forcing a full shallow scan on every cache hit.
        let socket_dir = folder_auto_banner::state::get_data_dir().ok();
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
                        config_mtime: current_config_mtime(),
                        // Persisted entries were always written from a full
                        // compute, so their enrichment is already present.
                        enriched: true,
                    },
                );
            }
            tracing::info!("Loaded {} banner caches from disk", cache.len());
        }

        let size_refresh_ctx = Arc::new(SizeRefreshContext {
            cache: self.cache.clone(),
            dir_sizes: self.dir_sizes.clone(),
            dir_size_mtimes: self.dir_size_mtimes.clone(),
            dir_size_sampled: self.dir_size_sampled.clone(),
            warm_in_flight: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
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
#[cfg(target_os = "linux")]
fn watch_loop(
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    dir_sizes: Arc<Mutex<HashMap<PathBuf, u64>>>,
    dir_size_mtimes: Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
    dir_size_sampled: Arc<Mutex<HashSet<PathBuf>>>,
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
                                    folder_auto_banner::cmd::banner_data_cache::
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
                                    prune_size_cache_for_root(
                                        &dir_sizes,
                                        &dir_size_mtimes,
                                        &dir_size_sampled,
                                        path,
                                    );
                                    // The client's fast path trusts the on-disk
                                    // banner cache file's mtime; a stale file
                                    // would keep serving the pre-event banner,
                                    // so drop it together with the memory entry.
                                    folder_auto_banner::cmd::banner_data_cache::remove_cache(path);
                                    tracing::info!("Cache invalidated: {}", path.display());
                                }
                            }
                        } else {
                            for path in &invalidated {
                                prune_size_cache_for_root(
                                    &dir_sizes,
                                    &dir_size_mtimes,
                                    &dir_size_sampled,
                                    path,
                                );
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

#[cfg(target_os = "linux")]
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
        {
            continue;
        }

        if !can_watch_path(&target) {
            // Failed paths are retried on the next refresh pass (which runs
            // whenever the active root set changes) instead of being skipped
            // forever — a transient failure (inotify limit, path not yet
            // created, permissions) would otherwise never recover.
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
                failed_watches.remove(&target);
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

#[cfg(target_os = "linux")]
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

    // Do not follow symlinks while building the watcher tree. A link may point
    // outside the requested project or back into an ancestor, which can cause
    // unrelated invalidations, duplicate watches, or recursive traversal.
    if meta.is_symlink() {
        return;
    }

    let is_dir = meta.is_dir();

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

    // A directory may contain millions of entries that are skipped or cannot
    // be watched. Bound the raw iterator as well as the global target count so
    // watcher refresh never performs an unbounded scan.
    for entry in entries.take(MAX_WATCH_CHILDREN_PER_DIR).flatten() {
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

#[cfg(target_os = "linux")]
fn can_watch_path(path: &Path) -> bool {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(_) => return false,
    };

    !meta.is_symlink() && (meta.is_file() || meta.is_dir())
}

#[cfg(target_os = "linux")]
fn find_owner_for_watch(path: &Path, active_roots: &HashSet<PathBuf>) -> PathBuf {
    let roots = active_roots;

    roots
        .iter()
        .filter(|root| path == root.as_path() || path.starts_with(root.as_path()))
        .max_by_key(|root| root.components().count())
        .cloned()
        .unwrap_or_else(|| path.to_path_buf())
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
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
    use std::io::Read;

    let t_start = std::time::Instant::now();
    // Length-prefixed JSON: read 4-byte LE length, then payload.
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let t_read = std::time::Instant::now();
    let req_len = u32::from_le_bytes(len_bytes) as usize;
    if req_len > MAX_IPC_FRAME_SIZE {
        send_response(
            &mut stream,
            &Response::Error {
                message: "IPC request exceeds the maximum frame size".to_string(),
            },
        )?;
        return Ok(());
    }
    let mut req_buf = vec![0u8; req_len];
    stream.read_exact(&mut req_buf)?;
    let request: Request = serde_json::from_slice(&req_buf)?;
    let t_parse = std::time::Instant::now();
    if std::env::var("FAB_PROFILE").is_ok() {
        // Which requests arrive, and for what. A single `f` invocation in a
        // container directory was fanning out into ~13 warm requests, which is
        // invisible without this trace.
        eprintln!("[FAB_PROFILE_REQ] {:?}", request);
    }
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
                    .filter(|entry| cache_entry_is_fresh(entry, &path))
                    .cloned()
            };

            if let Some(entry) = cached_entry {
                let t0 = std::time::Instant::now();
                let root_fresh = cache_entry_root_is_fresh(&entry, &path);
                // The cache filter above already guarantees the entry is
                // unexpired, so only the root-mtime check is needed here.
                let mut data = entry.data;
                let t1 = std::time::Instant::now();
                let _t_root_done = t1;
                if !root_fresh {
                    tracing::debug!(
                        "Cache recompute: path={} root_fresh={} root_mtime={:?} current_mtime={:?}",
                        path.display(),
                        root_fresh,
                        entry.root_mtime,
                        current_dir_mtime(&path),
                    );
                    // Re-compute and replace the outer `data` so the
                    // response below (and the disk cache write) reflects
                    // the freshly-computed banner, not the stale entry.
                    // (Pre-0.6.27 the inner `let data =` shadowed the
                    // outer `data` and the response used the old data.)
                    data = match compute_banner_data(&path, ComputeMode::Fast) {
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
                            config_mtime: current_config_mtime(),
                            enriched: false,
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
                // A root-mtime change invalidates the entry, so the fast pass
                // above replaced an enriched banner with a bare one. Queue the
                // enrichment again, or this path would show a listing with no
                // TODO/line/port context until its next cold miss.
                schedule_enrichment_refresh(
                    &cache,
                    &active_roots,
                    &active_order,
                    &size_refresh_ctx,
                    path.clone(),
                );
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
                return Ok(());
            }

            // Cache miss — serve the listing immediately.
            //
            // The expensive enrichment (recursive TODO/code-metrics walk,
            // port detection) is deliberately *not* on this path. On a
            // container directory like `~/Dev` those two phases cost ~1.5s
            // and ~0.5s, so putting them here meant the first `cd` after a
            // cache expiry blocked for seconds. A fast pass returns the
            // complete listing plus git in milliseconds; enrichment lands in
            // the cache a moment later and the next visit shows it.
            let mut data = match compute_banner_data(&path, ComputeMode::Fast) {
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
                        config_mtime: current_config_mtime(),
                        enriched: false,
                    },
                );
                touch_active_root(&active_roots, &active_order, path.clone());
            }
            // Persist the banner data to the per-path on-disk cache so
            // the client can skip the IPC round-trip on the next call
            // (the IPC `read4` has a 1–10 ms kernel-scheduling floor).
            persist_banner_data_cache(&path, &data);
            schedule_size_refresh(
                size_refresh_ctx.clone(),
                path.clone(),
                data.clone(),
                BACKGROUND_SIZE_CACHE_REFRESH_TIMEOUT,
            );
            schedule_enrichment_refresh(
                &cache,
                &active_roots,
                &active_order,
                &size_refresh_ctx,
                path.clone(),
            );
            Response::Banner(Box::new(data))
        }
        Request::Warm { path } => {
            let path = path.canonicalize().unwrap_or_else(|_| path.clone());
            let cache = cache.clone();
            let active_order = active_order.clone();
            // Pre-compute in background — don't block the client.
            //
            // Warm work is strictly speculative: a warm miss only means the
            // directory gets computed on demand when it is next visited. So
            // bound how much of it can run at once. Without this bound, one
            // `f` invocation in a container directory fires ~13 warm requests
            // (parent, grandparent, every child), each running its own ~1s
            // `scan_insights` tree walk concurrently with the single banner
            // the user actually asked for. That saturated I/O and made the
            // requested directory take 1.4s instead of ~50ms — the warm path
            // was actively making the real request slower.
            if size_refresh_ctx
                .warm_in_flight
                .fetch_update(
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                    |n| (n < MAX_CONCURRENT_WARM).then_some(n + 1),
                )
                .is_err()
            {
                tracing::debug!(
                    "Dropping warm request for {}: {} warm tasks already in flight",
                    path.display(),
                    MAX_CONCURRENT_WARM
                );
                return Ok(());
            }
            let warm_slot = WarmSlotGuard(size_refresh_ctx.warm_in_flight.clone());
            thread::spawn(move || {
                let _warm_slot = warm_slot;
                let cache_hit = {
                    let c = cache.lock().unwrap_or_else(|e| {
                        tracing::warn!("Mutex poisoned, recovering");
                        e.into_inner()
                    });
                    c.get(&path)
                        .map(|e| cache_entry_is_fresh(e, &path))
                        .unwrap_or(false)
                };
                if !cache_hit {
                    // Warm work is off the critical path, so it can afford the
                    // full pass and lands the result already enriched.
                    match compute_banner_data(&path, ComputeMode::Full) {
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
                                    config_mtime: current_config_mtime(),
                                    enriched: true,
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
        Request::Ping => Response::Pong,
        Request::Shutdown => {
            tracing::info!("Shutdown requested");
            #[cfg(unix)]
            SHUTDOWN_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);
            Response::Pong
        }
    };

    send_response(&mut stream, &response)?;
    tracing::trace!("Sent response successfully");

    // The client has its own length-prefixed read: it knows exactly when the
    // response ends and will close the socket (drop UnixStream) immediately
    // after. A drain loop here would just wait for the client's close, which
    // is bounded by the read timeout (5s) and adds 5s of latency per request
    // for nothing. Dropping `stream` returns the kernel-side FD to the OS.
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
    let resp_len = checked_frame_len(resp_bytes.len())
        .ok_or_else(|| anyhow::anyhow!("IPC response exceeds the maximum frame size"))?;
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

/// Fill in the expensive enrichment for a path in the background, off the
/// request path.
///
/// After a fast pass has answered the user, this recomputes the TODO /
/// code-metrics aggregation and port detection, then replaces the cache entry
/// and rewrites the on-disk cache file — so the next `cd` shows the full
/// banner. Mirrors the existing directory-size refresh: the prompt never
/// waits, and the enriched value is picked up on the following visit.
///
/// Skipped entirely when the entry is already enriched, so a warm path only
/// ever does this work once.
fn schedule_enrichment_refresh(
    cache: &Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    active_roots: &Arc<Mutex<HashSet<PathBuf>>>,
    active_order: &Arc<Mutex<Vec<PathBuf>>>,
    size_refresh_ctx: &Arc<SizeRefreshContext>,
    path: PathBuf,
) {
    // Already enriched: nothing to do.
    {
        let c = cache.lock().unwrap_or_else(|e| {
            tracing::warn!("Mutex poisoned, recovering");
            e.into_inner()
        });
        match c.get(&path) {
            Some(entry) if entry.enriched => return,
            // The directory changed while we were deciding; its own recompute
            // path will handle enrichment.
            Some(entry) if entry.root_mtime != current_dir_mtime(&path) => return,
            None => return,
            _ => {}
        }
    }

    let cache = cache.clone();
    let active_roots = active_roots.clone();
    let active_order = active_order.clone();
    let size_refresh_ctx = size_refresh_ctx.clone();
    thread::spawn(move || {
        let Ok(mut enriched) = compute_banner_data(&path, ComputeMode::Full) else {
            return;
        };
        apply_cached_displayed_dir_sizes(
            &mut enriched.summary.top_items,
            &size_refresh_ctx.dir_sizes,
            &size_refresh_ctx.dir_size_mtimes,
        );
        {
            let mut c = cache.lock().unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned, recovering");
                e.into_inner()
            });
            c.insert(
                path.clone(),
                CacheEntry {
                    data: enriched.clone(),
                    computed_at: Instant::now(),
                    root_mtime: current_dir_mtime(&path),
                    config_mtime: current_config_mtime(),
                    enriched: true,
                },
            );
        }
        // Rewrite the disk cache so the client's fast path picks up the
        // enriched banner without another IPC round trip.
        let _ = folder_auto_banner::cmd::banner_data_cache::write_cache(&path, &enriched);
        touch_active_root(&active_roots, &active_order, path.clone());
        // Sizes may also have filled in during the full pass.
        schedule_size_refresh(
            size_refresh_ctx,
            path,
            enriched,
            BACKGROUND_SIZE_CACHE_REFRESH_TIMEOUT,
        );
    });
}

fn compute_banner_data(path: &Path, mode: ComputeMode) -> Result<BannerData> {
    let __t_all = std::time::Instant::now();
    let config = folder_auto_banner::state::Config::load().unwrap_or_default();
    let extra_skip_dirs: Vec<&str> = config.ignore_dirs.iter().map(String::as_str).collect();
    let enrich = mode == ComputeMode::Full;
    let mut summary = DirSummary::scan_with_options(
        path,
        // Build checks spawn subprocesses (cargo check ≈ 6.7s) — opt-in via
        // `f config` (build_status), off by default. Never on the fast path.
        config.build_status && enrich,
        enrich
            && folder_auto_banner::utils::feature_enabled(
                config.todo_count,
                "FAB_TODOS",
                "FAB_NO_TODOS",
            ),
        enrich
            && folder_auto_banner::utils::feature_enabled(
                config.ports,
                "FAB_PORTS",
                "FAB_NO_PORTS",
            ),
        enrich
            && folder_auto_banner::utils::feature_enabled(
                config.docker,
                "FAB_DOCKER",
                "FAB_NO_DOCKER",
            ),
        enrich
            && folder_auto_banner::utils::feature_enabled(
                config.languages,
                "FAB_METRICS",
                "FAB_NO_METRICS",
            ),
        &extra_skip_dirs,
    )?;

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
    if config.git_status {
        if let Some(ref cache) = cache {
            let ck = folder_auto_banner::cache::cache_key(path, "git");
            if let Some(cached) = cache.get(&ck, std::time::Duration::from_secs(60)) {
                git_info = Some(cached);
            }
        }
    }
    if config.git_status && git_info.is_none() {
        let __t_git = std::time::Instant::now();
        git_info =
            folder_auto_banner::git::get_git_info_filtered_with(path, &filter_paths, enrich).ok();
        if std::env::var("FAB_PROFILE").is_ok() {
            eprintln!(
                "[FAB_PROFILE_GIT] {} rich={} took={:?}",
                path.display(),
                enrich,
                __t_git.elapsed()
            );
        }
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
            let ck = folder_auto_banner::cache::cache_key(path, "git");
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

    if std::env::var("FAB_PROFILE").is_ok() {
        eprintln!(
            "[FAB_PROFILE_TOTAL] {:?} {mode:?} total={:?}",
            path,
            __t_all.elapsed()
        );
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

#[cfg(target_os = "linux")]
fn prune_size_cache_for_root(
    dir_sizes: &Arc<Mutex<HashMap<PathBuf, u64>>>,
    dir_size_mtimes: &Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
    dir_size_sampled: &Arc<Mutex<HashSet<PathBuf>>>,
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

    dir_size_sampled
        .lock()
        .unwrap_or_else(|e| {
            tracing::warn!("Mutex poisoned, recovering");
            e.into_inner()
        })
        .retain(|path| path != root && !path.starts_with(root));
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
                    .filter(|entry| cache_entry_is_fresh(entry, &path))
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
            &ctx.dir_size_sampled,
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
            // A size refresh only re-lists the directory; it must not clear
            // an enrichment that a background full pass already produced.
            let was_enriched = c.get(&path).map(|entry| entry.enriched).unwrap_or(false);
            c.insert(
                path.clone(),
                CacheEntry {
                    data: refreshed,
                    computed_at,
                    root_mtime: current_dir_mtime(&path),
                    config_mtime: current_config_mtime(),
                    enriched: was_enriched,
                },
            );
            // Persist the refreshed sizes to the per-path disk cache: the
            // client's fast path reads those files and would otherwise keep
            // seeing pre-refresh sizes until the next full request.
            if let Some(entry) = c.get(&path) {
                let _ = folder_auto_banner::cmd::banner_data_cache::write_cache(&path, &entry.data);
            }
            touch_active_root(&ctx.active_roots, &ctx.active_order, path);
        }
    });
}

fn refresh_displayed_dir_sizes(
    items: &mut [DirEntry],
    dir_sizes: &Arc<Mutex<HashMap<PathBuf, u64>>>,
    dir_size_mtimes: &Arc<Mutex<HashMap<PathBuf, Option<SystemTime>>>>,
    dir_size_sampled: &Arc<Mutex<HashSet<PathBuf>>>,
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
        let sampled_paths = dir_size_sampled.lock().unwrap_or_else(|e| {
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
                    item.size_is_estimate = sampled_paths.contains(&item.path);
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
    let mut sampled_paths = dir_size_sampled.lock().unwrap_or_else(|e| {
        tracing::warn!("Mutex poisoned, recovering");
        e.into_inner()
    });
    for (idx, size, mtime_opt, measured, sampled) in results {
        let path = items[idx].path.clone();
        sizes.insert(path.clone(), size);
        if measured || sampled {
            // A sampled lower bound is still worth caching. The old code
            // cleared the mtime whenever a value was not an exact `du` total,
            // which made the value permanently uncacheable, so the daemon
            // re-walked the same huge tree on every refresh tick forever.
            // Recording the mtime lets the value be reused until the directory
            // actually changes, and re-sampled only then.
            if let Some(mt) = mtime_opt {
                mtimes.insert(path.clone(), Some(mt));
            }
        } else {
            mtimes.insert(path.clone(), None);
        }
        if sampled {
            sampled_paths.insert(path);
        } else {
            sampled_paths.remove(&path);
        }
        items[idx].size = size;
        items[idx].size_is_estimate = sampled;
    }
    drop(sizes);
    drop(mtimes);
    drop(sampled_paths);
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
                    r.push((
                        *orig_idx,
                        computed.size,
                        *mtime,
                        computed.measured,
                        computed.sampled,
                    ));
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

fn current_config_mtime() -> Option<SystemTime> {
    folder_auto_banner::state::Config::config_path()
        .ok()
        .and_then(|path| std::fs::metadata(path).ok())
        .and_then(|metadata| metadata.modified().ok())
}

fn cache_entry_is_fresh(entry: &CacheEntry, path: &Path) -> bool {
    entry.computed_at.elapsed() < CACHE_TTL
        && entry.root_mtime == current_dir_mtime(path)
        && entry.config_mtime == current_config_mtime()
}

/// Measure a directory's size with a bounded breadth-first walk.
///
/// This used to shell out to `du -s -b`, which cannot meet a prompt-time
/// budget on a real workspace tree: `du -s -b ~/Dev` takes 106s and
/// `~/Dev/dracon-platform` (222 GB) takes 56s, because the tree holds
/// 1.19M files. Every call therefore hit the timeout, returned the 4 KiB
/// directory-inode placeholder, and — since a placeholder is not
/// authoritative — was never cached. The daemon re-ran it forever
/// (5 concurrent `du` at all times, 5–10s per `cd`) and every directory in
/// the `~/Dev` banner rendered as `4.0k`.
///
/// No cache TTL can fix a 106-second primitive, so the primitive changed.
/// The walk is breadth-first, so the sample is spread across the top of the
/// tree rather than being consumed by one deep subtree, and it is bounded by
/// file count, directory count, *and* wall clock. Directories small enough to
/// finish stay exact; large ones return a lower bound in milliseconds,
/// flagged `sampled` so the UI can show `≥` instead of implying precision the
/// measurement does not have.
///
/// The returned `size` is always a true lower bound on the real total: it sums
/// only bytes actually observed, and is only exact when the whole subtree was
/// walked. Extrapolating a sample up to an estimated total was rejected: on a
/// build-artifact tree the sample is dominated by whichever subtree fills the
/// budget first, so a scaled-up number would be a fabrication presented as a
/// measurement.
fn compute_dir_size_with_status(path: &Path, timeout: Duration) -> SizeComputation {
    // Honour the caller's budget, but never spend more than the hard cap:
    // a size estimate is a display detail and must not hold up a prompt.
    let budget = timeout.min(SIZE_SAMPLE_TIME_BUDGET);
    let deadline = Instant::now() + budget;

    // A path we cannot even read is not a measurement, and is not a sample
    // either — there is no partial result to report. Fall back to the inode
    // size, matching the old `du`-timeout path, and stay uncacheable so a
    // later retry can pick the directory up once it exists.
    if let Err(e) = std::fs::read_dir(path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::debug!("Size sample could not read {}: {e}", path.display());
        }
        return SizeComputation {
            size: std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
            measured: false,
            sampled: false,
        };
    }

    let mut queue: std::collections::VecDeque<PathBuf> = std::collections::VecDeque::new();
    queue.push_back(path.to_path_buf());

    let mut size: u64 = 0;
    let mut files = 0usize;
    let mut dirs = 0usize;
    let mut truncated = false;

    while let Some(dir) = queue.pop_front() {
        if files >= SIZE_SAMPLE_FILE_BUDGET || dirs >= SIZE_SAMPLE_DIR_BUDGET {
            truncated = true;
            break;
        }
        dirs += 1;
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            // `file_type()` uses the dirent `d_type` on Linux, so classifying
            // an entry usually costs no syscall. Stat only what we must size.
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            // Never follow symlinks: they can escape the requested tree or
            // loop back into an ancestor, and would double-count.
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                queue.push_back(entry.path());
            } else if file_type.is_file() {
                if let Ok(meta) = entry.metadata() {
                    size = size.saturating_add(meta.len());
                }
                files += 1;
            }
            // Check the clock inside the inner loop too: a single directory
            // holding tens of thousands of entries would otherwise blow the
            // whole budget before returning to the outer check.
            if files >= SIZE_SAMPLE_FILE_BUDGET || Instant::now() >= deadline {
                truncated = true;
                break;
            }
        }
        if truncated {
            break;
        }
    }

    // An empty directory really is 0 bytes. Reporting the inode size there
    // would be a fabricated non-zero total, so only fall back when nothing at
    // all was observed on a readable directory.
    let size = if size == 0 {
        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else {
        size
    };
    let fully_walked = !truncated;
    SizeComputation {
        size,
        measured: fully_walked,
        sampled: !fully_walked,
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
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("fabd {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "fabd {}\nBackground daemon for folder-auto-banner\n\nUsage: fabd [OPTIONS]\n\nOptions:\n  -h, --help     Print help\n  -V, --version  Print version",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }

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
            config_mtime: None,
            enriched: true,
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
            config_mtime: None,
            enriched: true,
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
            size_is_estimate: false,
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
        let dir_size_sampled = Arc::new(Mutex::new(HashSet::new()));
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
            &dir_size_sampled,
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
        assert!(!computed.sampled);
    }

    #[test]
    fn test_fast_pass_skips_enrichment_and_full_pass_includes_it() {
        // The whole point of ComputeMode: a cache miss must return the listing
        // in milliseconds. `scan_insights` is a recursive tree walk costing
        // ~1.5s on a container directory like ~/Dev, so gating it off the
        // request path is what keeps the first `cd` fast.
        let tmp = tempfile::tempdir().unwrap();
        // A manifest so the directory is detected as a project: insights are
        // deliberately skipped for Generic directories.
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        std::fs::create_dir(tmp.path().join("src")).unwrap();
        std::fs::write(
            tmp.path().join("src/main.rs"),
            "// TODO: something\nfn main() {}\n",
        )
        .unwrap();

        let fast = compute_banner_data(tmp.path(), ComputeMode::Fast).unwrap();
        // The listing is complete either way — that must not regress.
        assert!(
            !fast.summary.top_items.is_empty(),
            "fast pass must still list the directory"
        );
        assert!(
            fast.summary.todo_info.is_none(),
            "fast pass must not run the TODO/metrics walk"
        );
        assert!(
            fast.summary.code_metrics.is_none(),
            "fast pass must not compute code metrics"
        );

        let full = compute_banner_data(tmp.path(), ComputeMode::Full).unwrap();
        assert_eq!(
            full.summary.todo_info.as_ref().map(|t| t.count),
            Some(1),
            "full pass must still find the TODO"
        );
        assert!(
            full.summary.code_metrics.is_some(),
            "full pass must still compute code metrics"
        );
    }

    #[test]
    fn test_compute_dir_size_small_tree_is_exact() {
        // A tree the budget can finish must stay exact and must NOT be marked
        // as an estimate, otherwise every small directory would render with a
        // misleading `≥` prefix.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("sub")).unwrap();
        std::fs::write(tmp.path().join("a.txt"), "0123456789").unwrap();
        std::fs::write(tmp.path().join("sub/b.txt"), "0123456789").unwrap();

        let computed = compute_dir_size_with_status(tmp.path(), Duration::from_secs(5));
        assert!(computed.measured, "small tree should be measured exactly");
        assert!(!computed.sampled, "small tree should not be a sample");
        assert!(
            computed.size >= 20,
            "should observe both 10-byte files, got {}",
            computed.size
        );
    }

    #[test]
    fn test_compute_dir_size_is_bounded_by_its_budget() {
        // The whole point of the sampler: a wide tree must return fast rather
        // than shelling out to `du`, which took 106s for ~/Dev. Assert the
        // wall-clock bound, not the exact number.
        let tmp = tempfile::tempdir().unwrap();
        // Enough entries to blow the entry budget many times over.
        for d in 0..40 {
            let sub = tmp.path().join(format!("d{d}"));
            std::fs::create_dir(&sub).unwrap();
            for f in 0..1_000 {
                std::fs::write(sub.join(format!("f{f}.txt")), "x").unwrap();
            }
        }

        let started = Instant::now();
        let computed = compute_dir_size_with_status(tmp.path(), Duration::from_secs(30));
        let elapsed = started.elapsed();

        // The caller may ask for 30s; the hard cap must still win.
        assert!(
            elapsed < SIZE_SAMPLE_TIME_BUDGET * 3,
            "size sampling must stay bounded, took {elapsed:?}"
        );
        // 40,000 files is well past the 20,000 file budget, so this is a
        // sample and must be reported as one.
        assert!(computed.sampled, "40k files must be reported as a sample");
        assert!(
            !computed.measured,
            "truncated walk must not claim exactness"
        );
        assert!(computed.size > 0, "sample should still observe bytes");
    }

    #[test]
    fn test_compute_dir_size_samples_across_the_tree_not_one_subtree() {
        // Breadth-first matters: a depth-first walk would spend its whole
        // budget inside the first subdirectory and report a number dominated
        // by whichever subtree happens to be widest.
        let tmp = tempfile::tempdir().unwrap();
        let mut expected: u64 = 0;
        for d in 0..8 {
            let sub = tmp.path().join(format!("d{d}"));
            std::fs::create_dir(&sub).unwrap();
            // Give each sibling a distinguishable, equal share.
            for f in 0..4_000 {
                std::fs::write(sub.join(format!("f{f}.txt")), "0123456789").unwrap();
                expected += 10;
            }
        }
        // 32,000 files > the 20,000 file budget, so only a prefix is visited.
        let computed = compute_dir_size_with_status(tmp.path(), Duration::from_secs(30));
        assert!(computed.sampled, "should be a sample");
        assert!(
            computed.size < expected,
            "a truncated sample must not claim the full total"
        );
        // A BFS spread across 8 siblings observes roughly half the tree; a DFS
        // wedged in one sibling would observe ~1/8. Require > 20% of total to
        // prove the sample is spread rather than concentrated.
        assert!(
            computed.size * 5 > expected,
            "sample should span siblings, got {} of {expected}",
            computed.size
        );
    }

    #[test]
    fn test_compute_dir_size_does_not_follow_symlinks() {
        // A symlink back into the tree would double-count and, pointing at an
        // ancestor, could loop forever.
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("a.txt"), "0123456789").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(tmp.path(), sub.join("loop")).unwrap();

        let computed = compute_dir_size_with_status(tmp.path(), Duration::from_secs(5));
        assert!(computed.measured, "should complete without looping");
        assert!(
            computed.size < 1_000_000,
            "symlink loop must not inflate the total, got {}",
            computed.size
        );
    }

    #[test]
    fn test_sampled_sizes_are_cached_and_marked() {
        // Regression guard for the 7s `cd ~/Dev`: a sampled value used to be
        // written with a cleared mtime, which made it permanently uncacheable,
        // so the daemon re-walked the same tree on every refresh forever.
        let tmp = tempfile::tempdir().unwrap();
        let child = tmp.path().join("child");
        std::fs::create_dir(&child).unwrap();
        std::fs::write(child.join("one.txt"), "one").unwrap();

        let mk_item = |path: std::path::PathBuf| DirEntry {
            name: "child".to_string(),
            path,
            is_dir: true,
            is_file: false,
            is_symlink: false,
            is_exec: true,
            size: 0,
            size_is_estimate: false,
            modified: None,
            perms: String::new(),
            owner: String::new(),
            group: String::new(),
            symlink_target: None,
            symlink_valid: true,
            content_probe: None,
        };
        let mut items = vec![mk_item(child.clone())];
        let dir_sizes = Arc::new(Mutex::new(HashMap::new()));
        let dir_size_mtimes = Arc::new(Mutex::new(HashMap::new()));
        let dir_size_sampled = Arc::new(Mutex::new(HashSet::new()));

        refresh_displayed_dir_sizes(
            &mut items,
            &dir_sizes,
            &dir_size_mtimes,
            &dir_size_sampled,
            Duration::from_secs(5),
        );

        // The critical assertion: a mtime was recorded, so the next refresh
        // reuses this value instead of recomputing.
        assert!(
            dir_size_mtimes
                .lock()
                .unwrap()
                .get(&child)
                .copied()
                .flatten()
                .is_some(),
            "computed size must record an mtime so it is cacheable"
        );
        // This small dir completes exactly, so it must not be marked.
        assert!(!items[0].size_is_estimate, "small dir should be exact");

        // Second pass reuses the cache rather than re-walking.
        let cached_size = items[0].size;
        let mut items_again = vec![mk_item(child.clone())];
        refresh_displayed_dir_sizes(
            &mut items_again,
            &dir_sizes,
            &dir_size_mtimes,
            &dir_size_sampled,
            Duration::from_secs(5),
        );
        assert_eq!(items_again[0].size, cached_size);
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
            size_is_estimate: false,
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
        let dir_size_sampled = Arc::new(Mutex::new(HashSet::new()));

        refresh_displayed_dir_sizes(
            &mut items,
            &dir_sizes,
            &dir_size_mtimes,
            &dir_size_sampled,
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
            &dir_size_sampled,
            Duration::from_secs(5),
        );

        assert!(items[0].size > first_size);
    }
}
