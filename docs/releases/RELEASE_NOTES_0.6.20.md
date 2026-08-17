# folder-auto-banner 0.6.20

## Summary
Improves active background directory-size refresh behavior so visited/pre-warmed directories do not remain stuck on placeholder `4.0k`/`4096` sizes.

## What changed
- Stale or placeholder child directory sizes are now added to a bounded pending-refresh queue.
- The background refresh loop processes that queue first, so active roots are refreshed even when they are not in the first few recently watched folders.
- Background refreshes for the same root are deduplicated to avoid repeated `du` work.
- Placeholder directory inode sizes are still retried instead of being treated as authoritative.

## Validation
- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`
