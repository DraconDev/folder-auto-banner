# Release Notes — f 0.7.7

**Date**: 2026-06-18

## Cold-path: cache the combined TODO + code-metrics scan

The first scan of a folder was noticeably slow because
`scan_insights` (TODO counts + code metrics in one bounded tree
walk) was the dominant cost on the cold path — 60–65% of
`compute_banner_data` time on `~/Dev` (127 ms of 198 ms). It
ran on every cold scan because it was the only expensive phase
in `DirSummary::scan_with_options` not covered by the existing
`cached_check!` macro.

## Changes

### Fixed

- `src/fs/mod.rs` — wrapped `scan_insights` in the same file
  cache used for `build_status`, `port_info`, and `docker_info`,
  with a 60 s TTL. The cache key is `<path>:insights`.
- `src/project_insights.rs` — `ProjectInsights` now derives
  `Serialize + Deserialize` so the combined result round-trips
  through the cache.
- `src/fs/mod.rs::tests` — 3 new tests:
  - `test_project_insights_serializes` (round-trip)
  - `test_scan_insights_cache_warm_returns_same_value` (hit)
  - `test_scan_insights_cache_expired_returns_none` (TTL)

### Added

- `PROFILE_COLD_PATH.md` — the cold-path profile harness, the
  per-phase breakdown, and the methodology.

## Measured impact (`~/Dev`, OS file cache warm)

| Path | Pre-fix | Post-fix | Speedup |
|------|--------:|---------:|--------:|
| First-ever scan of a folder | 198 ms | 204 ms | 1.0× (one-time) |
| Daemon restart, file cache populated | **198 ms** | **4 ms** | **50×** |
| Warm daemon cache (5 min TTL) | 3 ms | 3 ms | 1.0× (already fast) |

The fix targets the case the user actually feels: when they
type `f <folder>` after the daemon has been idle long enough
for its 5 min in-memory cache to expire, or after a daemon
restart. The file cache (`/tmp/f-cache/`) survives both, so
the second-and-onwards cold scan is now 4 ms instead of 198 ms.

## Files changed

- `src/fs/mod.rs` — insights caching, 3 new tests
- `src/project_insights.rs` — Serialize + Deserialize derives
- `PROFILE_COLD_PATH.md` — design doc
- `CHANGELOG.md` — entry for 0.7.7
- `Cargo.toml` — version bump 0.7.6 → 0.7.7
- `f.1` — version bump 0.7.6 → 0.7.7

## Test metrics

- 238 tests pass (up from 235 in 0.7.6)

## Installation

```bash
cargo install folder-auto-banner --version 0.7.7 --locked --force
```

Or update an existing install:

```bash
cargo install folder-auto-banner --force
```
