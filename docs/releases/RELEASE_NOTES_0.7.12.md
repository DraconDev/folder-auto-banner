# Release Notes — v0.7.12

**Date**: 2026-08-31

## Summary

This release fixes a shell responsiveness issue when the current directory
contains hundreds of thousands of entries, such as `/tmp`. Banner scans now
bound the raw directory iterator instead of continuing to enumerate the whole
directory after the display limit has been reached.

## Changes

- Normal banner scans stop after 500 observed entries and mark the result as
  truncated; sampled counts are displayed as `500+`.
- Project-type detection, Python build-file discovery, child directory counts,
  inline previews, daemon watcher refreshes, and descendant cache freshness
  checks are bounded as well.
- Older daemon/cache payloads without the new `truncated` field remain
  readable.
- Added regression coverage for 501-entry directories and legacy cache data.

## Validation

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --release --locked`
- `cargo test --all-targets`
- `bash -n install.sh`

## Installation

```bash
cargo install folder-auto-banner --version 0.7.12 --locked --force
```

Or install from a checkout with:

```bash
./install.sh
```
