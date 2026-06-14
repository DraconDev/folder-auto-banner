# folder-auto-banner 0.6.27

## Summary
Per-path on-disk response cache plus a fix for a pre-existing shadowing
bug in the daemon's cache-invalidation path. Eliminates the IPC round-trip
for warm cache hits, dropping median per-call latency from ~10 ms to
~1.5–2.2 ms across all tested paths.

## What changed

- **New `src/cmd/banner_data_cache.rs`** — per-path on-disk cache.
  After every successful banner compute, the daemon writes the
  serialized `BannerData` to
  `~/.local/share/fab/banner_data/<hash>.json` where `<hash>` is a
  stable FNV-1a 64-bit digest of the canonicalized path. The client,
  before opening a Unix-socket connection, checks whether the file
  exists, is younger than `CACHE_TTL` (5 min), and is not older than
  the directory's mtime. If all three checks pass, the client reads
  the file directly and skips the IPC.

- **Fixed pre-existing shadowing bug in
  `Request::Banner` cache-invalidation** (in `src/daemon.rs`).
  The inner `let data = compute_banner_data(...)` was shadowing
  the outer `data`, so the response (and the new disk-cache write)
  used the OLD cached entry instead of the freshly-computed data.
  The cache itself was updated correctly, but the response was one
  compute behind in the cache-invalidation case. The shadowing is
  now removed.

## Why the disk cache is a big win

A 4-byte read of a Unix-socket response is dominated by kernel
scheduling (1–10 ms on Linux). A stat + read of a 70 KB file is
dominated by page-cache hits (<0.1 ms). The disk path is therefore
typically 5–50× faster than the IPC path for a warm cache hit.

## Validation

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (121 tests, up from 117)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`

## Measured impact (vs 0.6.26, daemon settled)

- `f /home/dracon/Downloads` (221 items):
  - median: 10.17 ms → 2.13 ms (**4.8× faster**)
  - p99: 15.62 ms → 2.76 ms (**5.7× faster**)
- `f /home/dracon/Dev` (17 items):
  - median: 10.80 ms → 1.67 ms (**6.5× faster**)
- `f /home/dracon/Dev/folder-auto-banner` (53 items):
  - median: 10.31 ms → 1.50 ms (**6.9× faster**)
- `f /home/dracon/Dev/dracon-code` (44 items):
  - median: 10.68 ms → 1.61 ms (**6.6× faster**)
- `f /home/dracon/Dev/dracon-platform` (26 items):
  - median: 10.61 ms → 1.50 ms (**7.1× faster**)
- All paths now consistently under 2.2 ms median with p99 ≤ 2.8 ms
  after the daemon has settled.

## Preserved behavior

- Image resolution extraction still works (PNG `WxH`, JPG `WxH`).
- ZIP entry counts still work (e.g. `19`).
- Text file line counts still work.
- Per-extension sort, type sort, and group_dirs still work.
- Icons, exact-name matches, and lowercase ordering all unchanged.
- MP4/MKV duration extraction unchanged.
- Cache freshness: the file is invalidated when the directory's
  mtime advances (e.g., a new file is created in the directory),
  so stale data is never shown to the user.
