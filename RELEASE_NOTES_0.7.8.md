# Release Notes — f 0.7.8

**Date**: 2026-06-18

## Cold-path: cache git status for large repos (120× speedup)

After the scan_insights fix in 0.7.7, the remaining cold-path
bottleneck on large repos was git status. On `~/Dev/dracon-platform`
(15K commits, 5.8 GB `.git`), `get_git_info_filtered` takes
8–11 seconds per cold scan because `libgit2::repo.statuses()` walks
the entire git tree.

The daemon's in-memory BannerCache (5 min TTL) masks this in the
common case, but after it expires or on daemon restart the full
cost is re-paid.

## Changes

### Fixed

- `src/daemon.rs` — `compute_banner_data` now caches `GitInfo` in
  the existing file cache with a 60 s TTL. The cache key is
  `<path>:git`. The cache hit path skips `get_git_info_filtered`
  entirely, collapsing an 8+ second libgit2 tree walk to < 1 ms.

## Measured impact (`~/Dev/dracon-platform/web/music`, 15K commits, 5.8 GB `.git`)

| Path | Pre-fix | Post-fix | Speedup |
|------|--------:|---------:|--------:|
| First-ever scan of the folder | 8.3 s | 8.3 s | 1.0× (one-time) |
| Daemon restart, file cache populated | **8.3 s** | **96 ms** | **120×** |
| Warm daemon cache (5 min TTL) | 2 ms | 2 ms | 1.0× (already fast) |

## Files changed

- `src/daemon.rs` — git status caching in `compute_banner_data`
- `PROFILE_COLD_PATH.md` — updated with large-repo measurements
- `CHANGELOG.md` — entry for 0.7.8
- `Cargo.toml` — version bump 0.7.7 → 0.7.8
- `f.1` — version bump 0.7.7 → 0.7.8

## Test metrics

- 238 tests pass (no change from 0.7.7)

## Installation

```bash
cargo install folder-auto-banner --version 0.7.8 --locked --force
```

Or update an existing install:

```bash
cargo install folder-auto-banner --force
```
