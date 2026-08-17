# Release notes — folder-auto-banner 0.6.7

Date: 2026-06-10

## Headline

Cold banner generation is faster because project insights and rich-render metadata are scanned once and reused.

## What's in this release

### Performance improvements

- **Combined TODO and code-metric scans**: TODO counts and code metrics both need a bounded text-file scan. The daemon now scans those insights in a single pass when both are enabled, avoiding a second tree walk and a second round of file reads.
- **Reused rich-banner contents metadata**: `output_rich` previously computed directory item counts and file content previews once to size columns, then computed them again while rendering rows. It now precomputes this metadata once per displayed item and reuses it for both column sizing and row rendering.
- **Faster project-type detection**: marker-file detection now probes common marker paths directly before falling back to a directory read, reducing repeated detection work for normal project roots.

### Regression coverage

- Existing tests continue to cover banner rendering, navigation, daemon behavior, and filesystem metadata.
- The performance benchmark suite now includes `ProjectType::detect temp` and a bounded temp directory scan so future regressions are visible in `cargo bench --bench performance`.

## What end users should expect

- Same banner output and navigation behavior as 0.6.6.
- Faster cold scans for folders with many files, especially when TODO counts and code metrics are enabled.
- Faster rich banner rendering for folders with many directories or files that need content previews.

## Validation evidence (all green on 0.6.7)

- `cargo fmt --all` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — no issues
- `cargo test --all-features` — 106 passed (5 suites)
- `cargo doc --no-deps` — passed
- `cargo build --release --locked` — passed
- `cargo publish --dry-run` — packaged successfully before upload
- `cargo publish` — uploaded `folder-auto-banner 0.6.7` to crates.io

### Benchmark evidence

- `DirSummary::scan /tmp` improved from about `724–1002 ms` before the optimization to about `390–400 ms` after, with Criterion reporting `Performance has improved` on the first post-change run and stable `~394 ms` after the final render reuse change.
- Rich/live smoke checks stayed in the single-digit millisecond range for cached daemon responses: `FAB_PROFILE=1 ~/.local/bin/f --json /home/dracon/Dev/dracon-code` reported about `7–8 ms` IPC latency after cache warmup.

## Notes for maintainers

This is the real 0.6.7 release. It keeps the 0.6.6 behavior stable while reducing duplicate work in cold scans and rich rendering.
