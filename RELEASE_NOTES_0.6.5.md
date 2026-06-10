# Release notes — folder-auto-banner 0.6.5

Date: 2026-06-10

## Headline

The active-folder watcher no longer chases VCS and build internals, and descendant file churn no longer invalidates the parent banner.

## What's in this release

### VCS and build internals are excluded from the watcher

In 0.6.4 the inotify watcher would observe every file under an active folder, including files inside `.git/`, `target/`, `node_modules/`, and other internal directories. The daemon's own git operations (creating `.git/index.lock` and `.git/objects/tmp_object_*` files) would then trigger the watcher and invalidate the cached banner within seconds of a request, which prevented the size cache from ever persisting to disk.

In 0.6.5 the watcher now skips these directories entirely:

- `.git`, `.hg`, `.svn`
- `target`, `node_modules`, `.next`, `dist`, `build`
- `.cache`, `.parcel-cache`, `.turbo`

### Descendant changes only prune the size cache

Even with VCS/build directories excluded, the watcher was still invalidating the **entire parent banner** when any descendant file or directory changed. The banner's item listing only changes when a direct child of the root is created or removed, so 0.6.5 now:

- Invalidates the banner cache only for events on the root directory itself.
- Prunes the size cache for the affected root on descendant events, so the next request recomputes the affected child size without recomputing the full listing.

## What end users should expect

- The cached banner and size data survive across requests and daemon restarts without being wiped by background VCS or build activity.
- Warm requests for large folders such as `~/Dev` stay in the single-digit millisecond range.
- Freshness behavior from 0.6.3/0.6.4 remains unchanged.

## Validation evidence (all green on 0.6.5)

- `cargo fmt --all` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — no issues
- `cargo test --all-features -- --nocapture --test-threads=1` — 104 passed (5 suites)
- `cargo build --release --locked` — passed
- `cargo publish --dry-run` — packaged successfully before upload
- `cargo publish` — uploaded `folder-auto-banner 0.6.5` to crates.io and verified via crates.io API

### Manual smoke (live local install)

- `~/.local/bin/f --version` reports `f 0.6.5`.
- `~/.local/bin/fabd` is the installed 0.6.5 daemon.
- 12 manual test scenarios all pass:
  1. Cold and warm `~/Dev` switching: cold ~1.3s, warm 7–10ms.
  2. Nested file modification detected (size cache pruned for affected root).
  3. Top-level create/delete detected within 400ms.
  4. `--max` and `--filter` affect human display only; `--json` is unfiltered.
  5. Missing and unreadable paths return empty banner with exit code 0.
  6. `f config` opens the editor cleanly.
  7. `f daemon status`, `stop`, `restart`, `clear-cache` all work without misleading shutdown warnings.
  8. 10 consecutive warm runs all under 10ms.
  9. Daemon survives SIGTERM and SIGKILL; restart reloads persisted caches and serves warm requests in 15–30ms.
  10. 5 parallel clients all get consistent results.
  11. Git repo shows `is_repo: true` with branch info; non-git folders show `is_repo: false` without panic.
  12. Rapid switching between 5 folders completes in 1–2s for cold and under 200ms for warm.

## Notes for maintainers

This is the real 0.6.5 release. It fixes the remaining cache-invalidation regression from 0.6.4 where the inotify watcher would observe VCS and build internals and invalidate the banner cache within seconds of a request.

If the GitHub Actions `release.yml` workflow is canceled again during `Build release`, create the GitHub release manually with `gh release create v0.6.5 --notes-file RELEASE_NOTES_0.6.5.md` and attach Linux x86_64 binaries built locally.

## Assets in this release

- `f`, `fabd` — Linux x86_64 (built locally and attached manually if the `release.yml` workflow is canceled again).

aarch64-linux and macOS binaries remain optional follow-up assets.
