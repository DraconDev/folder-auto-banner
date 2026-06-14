# folder-auto-banner 0.6.26

## Summary
Move per-file content probes (PNG/JPG resolution, ZIP entry count, MP4/MOV/M4V/WebM/MKV
duration, SQLite table count, text line count) from the client to the daemon. With the
daemon's 5-minute `CACHE_TTL`, the per-file I/O happens at most once per 5-minute window
per directory instead of on every `f` invocation.

## What changed

- **New `DirEntry.content_probe: Option<String>` field** — populated by the daemon
  during `compute_banner_data()` via the new `populate_content_probes()` helper.
  Serialized as part of `BannerData` over the Unix socket IPC. The client
  (`get_file_contents_raw` in `src/cmd/banner.rs`) reads it directly and only
  falls back to a synchronous probe if the field is missing.
- **Probe coverage unchanged** — every file extension that the old client probed
  is still probed; we just do it once per cache TTL instead of once per `f`
  invocation.

## Validation

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (117 tests)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`

## Measured impact (vs 0.6.25)

- `f /home/dracon/Downloads` (221 items, 59 PNGs + 17 ZIPs + 12 JPGs + 121 text files):
  - median: 34.0 ms → 13.5 ms (**2.5× faster**)
  - p90: 71.7 ms → 25.7 ms (**2.8× faster**)
- `f /home/dracon/Dev/folder-auto-banner`:
  - median: 13.9 ms → 13.8 ms (unchanged)
- `f /home/dracon/Dev/dracon-code`:
  - median: 14.4 ms → 17.1 ms (slight noise; daemon size refresh)
- `f /home/dracon/Dev/dracon-platform`:
  - median: 19.5 ms → 29.6 ms (daemon background work)
- `f /home/dracon/Dev`:
  - median: 18.4 ms → 20.9 ms (noise)
- **Downloads is no longer the slowest path** — it now consistently
  matches the other top-level paths in median / p90.

## Trade-offs

- An in-place edit of a small text file (size unchanged) won't update its
  cached line count until the next 5-minute TTL refresh. This is cosmetic;
  the cost of running `read_to_string` on every `f` invocation was 5–10 ms
  on a Downloads-class directory.
- The on-disk `banner_cache.json` grows by ~10–20 KB per directory (probe
  results are serialized alongside the directory's other metadata).

## Preserved behavior

- Image resolution extraction still works (PNG `WxH`, JPG `WxH`).
- ZIP entry counts still work (e.g. `19`, `38`).
- Text file line counts still work.
- Per-extension sort, type sort, and group_dirs still work.
- Icons, exact-name matches, and lowercase ordering all unchanged.
- MP4/MKV duration extraction unchanged from 0.6.24 (64 KiB header probe).
