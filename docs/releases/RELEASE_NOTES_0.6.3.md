# Release notes — folder-auto-banner 0.6.3

Date: 2026-06-09

## Headline

Fresh folder information without the cache-hit latency introduced by the 0.6.2 shallow-validation approach.

## What's in this release

### Snappy active-folder freshness

Version 0.6.2 fixed stale folder information by validating every daemon cache hit against a fresh shallow scan. That preserved freshness, but it reintroduced latency on hot paths because every `f` invocation could pay for another directory walk.

In 0.6.3 the daemon keeps the freshness guarantees while avoiding that per-request scan:

- Requested folders become active and are watched with inotify.
- Watch coverage includes a bounded set of descendant files and directories, so nested creates, deletes, modifications, and moves can invalidate the cached banner.
- A cheap root-mtime check remains as a lightweight fallback for changes that do not produce an actionable inotify event.
- Newly active folders are prioritized so hot folders get watched promptly even when many folders are cached.
- Persisted caches from a previous daemon run are treated as expired, so the first request after restart recomputes the banner and refreshes directory size data.
- When an active-folder event invalidates a banner cache entry, related directory-size cache entries are pruned so displayed directory sizes recompute instead of being shadowed by stale cached sizes.

### Preserved behavior

- Existing banner output shape and CLI behavior are unchanged.
- `f daemon clear-cache` behavior from 0.6.2 remains unchanged.
- No new dependencies were added.

## What end users should expect

- `f` stays snappy for folders that have already been computed by the daemon.
- Nested folder changes still refresh on the next `f` invocation without waiting for the 5-minute TTL.
- Displayed directory sizes still reflect current nested content after file changes.
- Daemon restarts may recompute the first request for a folder, then cache normally.

## Validation evidence (all green on 0.6.3)

- `cargo fmt --all` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — no issues
- `cargo test --all-features -- --nocapture --test-threads=1` — 104 passed (5 suites)
- `cargo doc --no-deps` — passed
- `cargo build --release --locked` — passed
- `cargo publish --dry-run` — packaged successfully before upload
- `cargo publish` — uploaded `folder-auto-banner 0.6.3` to crates.io and verified via crates.io API

### Manual smoke (this repo, `target/release/f`)

- Warm cache hits for the current repo returned in the single-digit millisecond range after the first request.
- New top-level file creation was detected while the daemon was warm.
- Top-level file deletion was detected while the daemon was warm.
- Nested file creation was detected while the daemon was warm.
- Nested file modification invalidated the cache and refreshed the displayed directory size.
- `f daemon restart` / `f daemon stop` completed without spurious shutdown warnings.

## Notes for maintainers

This is the real 0.6.3 release. It supersedes the 0.6.2 freshness approach by replacing per-cache-hit shallow validation with active-folder watching plus a cheap root-mtime fallback.

The `release.yml` workflow has previously been canceled by GitHub-hosted runners during the `Build release` step. If that happens again for 0.6.3, the release should still be created manually with `gh release create v0.6.3 --notes-file RELEASE_NOTES_0.6.3.md`, and Linux x86_64 binaries can be built locally and attached with `gh release upload`.

## Assets in this release

- `f`, `fabd` — Linux x86_64 (built locally and attached manually if the `release.yml` workflow is canceled again).

aarch64-linux and macOS binaries remain optional follow-up assets; they can be attached after a successful `release.yml` run with `gh release upload v0.6.3 ...`.
