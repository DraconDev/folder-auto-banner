# folder-auto-banner 0.6.24

## Summary
Fixes slow path switching in directories with many images, archives, videos, and text files.

## What changed
- The `contents` column's per-file content probe (PNG/JPG resolution, ZIP entry count, MP4/MOV/MKV/WebM duration) now reads at most 64 KiB of each file instead of the entire file. The metadata these probes return always lives near the start of the file, so the full read was wasteful.
- The per-file content probe is skipped entirely when the user has hidden the `contents` column.
- The smart-truncation hidden counter is now O(N) total instead of O(N×M) per category.

## Validation
- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`

## Measured impact (vs 0.6.23)
- `f /home/dracon/Downloads` (221 items, 59 PNGs + 17 ZIPs + 12 JPGs + 149 text files):
  - median: 262ms → 41ms (**6.4× faster**)
  - p90: ~700ms → 58ms (**12× faster**)
- `f /home/dracon/Dev/dracon-code`:
  - median: 52ms → 32ms
- Numeric navigation `f N` after `cd`:
  - 6–17 ms (was already fast)
