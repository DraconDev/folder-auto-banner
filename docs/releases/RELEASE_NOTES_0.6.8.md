# Release notes — folder-auto-banner 0.6.8

Date: 2026-06-10

## Headline

More daemon and git-status efficiency, with better benchmarks for large-repo status work.

## What's in this release

### Performance improvements

- **Tighter git status pathspecs**: filtered git status collection now uses `dir/*` for displayed directories, so libgit2 only walks immediate children that the banner can display or aggregate. This avoids scanning every nested file under large directories when only top-level rows need status dots.
- **Leaner active watcher refreshes**: the daemon watcher now refreshes watched paths only when the active root set or active priority order changes. While a folder remains active and unchanged, the daemon no longer recursively re-walks watched descendants every second.
- **Skip git work for raw/oneline fallback**: when the daemon is unavailable and the requested output is raw or oneline, direct fallback no longer collects git metadata that will not be displayed.
- **Expanded benchmarks**: `cargo bench --bench performance` now includes `get_git_info manifest`, making large-repo git status regressions visible.

### Behavior preserved

- Rich banner output, JSON output, and numeric navigation behavior remain unchanged.
- Git status aggregation still covers the same depth-0 and depth-1 paths used by the banner.
- No new dependencies.

## Validation evidence (all green on 0.6.8)

- `cargo fmt --all` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — no issues
- `cargo test --all-features` — 108 passed (5 suites)
- `cargo doc --no-deps` — passed
- `cargo build --release --locked` — passed
- `cargo publish --dry-run` — packaged successfully before upload
- `cargo publish` — uploaded `folder-auto-banner 0.6.8` to crates.io

### Benchmark evidence

- `get_git_info manifest` measured about `88–91 ms` before the tighter pathspec work and about `83–86 ms` after the first filtered-pathspec change; final runs stayed in that range with no new regression.
- `DirSummary::scan /tmp` stayed around `380–396 ms` in the final benchmark run.
- `ProjectType::detect temp` improved in the final run to about `125 µs`.
- Warm daemon JSON smoke after cache warmup stayed around `7 ms` IPC latency in live tests.

## Notes for maintainers

This is the real 0.6.8 release. It keeps the 0.6.7 behavior stable while reducing unnecessary git and watcher work.
