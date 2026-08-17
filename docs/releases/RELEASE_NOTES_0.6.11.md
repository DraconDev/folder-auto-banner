# Release notes — folder-auto-banner 0.6.11

Date: 2026-06-10

## Headline

Smarter pre-warming of nearby directories to keep the banner instant when moving around large trees like `~/Dev`.

## What's in this release

### Performance improvement

- After a banner is rendered, the client now warms the parent directory, the grandparent directory, and a small number of immediate children of the current directory.
- This means `cd ..`, `cd ../..`, and `cd <child>` are served from the daemon cache instead of recomputing the banner from scratch.

### Behavior preserved

- Rich banner output, JSON output, and numeric navigation behavior remain unchanged.
- The total number of background warm requests is bounded to a small set of paths, so background scanning cost stays modest.
- No new dependencies.

## Validation evidence

- `cargo fmt --all` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — no issues
- `cargo test --all-features` — 108 passed (5 suites)
- `cargo doc --no-deps` — passed
- `cargo build --release --locked` — passed
- `cargo publish --dry-run` — packaged successfully before upload
- `cargo publish` — uploaded `folder-auto-banner 0.6.11` to crates.io

### Timing evidence

- Warm cached daemon JSON smoke stayed around `7–9 ms` after cache warmup.
- Cold daemon first compute on `~/Dev` was around `1.2–1.4 s` (initial directory/git/insight work); the new warming keeps subsequent visits to immediate children/parents/grandparents in the single-digit millisecond range from cache.

## Notes for maintainers

This release keeps behavior stable while reducing how often the user sees a cold compute when moving around a directory tree.
