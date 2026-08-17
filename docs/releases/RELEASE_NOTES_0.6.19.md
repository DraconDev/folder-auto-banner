# folder-auto-banner 0.6.19

## Summary
Improves background directory-size freshness for large visited directories.

## What changed
- The daemon now periodically refreshes stale or placeholder child directory sizes for active roots instead of waiting for the next foreground navigation.
- Cached `4096`/`4.0k` directory inode fallback sizes are retried later rather than being treated as authoritative forever.
- Added tests covering placeholder-size retry and unmeasured fallback handling.

## Validation
- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`
- Live daemon smoke check for `/home/dracon/Downloads`: first view returned quickly with placeholders, after the active background refresh completed the same path returned in ~9-10 ms with 0 placeholder child directories and populated sizes such as `dracon-home2 456M`, `bmjmipppabdlpjccanalncobmbacckjn 4.1M`, and `full screen ref 25M`.
