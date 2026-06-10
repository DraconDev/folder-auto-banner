# Release notes — folder-auto-banner 0.6.10

Date: 2026-06-10

## Headline

Corrected crates.io package repository links.

## What's in this release

### Packaging metadata

- Set `repository` to `https://github.com/DraconDev/folder-auto-banner`.
- Added `homepage` pointing to `https://github.com/DraconDev/folder-auto-banner`.
- Added `documentation` pointing to `https://docs.rs/folder-auto-banner`.

### Important note about older versions

crates.io does not allow repository metadata to be changed for already-published versions. Older published versions may still show the previous incorrect repository URL. The corrected metadata applies to the new `0.6.10` release and to `cargo install folder-auto-banner`, which installs the latest version by default.

## Validation evidence

- `cargo fmt --all` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — no issues
- `cargo test --all-features` — 108 passed (5 suites)
- `cargo doc --no-deps` — passed
- `cargo build --release --locked` — passed
- `cargo publish --dry-run` — packaged successfully before upload
- `cargo publish` — uploaded `folder-auto-banner 0.6.10` to crates.io
- Live crates.io API checks verified `0.6.10` repository/homepage/documentation metadata.
- GitHub release `v0.6.10` is live with `f` and `fabd` assets.
- Local installs updated to `f 0.6.10`.

## Notes for maintainers

This is a packaging-only release to make crates.io links point to the correct repository.
