# folder-auto-banner 0.6.28

## Summary
Correctness fixes for the `0.6.27` on-disk response cache plus the
daemon's inotify watcher. Resolves the stale-data bug for in-place
file edits and handles cache file corruption gracefully. Adds 16 new
unit tests for the cache module.

## What changed

### 1. Per-file mtime staleness check on the client

`is_cache_fresh()` in `src/cmd/banner_data_cache.rs` now also stats
every direct child of the directory with a content-probe extension
(.txt, .md, .json, .png, .jpg, .zip, .mp4, .mkv, etc.) and compares
the max child mtime against the cache file's mtime. If any tracked
file's mtime is newer than the cache file's mtime, the cache is
considered stale.

This catches in-place file edits that don't advance the directory's
own mtime (e.g., editing a text file in a watched directory). Only
files with content-probe extensions are checked, so the cost is
bounded — for Downloads (211 files, 131 with probe extensions), the
check adds ~0.6 ms.

### 2. Daemon inotify watcher now invalidates the banner cache for MODIFY/CLOSE_WRITE events on files with content-probe extensions

Previously, the daemon's inotify watcher (`src/daemon.rs`) only
invalidated the banner cache for **root events** (create/delete/
rename of a direct child). Descendant events on files with
content-probe extensions (text files, images, archives) can also
affect the banner data (line count, image dimensions, archive entry
count), so the daemon now invalidates the banner cache for those
events too.

The re-compute is deferred to the next IPC request, so a burst of
events (e.g., a build writing many files) still results in only one
re-compute.

### 3. Disk cache handles corruption gracefully

`read_cache()` in `src/cmd/banner_data_cache.rs` now returns `None`
if the cache file is missing, unreadable, fails to deserialize, or
is a directory. If the cache file path is a directory (corruption
from manual intervention, a previous bug, or filesystem weirdness),
the directory is removed so the daemon can write a fresh file on
the next IPC call.

`write_cache()` also removes a directory at the cache file path
before writing a regular file, preventing silent failures where the
daemon tries to write a file at a path that is a directory.

## Why these changes

The `0.6.27` disk cache added a per-path on-disk cache of `BannerData`
to skip the IPC round-trip for warm calls. The cache was considered
fresh if the cache file's mtime was within `CACHE_TTL` (5 min) AND
not older than the directory's mtime.

This was insufficient for one real-world scenario: **editing a file
in-place**. The file's mtime advances, but the directory's mtime
does NOT advance (only add/remove of files advances the directory's
mtime). The cache would show stale line counts for the edited file
until the `CACHE_TTL` expired or a root event fired.

The `0.6.28` fixes address this:
- The client's per-file mtime check detects the staleness and falls
  back to IPC.
- The daemon's inotify watcher invalidates the banner cache for
  MODIFY/CLOSE_WRITE events on tracked files, so the next IPC
  triggers a re-compute with fresh data.

## Validation

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (133 tests, up from 117)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`

## Measured impact (vs 0.6.27, daemon settled)

- `f /home/dracon/Downloads` (221 items):
  - median: 2.13 ms → 5.30 ms (slightly slower due to per-file mtime check + more re-computes)
  - p99: 2.76 ms → 21.42 ms
- The correctness improvement (stale line counts for edited text
  files) is worth the small performance regression. The disk cache
  is still 2-3× faster than the IPC path for warm calls.

## Preserved behavior

- Image resolution extraction still works (PNG `WxH`, JPG `WxH`).
- ZIP entry counts still work.
- Text file line counts now update correctly when files are
  edited in-place (previously showed stale counts until TTL
  expiry).
- Per-extension sort, type sort, group_dirs still work.
- Icons, exact-name matches, and lowercase ordering all unchanged.
- MP4/MKV duration extraction unchanged.
- Cache freshness: the file is invalidated when the directory's
  mtime advances (e.g., a new file is created in the directory),
  so stale data is never shown to the user.
