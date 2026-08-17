# Release notes — folder-auto-banner 0.6.2

Date: 2026-06-09

## Headline

Folder information shown by `f` is now reliably fresh.

## What's in this release

### Freshness fix

The daemon's cached banner data could remain stale between filesystem changes because the
cache was trusted until the 5-minute TTL (or until an inotify event invalidated it). In
practice this meant edits inside nested folders — including new or modified files — could
not show up immediately, and a directory's size could lag behind the real on-disk total.

In 0.6.2 the daemon now:

- Validates the cached folder snapshot against a fresh shallow scan on every cache hit, so
  out-of-band edits are surfaced on the next `f` invocation without waiting for TTL.
- Refreshes displayed directory sizes whenever the directory's mtime changes, so nested
  content edits immediately update parent folder size information.
- Tracks directory mtimes alongside cached sizes, so a persisted size cache from a previous
  daemon run can no longer shadow fresh data.

### Smaller fixes shipped along the way

- `f daemon clear-cache` now also clears the daemon's `banner_cache.json` and
  `dir_sizes.json` (it previously only cleared the cache directory), and does so without
  emitting the misleading `Failed to send shutdown request` warning.
- The benchmark harness references the real crate name (`folder_auto_banner`) instead of
  the historical `fab_lib` placeholder, so `cargo bench` is consistent with the rest of
  the workspace.
- A rustdoc HTML warning in `port_usage` for the `<pid>` token was fixed.

## What end users should expect

- The banner shown by `f` (and the auto-banner hook on `cd`) reflects the current state of
  the directory on every invocation.
- For directories with a running daemon, the daemon continues to cache the *expensive*
  parts (git status, port/docker detection) but always re-validates the *immediate* folder
  snapshot and per-directory sizes.
- `f daemon clear-cache` is a safe, idempotent way to reset the daemon's persisted state
  (banner cache, size cache, stale socket).

## Validation evidence (all green on 0.6.2)

- `cargo fmt --all` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — no issues
- `cargo test --all-features -- --nocapture --test-threads=1` — 104 passed (5 suites)
- `cargo doc --no-deps` — passed
- `cargo build --release --locked` — passed
- `cargo publish` — uploaded `folder-auto-banner 0.6.2` to crates.io (verified via crates.io API; created_at 2026-06-09T20:51:37Z)
- `cargo package` artifact list — captured (50 files, 845.1 KiB / 526.3 KiB compressed)

### Manual smoke (this repo, `target/release/f`)

- New top-level file shows up in JSON output on the next `f` call (0 → 2 references).
- `src/` size: 288449 → 558626 after adding 200KB nested file, back to 288449 after
  delete.
- With daemon running, `Cargo.toml` size 1439 → 1478 after one-line edit, back to 1439
  after `git checkout`.
- New top-level file detected while daemon is warm; deleted file disappears immediately.
- `f daemon clear-cache` removes `banner_cache.json` and `dir_sizes.json` (and the stale
  socket) with no spurious shutdown warnings.

## Notes for maintainers

This is the real 0.6.2 release. `cargo publish` succeeded, the `v0.6.2` tag is pushed
to origin, and the GitHub release was created with the full body above and the Linux
x86_64 binaries attached. The aarch64-linux and macOS binaries are not attached because
the `release.yml` workflow was repeatedly canceled by the hosted runners during the
`Build release` step on this tag push (4 attempts, all canceled mid-build, in 3 different
matrix legs). The release was created via `gh release create` so the publish pipeline
didn't block on the flaky CI build; the Linux x86_64 binaries were built locally and
attached manually.

To add the aarch64-linux and macOS binaries after the fact, re-run the `release.yml`
workflow against `v0.6.2` once the hosted runners stop being canceled, then attach the
artifacts with `gh release upload v0.6.2 ...`.

## Assets in this release

- `f`, `fabd` — Linux x86_64 (built and attached manually because the
  GitHub Actions `release.yml` workflow was repeatedly canceled by the
  hosted runners during the `Build release` step on this tag push;
  the release was created via `gh release create` so the publish
  pipeline didn't block on a flaky CI build).

  aarch64-linux and macOS binaries are pending; they can be added by
  re-running the `release.yml` workflow against `v0.6.2` once the
  hosted runners stop being canceled mid-build.
