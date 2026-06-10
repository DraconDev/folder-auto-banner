# Release notes — folder-auto-banner 0.6.9

Date: 2026-06-10

## Headline

Small, low-risk runtime cleanup after the 0.6.8 git-status and watcher improvements.

## What's in this release

### Performance improvements

- **Global uid/gid name caches**: `/etc/passwd` and `/etc/group` are now loaded once per process and reused across directory scans instead of being reparsed on every banner request.
- **Lower-allocation permission formatting**: Unix mode rendering now builds the 10-character permission string directly, avoiding per-row `format!` allocation for every displayed item.
- **Leaner active watcher maintenance**: the daemon watcher cleanup path reuses the active-root snapshot from the periodic refresh when available, reducing mutex churn while maintaining the same watched-path behavior.

### Behavior preserved

- Rich banner output, JSON output, and numeric navigation behavior remain unchanged.
- Directory scans still resolve owners/groups, permissions, symlink metadata, project type, todos, code metrics, ports, and docker info the same way.
- No new dependencies.

## Validation evidence (all green on 0.6.9)

- `cargo fmt --all` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — no issues
- `cargo test --all-features` — 108 passed (5 suites)
- `cargo doc --no-deps` — passed
- `cargo build --release --locked` — passed
- `cargo publish --dry-run` — packaged successfully before upload
- `cargo publish` — uploaded `folder-auto-banner 0.6.9` to crates.io

### Benchmark/timing evidence

- `DirSummary::scan /tmp` improved in the final benchmark run from about `523–622 ms` to about `440–477 ms`.
- `format_size_compact` improved in the final benchmark run from about `50–56 ns` to about `44–47 ns`.
- Warm daemon JSON smoke stayed around `7 ms` IPC latency in live tests.
- Cold daemon/banner smoke stayed in the same sub-second range as 0.6.8; the main remaining cold-start cost is still initial directory/git insight computation, not daemon IPC.

## Notes for maintainers

This release keeps the 0.6.8 behavior stable while shaving avoidable per-scan allocation and parsing costs.
