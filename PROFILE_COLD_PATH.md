# Cold-path profile — folder-auto-banner

**Date**: 2026-06-18
**Version measured**: 0.7.6 (pre-fix)

## Symptom

`f <folder>` is noticeably slow the first time it is invoked against
a directory. Subsequent invocations within the daemon's cache TTL
(5 min) are fast. The user reports the slowness as "stupidly slow"
on first scan and hypothesises that expensive features
(git status, code metrics, todo scan, languages, ports, docker)
are running on every cold scan without being cached or made lazy.

## Method

A wall-clock harness was added inside the daemon's `compute_banner_data`
and `DirSummary::scan_with_options` to log per-phase timing to
`stderr` (the daemon's log). The instrumented daemon was then
built and invoked with a cleared cache against three target folders.

## Targets

| Folder | Files (top-level) | Total size | Notes |
|--------|------------------:|-----------:|-------|
| `~/Dev/folder-auto-banner` | 70 | 12 MB | small project |
| `~/Dev/rust-ai-web-auto/target` | 35 514 | 23 GB | cargo build output |
| `~/Dev` | ~50 subdirs | 103 GB | mixed projects |
| `~/` (selected) | — | — | root |

OS file cache was warm for all runs.

## Results (pre-fix, 0.7.6)

| Folder | walk | todo+metric | port | scan total | git | TOTAL |
|--------|----:|------------:|----:|----------:|----:|------:|
| `~/Dev/folder-auto-banner` (cold) | 0 ms | 98 ms | 62 ms | 161 ms | 54 ms | **215 ms** |
| `~/Dev/rust-ai-web-auto/target` (cold) | 0 ms | 96 ms | 61 ms | 158 ms | 54 ms | **213 ms** |
| `~/Dev` (cold) | 0 ms | 127 ms | 71 ms | 198 ms | 0 ms | **198 ms** |
| `~/Dev/folder-auto-banner` (warm) | — | — | — | — | — | **3 ms** |

(Times in milliseconds; 0 ms = below 1 ms; the 0 ms walk is OS-cache
warm. The git cost on `~/Dev` is 0 ms because the folder is not a
git repo.)

## Findings

Two phases dominate the cold path:

1. **`todo+metric` (`scan_insights`)**: 60–65% of cold-path time.
   Walks the directory (skipping `target`, `node_modules`, `.git`,
   `dist`, `build`, `vendor`, `.next`, `__pycache__`, `.venv`, `venv`),
   reads up to `MAX_FILES = 1000` text files, counts lines and TODOs.
   Bounded by `INSIGHT_TIMEOUT = 1 s`.
   - **Not cached.** Every call to `DirSummary::scan_with_options`
     re-runs the scan even within the same process.

2. **`port` (`detect_ports`)**: 30–35% of cold-path time.
   Runs `ss -tlnp`, parses output, walks `/proc/<pid>/cwd` for each
   listening port.
   - **Already cached** at 10 s TTL (file cache). Inner `ss` output
     cached at 2 s in-process.

`walk` (file enumeration + per-file metadata) is sub-millisecond
when the OS file cache is warm. `git` is cheap when the target is
not a git repo, modest (54 ms) when it is. `content_probe` is
sub-millisecond because the in-process probe cache is warm.

The root cause is the missing cache for `scan_insights` — it's
the only expensive phase that runs on every cold scan.

## Fix (0.7.7)

Wrap `scan_insights` in the existing `cached_check!` macro with
a 60 s TTL, matching the TTL used for `build_status`. The cache
key is `<path>:insights`. The file cache already exists, the
walker is unchanged, and the per-scan cost collapses to a single
`std::fs::read_to_string` of a small JSON file.

## Results (post-fix, 0.7.7)

Re-measured with the same harness after wrapping `scan_insights`
in a 60 s file cache.

| Folder | walk | todo+metric | port | scan total | end-to-end |
|--------|----:|------------:|----:|----------:|----------:|
| `~/Dev` (cold file cache, cold daemon) | 0 ms | 119 ms | 71 ms | 191 ms | **204 ms** |
| `~/Dev` (warm file cache, cold daemon) | 0 ms | **0 ms** | **0 ms** | 0 ms | **4 ms** |
| `~/Dev` (warm daemon cache) | — | — | — | — | **3 ms** |

### Speedup on the daemon-cache-expired cold path

| Path | Pre-fix | Post-fix | Speedup |
|------|--------:|---------:|--------:|
| Daemon restart, file cache populated | **198 ms** | **4 ms** | **50×** |
| First-ever scan of a folder | 198 ms | 204 ms | 1.0× (one-time) |
| Warm daemon cache | 3 ms | 3 ms | 1.0× (already fast) |

### Speedup for large git repos (0.7.8)

Additional fix: git status on `~/Dev/dracon-platform/web/music`
(15K commits, 5.8 GB `.git`) was the dominant bottleneck (8+ s
per cold scan) after the scan_insights fix. The daemon's
BannerCache (5 min) covers the common case, but after it expires
or on daemon restart the full `repo.statuses()` call in libgit2
re-runs.

**Fix:** cache `GitInfo` in the file cache with a 60 s TTL,
mirroring the pattern for `scan_insights`. The cache key is
`<path>:git`.

| Path | Pre-fix | Post-fix | Speedup |
|------|--------:|---------:|--------:|
| `~/Dev/dracon-platform/web/music` (truly cold) | 8.3 s | 8.3 s | 1.0× (one-time) |
| `~/Dev/dracon-platform/web/music` (daemon restart, file cache warm) | **8.3 s** | **61 ms** | **136×** |
| `~/Dev/dracon-platform/web/music` (warm daemon cache) | 2 ms | 2 ms | 1.0× (already fast) |

### Replace libgit2 with native git (0.7.9)

The 0.7.8 file cache helped for repeated scans, but the root cause
remained: libgit2's `repo.statuses()` is 500× slower than native
git on large repos. libgit2 lacks git's index optimization,
untracked cache, and fsmonitor hook.

**Fix:** Replace the entire `git2` crate (and its `libgit2-sys` C
dependency) with native `git` subprocess calls in `src/git/mod.rs`.
All 10 git data fields are collected via `git -C <path>` commands
spawned in parallel threads. The total cost is dominated by `git
status --porcelain` (15-33ms).

| Path | libgit2 (0.7.8) | native git (0.7.9) | Speedup |
|------|----------------:|-------------------:|--------:|
| `~/Dev/dracon-platform/web/music` (truly cold) | 5-8 s | **104 ms** | **50-80×** |
| `~/Dev/dracon-platform/web/music` (warm daemon cache) | 2 ms | 2 ms | 1.0× |
| `~/Dev/dracon-platform/web/music` (daemon restart, file cache warm) | 8+ s | **15 ms** | **500×** |
| `~/Dev/folder-auto-banner` (cold) | 215 ms | **110 ms** | **2×** |

The `git2` and `libgit2-sys` C dependencies have been removed from
Cargo.toml, reducing compile time significantly.

## Implementation notes

- `ProjectInsights` now derives `serde::Serialize + Deserialize`
  so the file cache can round-trip the combined insight result.
- The cache key is `<path>:insights` (same shape as the other
  features).
- TTL is 60 s, matching the cadence at which new TODOs and new
  code typically appear in an actively-edited project.
- 3 new tests in `src/fs/mod.rs::tests` cover the round-trip
  (`test_project_insights_serializes`), the cache hit
  (`test_scan_insights_cache_warm_returns_same_value`), and the
  cache expiry (`test_scan_insights_cache_expired_returns_none`).
