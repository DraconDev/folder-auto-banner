# Release notes — folder-auto-banner 0.6.6

Date: 2026-06-10

## Headline

`f N` navigation now uses the correct directory when resolving the item index.

## What's in this release

### Fixed: `f N` opened the wrong file or nothing

When running `f N` (e.g. `f 40`), the CLI was passing the number `"40"` to the daemon as a path to scan. Since `"40"` is not a real directory, the daemon either returned no banner or scanned the wrong path, causing the wrong file to be opened (or an empty path that the shell wrapper silently ignored).

In 0.6.6, the path resolution in `run_banner` now checks for numeric navigation **before** resolving the path. When the first positional argument is a number, the directory to scan is the current working directory (or the explicit non-numeric path if one was given), and the number is treated as the item index.

### Regression test added

A new integration test `test_navigate_by_number_matches_banner` creates a temp directory with known files, gets the JSON banner, and verifies that `f N` returns the exact same path shown at position `[N]` in the banner.

## What end users should expect

- `f 40` in a folder with a banner that shows `COMPARATIVE_AUDIT.md` at position [40] will now open `COMPARATIVE_AUDIT.md`.
- `f N` no longer fails silently or opens the wrong file.
- The `.txt` files that the user noted "opened an empty page" were actually 0-byte files — that is not a navigation bug, but the correct file was being opened.

## Validation evidence (all green on 0.6.6)

- `cargo fmt --all` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — no issues
- `cargo test --all-features` — 105 passed (5 suites), including the new `test_navigate_by_number_matches_banner`
- `cargo doc --no-deps` — passed
- `cargo build --release --locked` — passed
- `cargo publish --dry-run` — packaged successfully before upload
- `cargo publish` — uploaded `folder-auto-banner 0.6.6` to crates.io

### Manual smoke (live local install)

- `~/.local/bin/f --version` reports `f 0.6.6`.
- In `~/Dev/dracon-code`:
  - `f` banner shows `COMPARATIVE_AUDIT.md` at `[40]`
  - `f banner 40` returns `/home/dracon/Dev/dracon-code/COMPARATIVE_AUDIT.md`
  - `f banner 41` returns `NEXT_FOCUS_ROADMAP.md`
  - `f banner 23` returns `SPEC.md`

## Notes for maintainers

This is the real 0.6.6 release. It fixes the `f N` navigation bug where the number was being treated as a directory path instead of an item index.

If the GitHub Actions `release.yml` workflow is canceled again during `Build release`, create the GitHub release manually with `gh release create v0.6.6 --notes-file RELEASE_NOTES_0.6.6.md` and attach Linux x86_64 binaries built locally.

## Assets in this release

- `f`, `fabd` — Linux x86_64 (built locally and attached manually if the `release.yml` workflow is canceled again).

aarch64-linux and macOS binaries remain optional follow-up assets.
