# folder-auto-banner 0.6.16

## Summary
Performance audit follow-up for very large directories such as `~/Dev` and `~/Dev/dracon-platform`.

## What changed
- Project-insight scans now skip known heavy directories before descent, avoiding slow recursive work in `target`, `.git`, `node_modules`, and similar directories.
- Large-file TODO/LOC parsing is capped; newline counts are computed without materializing every line.
- Port detection briefly caches `ss -tlnp` output to avoid repeated shell-outs during warm pre-warming bursts.
- Displayed directory-size refresh uses filesystem-local `du -s --bytes -x`.

## Validation
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- Fresh daemon cold/warm timing for `/home/dracon/Dev` and `/home/dracon/Dev/dracon-platform`
- Missing relative path handling verified without daemon IPC EOF errors
