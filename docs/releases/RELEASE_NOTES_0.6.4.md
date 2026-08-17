# Release notes — folder-auto-banner 0.6.4

Date: 2026-06-10

## Headline

Large folder switches stay snappy after daemon restarts by persisting and reusing directory size data.

## What's in this release

### Persisted directory sizes

The 0.6.3 release made cache hits fast and bounded the first cold request for large folders, but the daemon did not persist the directory size cache. That meant a daemon restart could force large folders such as `~/Dev` to recompute child directory sizes again.

In 0.6.4 the daemon now:

- Persists directory sizes together with their mtimes in `dir_sizes.json`.
- Reloads those sizes and mtimes on daemon restart.
- Reuses cached sizes when the directory mtime still matches, so large parent folders avoid repeated `du` scans after restart.
- Falls back to fresh size computation when a directory mtime changed.

### Bounded large-folder size computation

The first cold request for a large folder with many children can still need fresh size data, but 0.6.4 computes those sizes with a bounded worker pool instead of waiting serially for each child directory. This keeps the first cold request bounded to low seconds rather than many seconds.

## What end users should expect

- Switching into large folders such as `~/Dev` is fast after the size cache is warm.
- Restarting the daemon no longer throws away all directory size data.
- The first cold request for a large folder may still take around a second, but warm requests should be back in the single-digit millisecond range.
- Freshness behavior from 0.6.3 remains unchanged.

## Validation evidence (all green on 0.6.4)

- `cargo fmt --all` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — no issues
- `cargo test --all-features -- --nocapture --test-threads=1` — 104 passed (5 suites)
- `cargo build --release --locked` — passed
- `cargo publish --dry-run` — packaged successfully before upload
- `cargo publish` — uploaded `folder-auto-banner 0.6.4` to crates.io and verified via crates.io API

### Manual smoke (live local install)

- `~/.local/bin/f --version` reports `f 0.6.4`.
- `~/.local/bin/fabd` process is `/home/dracon/.local/bin/fabd`.
- Cleared cache, started daemon, requested `/home/dracon/Dev`: cold request completed in ~1s.
- Stopped daemon gracefully: `dir_sizes.json` was saved with 20 entries and 20 mtimes.
- Restarted daemon: it loaded 20 cached directory sizes from disk.
- Re-requested `/home/dracon/Dev` after restart: ~15ms, not ~10s.
- Warm requests stayed in the single-digit millisecond range.
- Freshness smoke still passed: top-level create/delete and nested file modification are reflected without waiting for TTL.

## Notes for maintainers

This is the real 0.6.4 release. It fixes the remaining large-folder latency issue from 0.6.3 by persisting and reusing directory size data across daemon restarts.

If the GitHub Actions `release.yml` workflow is canceled again during `Build release`, create the GitHub release manually with `gh release create v0.6.4 --notes-file RELEASE_NOTES_0.6.4.md` and attach Linux x86_64 binaries built locally.

## Assets in this release

- `f`, `fabd` — Linux x86_64 (built locally and attached manually if the `release.yml` workflow is canceled again).

aarch64-linux and macOS binaries remain optional follow-up assets.
