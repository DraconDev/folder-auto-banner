# folder-auto-banner 0.6.25

## Summary
Further extension-handling and sort performance improvements on top of 0.6.24.

## What changed

- **Pre-computed sort keys** — the per-comparison sort callback no longer
  allocates a fresh lowercase copy of every entry name on each comparison.
  We build a parallel `Vec<SortKeys>` once (lowercase name, lowercase
  extension, date, git status) and sort the indices into it. This makes
  the sort `O(N)` allocations instead of `O(N log N)`, and avoids a hot
  per-row `to_lowercase` String allocation for every comparison.
- **`sort_by_cached_key` for grouped dirs/symlinks** — the
  `group_dirs=first/last` pre-sort that splits display_items into
  `dirs / files / symlinks` now caches the lowercase key once per entry.
- **`Path::extension()` instead of `name.to_lowercase().ends_with(...)`**
  in `get_file_contents` — `std::path::Path::extension()` returns a
  borrowed `&str` with no allocation, and the per-extension dispatch is
  now a `match` arm instead of a chain of `if` blocks.
- **Dropped redundant `metadata().len()` in `read_file_header`** — the
  caller has already populated `entry.size` for the size column, so we
  skip the per-file `stat()` syscall and just `take(64 KiB).read_to_end()`.
- **Skipped `stream.shutdown(Shutdown::Write)` on the daemon IPC client** —
  the daemon's length-prefixed protocol already knows when the request
  ends (it reads exactly `req_len` bytes), so the shutdown is unnecessary.
  Removing it shaves a few hundred microseconds off the per-call latency.

## Validation

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (112 tests)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`

## Measured impact (vs 0.6.24)

- `f /home/dracon/Downloads` (221 items, 59 PNGs + 17 ZIPs + 12 JPGs):
  - median: ~30 ms → ~10 ms (**3× faster**)
  - p99: ~150 ms → ~20 ms (**7× faster**)
- `f /home/dracon/Dev/folder-auto-banner`:
  - median: 11 ms → 10 ms
- `f /home/dracon/Dev/dracon-code`:
  - median: 21 ms → 10 ms (**2× faster**)
- All paths now consistently under 11 ms median with p99 ≤ 31 ms after
  warm cache.

## Preserved behavior

- Image resolution extraction still works (PNG `WxH`, JPG `WxH`).
- ZIP entry count still works (e.g. `19`, `38`).
- Text file line counts still work.
- SQLite table counts still work.
- Extension sort, type sort, and group_dirs still work.
- Icons, exact-name matches, and lowercase ordering all unchanged.
- MP4/MKV duration extraction unchanged from 0.6.24 (64 KiB header probe).
