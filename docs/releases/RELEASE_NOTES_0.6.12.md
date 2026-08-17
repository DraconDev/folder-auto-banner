# Release notes — folder-auto-banner 0.6.12

Date: 2026-06-10

## Headline

Fixes the daemon IPC failure on bad paths and tightens cold-size refresh latency for large trees.

## What's in this release

### Fixes and performance

- **No more half-closed daemon stream on compute errors**: daemon-side compute failures now return a structured IPC error response instead of closing without a response.
- **Clearer missing-path errors**: a relative path that does not exist, such as `f Dev` from inside `folder-auto-banner`, now fails directly with `No such file or directory: Dev` instead of producing a confusing `send_and_recv` error.
- **Tighter directory-size refresh timeout**: cold views of large trees now bound the expensive `du` size refresh more tightly, reducing first-hit latency while preserving cached sizes on later visits.

### Behavior preserved

- Warm cache hits remain single-digit millisecond responses.
- Rich banner output, JSON output, and numeric navigation behavior remain unchanged.
- The active local installs are updated to `0.6.12`.

## Validation evidence

- `cargo fmt --all` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — no issues
- `cargo test --all-features` — 108 passed (5 suites)
- `cargo doc --no-deps` — passed
- `cargo build --release --locked` — passed
- `cargo publish --dry-run` — packaged successfully before upload
- `cargo publish` — uploaded `folder-auto-banner 0.6.12` to crates.io

### Timing evidence

- Cold first view of `/home/dracon/Dev` improved from about `1.2–1.4 s` to about `0.7–0.8 s` in live tests.
- Warm cached view of `/home/dracon/Dev` stayed around `4–9 ms`.
- `f Dev` from inside `folder-auto-banner` now reports a direct missing-path error and no longer emits the daemon `failed to fill whole buffer` IPC error.

## Notes for maintainers

This release keeps the latest local binary active and makes large-tree first hits less painful without changing warm-cache behavior.
