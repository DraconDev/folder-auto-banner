# folder-auto-banner 0.6.18

## Summary
Fixes inconsistent branch-badge coloring in rich banner output.

## What changed
- The closing `]` in git branch badges such as `[main*]` now remains inside the same color/bold span as the branch text, so the right edge no longer renders in a darker shade.
- Added unit coverage for dirty and clean branch-badge ANSI output.

## Validation
- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`
- Verified rendered banner output for `~/Dev/browser-extensions-shared/extension-research` uses a uniform branch-badge style.
