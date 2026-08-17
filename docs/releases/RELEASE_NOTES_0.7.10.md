# Release Notes — f 0.7.10

**Date**: 2026-06-18

## Fix cold-path slowdown on non-project directories

Large directories like `/tmp` (136K entries) and `~/Downloads` were
slow on first scan because the daemon walked every entry and ran
insights scanning (TODOs, code metrics) even for non-code dirs.

| Directory | Before | After | Speedup |
|-----------|-------:|------:|--------:|
| `/tmp` (136K entries) | 82 s | **803 ms** | **100×** |
| `~/Downloads` (222 entries) | 2.0 s | **282 ms** | **7×** |
| `~/Dev/dracon-platform/web/music` | 209 ms | **266 ms** | same |

## Changes

### Fixed

- `src/fs/mod.rs` — 500-item directory walk cap: stop collecting
  metadata after 500 entries. The banner only displays a limited
  number of items, so walking 100K+ entries is pure waste.
- `src/fs/mod.rs` — Skip insights for Generic project types: no
  TODO/code metric scanning for non-code directories.
- `src/fs/mod.rs` — Skip insights for large directories: even if
  detected as a project type, skip insights when >500 entries.
- `src/daemon.rs` — 10-second minimum invalidation age: prevents
  rapid cache invalidation in active directories like /tmp.

### Technical details

- `ProjectType` now derives `PartialEq` for comparison
- `scan_with_options` uses a `hit_cap` flag to track when the
  500-item limit is reached
- `should_skip_dir` skips `.git`, `target`, `node_modules`, etc.
  for inotify watching

## Files changed

- `src/fs/mod.rs` — Directory walk cap, insights skip logic
- `src/daemon.rs` — Minimum invalidation age, removed profiling
- `Cargo.toml` — Version bump
- `f.1` — Version bump
- `CHANGELOG.md` — Entry for 0.7.10

## Test metrics

- 238 tests pass (no change)

## Installation

```bash
cargo install folder-auto-banner --version 0.7.10 --locked --force
```
