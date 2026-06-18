# Release Notes — f 0.7.9

**Date**: 2026-06-18

## Replace libgit2 with native git (50-80× faster cold scan)

The `git2` crate (libgit2) has been replaced with native `git`
subprocess calls. On `~/Dev/dracon-platform/web/music` (15K
commits, 5.8 GB `.git`):

| Path | libgit2 (0.7.8) | native git (0.7.9) | Speedup |
|------|----------------:|-------------------:|--------:|
| Cold scan (no cache) | 5-8 s | **104 ms** | **50-80×** |
| Warm daemon cache | 2 ms | 2 ms | 1.0× |
| Daemon restart, file cache warm | 8+ s | **15 ms** | **500×** |

## Changes

### Fixed

- `src/git/mod.rs` — Replaced all libgit2 calls with native `git`
  subprocess commands. All 10 git data fields (branch, staged,
  modified, untracked, ahead/behind, last commit, commits today,
  branch count, stash count, merge state, tag, diff stats) are
  collected via `git -C <path>` commands spawned in parallel
  threads. Total cost dominated by `git status --porcelain`
  (15-33ms).

### Removed

- `git2` and `libgit2-sys` C dependencies removed from Cargo.toml.
  Compile time reduced significantly.

## Why libgit2 was slow

libgit2's `repo.statuses()` walks the entire working tree to compute
status, while native `git status` uses:

1. **Index optimization** — pre-computed file list
2. **Untracked cache** — cached untracked files
3. **Fsmonitor hook** — filesystem watcher integration (git 2.37+)
4. **Diff cache** — `ce_uptodate` flag
5. **Racy-git optimization** — skip files modified within the same
   second as the index

libgit2 doesn't have these optimizations. On a 15K-commit repo
with 5.8 GB `.git`, the difference is 500× (7s vs 15ms).

## Files changed

- `src/git/mod.rs` — Complete rewrite: libgit2 → native git
- `src/daemon.rs` — Updated comment about pathspec behavior
- `Cargo.toml` — Removed `git2` dependency, version bump
- `Cargo.lock` — Updated
- `f.1` — Version bump
- `CHANGELOG.md` — Entry for 0.7.9
- `PROFILE_COLD_PATH.md` — Updated with new benchmark data

## Test metrics

- 238 tests pass (no change)

## Installation

```bash
cargo install folder-auto-banner --version 0.7.9 --locked --force
```

Or update an existing install:

```bash
cargo install folder-auto-banner --force
```
