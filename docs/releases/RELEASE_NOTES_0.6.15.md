# Release notes — folder-auto-banner 0.6.15

Date: 2026-06-10

## Headline

Fixes the warm-cache path so large trees like `~/Dev` pre-warm the directories you actually jump into next.

## What's in this release

### Fixes and performance

- **Reliable child pre-warming**: warm requests now use one short-lived daemon connection per path and are sent before the CLI exits, so the pre-warmed child directories actually get cached.
- **Wider pre-warm coverage**: the client now warms the parent, grandparent, and up to 30 immediate children of the current directory, which covers large `~/Dev` trees much better than the previous 5-child limit.
- **Bounded cold-size refresh**: directory size refresh uses a bounded `du` timeout to reduce first-hit latency on very large trees while keeping normal directory sizes accurate.
- **Cleaner daemon IPC failures**: daemon-side compute errors now return structured IPC errors instead of leaving the client to read a half-closed stream.
- **Clearer missing-path errors**: relative paths that do not exist now fail directly with `No such file or directory: Dev` instead of producing a confusing `send_and_recv` error.

### Behavior preserved

- Warm cache hits remain single-digit millisecond responses.
- Rich banner output, JSON output, and numeric navigation behavior remain unchanged.
- The active local installs are updated to `0.6.15`.

## Validation evidence

- `cargo fmt --all` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — no issues
- `cargo test --all-features` — 108 passed (5 suites)
- `cargo doc --no-deps` — passed
- `cargo build --release --locked` — passed
- `cargo publish --dry-run` — packaged successfully before upload
- `cargo publish` — uploaded `folder-auto-banner 0.6.15` to crates.io

### Timing evidence

- Cold first view of `/home/dracon/Dev` stayed around `0.7–0.8 s` after the bounded size refresh.
- Warm cached view of `/home/dracon/Dev` stayed around `4–9 ms`.
- After pre-warming `~/Dev`, a later jump into `~/Dev/dracon-platform` dropped from a cold `~1.2 s` to about `0.4 s`, and a second hit was `~7–9 ms`.
- `f Dev` from inside `folder-auto-banner` now reports a direct missing-path error and no longer emits the daemon `failed to fill whole buffer` IPC error.

## Notes for maintainers

This release keeps the latest local binary active and makes large-tree navigation much more cache-friendly.
