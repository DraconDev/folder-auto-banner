## [0.6.24] - 2026-06-14

### Performance
- **Header-only file content probes** — the contents column for `.png` / `.jpg` / `.zip` / `.mp4` / `.mov` / `.mkv` / `.webm` now reads at most 64 KiB of each file instead of the entire file. This makes `f` in directories with many images, archives, or videos dramatically faster.
- **Skipped contents probe when hidden** — the per-file content probe is skipped entirely when the `contents` column is not in the effective column set, removing a per-item O(file size) cost from the hot path.
- **Bounded hidden-count scan** — the smart-truncation hidden counter is now O(N) total instead of O(N×M) per category.

## [0.6.23] - 2026-06-11

### Docs
- **Documentation version cleanup** — README and `f(1)` man page now consistently describe the current install flow, daemon management, shell-wrapper behavior, and background size-refresh behavior.

## [0.6.22] - 2026-06-11

### Docs
- **Cleaner user-facing documentation** — README, INSTALL, and the `f(1)` man page now describe the current install flow, daemon management, shell-wrapper behavior, and placeholder-size refresh behavior more consistently.

## [0.6.20] - 2026-06-11

### Fixes
- **Stale active-root size refresh queue** — stale or placeholder sizes now go through a bounded pending-refresh queue, so active roots are refreshed even when they are not among the first five recently watched folders.
- **Deduplicated background refreshes** — concurrent background size refreshes for the same root are coalesced, preventing repeated `du` work while preserving fast foreground responses.

## [0.6.19] - 2026-06-11

### Fixes
- **Active background directory-size refresh** — the daemon now periodically refreshes stale or placeholder directory sizes for active roots, so large visited directories such as `~/Downloads` populate child sizes without waiting for the next foreground navigation.
- **Placeholder-size retry** — cached `4096`/`4.0k` directory inode fallback sizes are no longer treated as authoritative; background refresh retries them later with a longer timeout.

## [0.6.18] - 2026-06-11

### Fixes
- **Uniform git branch bracket styling** — the closing `]` in dirty/clean branch badges now stays inside the same color/bold span as the branch name, preventing the right edge of badges like `[main*]` from rendering in a darker shade.

## [0.6.17] - 2026-06-11

### Performance
- **Non-blocking directory-size refresh** — banner responses now return immediately with cached sizes while stale or missing child directory sizes refresh in the background, preventing zoxide/chpwd navigation from blocking on large `du` work.
- **Faster logical size calculation** — displayed directory sizes now use `du -s -b`, which is much faster for normal workspace trees and avoids falling back to the 4 KiB directory inode size for large directories.
- **Warmer size cache prepopulation** — warm precompute requests now schedule background size refreshes so parent and child banners are populated before the next navigation.

### Notes
- The first cold view of a very large directory returns quickly and may show cached placeholders until the background size refresh completes; subsequent warm calls use populated single-digit-millisecond cache entries.
- Warm cache hits remain single-digit milliseconds after pre-warm.


## [0.6.16] - 2026-06-11

### Performance
- **Heavier project-insight pruning** — project-insight scans now skip known heavy directories before descent, so `target`, `.git`, `node_modules`, and similar directories do not slow TODO/code-metric collection in large workspace trees.
- **Leaner large-file insight parsing** — very large files are counted without full TODO/LOC parsing, and newline counts are computed without materializing every line.
- **Port-detection shell cache** — `ss -tlnp` output is cached briefly to avoid repeated shell-outs during warm pre-warming bursts.
- **Filesystem-local size refresh** — displayed directory sizes use `du -s --bytes -x` so size refresh stays on the same filesystem.

### Notes
- Warm cache hits remain single-digit milliseconds after pre-warm.
- Cold scans for very large directories are still bounded by accurate directory-size refresh work, but project-insight and port-detection overhead is lower.


## [0.6.15] - 2026-06-10

### Performance
- **Reliable child pre-warming** — warm requests now use one short-lived daemon connection per path and are sent before the CLI exits, so pre-warmed child directories actually get cached.
- **Wider pre-warm coverage** — the client now warms the parent, grandparent, and up to 30 immediate children of the current directory, which covers large `~/Dev` trees much better than the previous 5-child limit.
- **Bounded cold-size refresh** — directory size refresh uses a bounded `du` timeout to reduce first-hit latency on very large trees while keeping normal directory sizes accurate.
- **Cleaner daemon IPC failures** — daemon-side compute errors now return structured IPC errors instead of leaving the client to read a half-closed stream.
- **Clearer missing-path errors** — relative paths that do not exist now fail directly with `No such file or directory: Dev` instead of producing a confusing `send_and_recv` error.

### Notes
- Warm cache hits remain single-digit milliseconds.
- The first cold view of a large directory still has to compute directory sizes, git status, TODOs, ports, and metrics, but the worst-case size refresh is now bounded.


## [0.6.14] - 2026-06-10

### Performance
- **More accurate bounded size refresh** — directory-size refresh keeps its timeout bounded while preserving normal directory-size accuracy better than the tighter 0.6.12 refresh window.

### Notes
- Warm cache hits remain single-digit milliseconds.
- No banner, JSON, or navigation behavior changes.


## [0.6.13] - 2026-06-10

### Performance
- **Expanded pre-warm coverage** — the client now warms the parent, grandparent, and up to 30 immediate children of the current directory.
- **Reliable warm-request delivery** — warm requests are sent on short-lived daemon connections before the CLI exits.

### Notes
- Preserves banner output, JSON output, and numeric navigation behavior.


## [0.6.12] - 2026-06-10

### Performance
- **Tighter cold-size refresh** — directory size refresh keeps a bounded `du` timeout to reduce first-hit latency on very large trees.
- **Cleaner daemon IPC failures** — daemon compute errors now return structured IPC errors instead of leaving the client to read a half-closed stream.
- **Clearer missing-path errors** — relative paths that do not exist now fail directly with `No such file or directory: Dev` instead of producing a confusing `send_and_recv` error.

### Notes
- Warm cache hits remain single-digit milliseconds.
- The first cold view of a large directory still has to compute directory sizes, git status, TODOs, ports, and metrics, but the worst-case size refresh is now bounded more tightly.


## [0.6.11] - 2026-06-10

### Performance
- **Smarter pre-warming of nearby directories** — after a banner is rendered, the client now warms the parent, the grandparent, and the first few immediate children of the current directory, so moving up or stepping into a sibling/child is served from the daemon cache instead of recomputing.

### Notes
- The total number of background warm requests is bounded to a small set of paths to avoid expensive background scans.
- Preserves banner output, JSON output, and numeric navigation behavior.


## [0.6.10] - 2026-06-10

### Packaging
- **Corrected crates.io repository metadata** — package metadata now points to `https://github.com/DraconDev/folder-auto-banner` and includes homepage/documentation links.

### Notes
- crates.io does not allow repository metadata to be changed for already-published versions, so older published versions may still show the previous incorrect repository URL. New installs via `cargo install folder-auto-banner` use the corrected latest release.
- No runtime behavior changes.


## [0.6.9] - 2026-06-10

### Performance
- **Global uid/gid name caches** — `/etc/passwd` and `/etc/group` are loaded once per process instead of reparsing them for every directory scan.
- **Lower-allocation permission formatting** — file mode rendering now builds the 10-character mode string directly instead of using `format!` per row.
- **Leaner active watcher maintenance** — inactive watcher cleanup now reuses the active-root snapshot from the periodic refresh when available and avoids extra mutex work.

### Notes
- Preserves banner output, JSON output, and numeric navigation behavior.
- No new dependencies.


## [0.6.8] - 2026-06-10

### Performance
- **Tighter git status pathspecs** — filtered git status collection now limits directory status walks to immediate children (`dir/*`) instead of asking libgit2 to scan every nested file under displayed directories.
- **Leaner daemon watcher refreshes** — the active-folder watcher now refreshes watched paths only when the active root set or priority order changes, avoiding repeated recursive directory scans while idle.
- **Skip git work for raw/oneline fallback** — when the daemon is unavailable and output does not need git metadata, direct fallback avoids collecting git status entirely.
- **Expanded benchmarks** — added a manifest-repository git-info benchmark so large-repo status collection regressions are visible.

### Notes
- Preserves rich banner, JSON, and navigation behavior.
- No new dependencies.


## [0.6.7] - 2026-06-10

### Performance
- **Avoid duplicate project-insight scans** — TODO counts and code metrics now share one bounded tree walk when both are enabled, reducing cold daemon scans and file reads.
- **Reuse rendered item contents** — rich banner rendering now computes directory counts and file content previews once per item, then reuses those values for column sizing and row output.
- **Leaner project-type detection** — marker-file checks use direct path probes before reading directory entries, making repeated project detection faster.

### Notes
- Preserves existing banner output and daemon freshness behavior.
- No new dependencies.


## [0.6.6] - 2026-06-10

### Fixed
- **`f N` navigation bug** — when running `f N` (e.g. `f 40`), the daemon was being asked for the banner of the path `"40"` (the number string) instead of the current directory. This caused `f N` to return an empty path or open the wrong file when the number didn't match a real directory. The path is now resolved correctly: numeric navigation always uses the current directory, matching how the shell function invokes `f banner N`.

### Notes
- Preserves 0.6.5 behavior in all other respects.
- No new dependencies.


## [0.6.2] - 2026-06-09

### Fixed
- **Fresh folder information** — `f` now consistently shows the latest folder contents and sizes:
  - Daemon cache hits validate the cached folder snapshot against a fresh shallow scan before returning, so out-of-band edits are no longer masked by the TTL.
  - Displayed directory sizes are refreshed when their mtime changes, so nested folder content edits immediately update parent folder size information.
  - Directory size cache tracks mtimes, so persisted sizes from a previous daemon run can no longer shadow fresh data.
- **Daemon clear-cache** now also clears `banner_cache.json` and `dir_sizes.json` (previously only the cache directory), with no spurious shutdown warning.
- Bench harness `benches/performance.rs` now references the real crate name (`folder_auto_banner`) instead of the old `fab_lib`.
- rustdoc HTML warning in `port_usage` for `<pid>` token.

### Notes
- No user-visible behavior changes beyond the freshness fix and the expanded `f daemon clear-cache`.
- No new dependencies.

## [0.4.0] - 2024-05-31

### Added
- **Config file** (`~/.config/fab/config.toml`) with all display preferences
- `f config` command — opens config in $EDITOR
- `f daemon stop/status` commands — daemon management
- Two-row banner layout for better readability
- Dynamic truncation for narrow terminals
- Git enhancements: last commit time, commits today, branch count
- Languages breakdown with percentages
- Build timing display
- Cached test results display
- Permission modes: rwx, octal, disable
- Column selection (show/hide columns)
- Feature toggles (git, build, todos, languages, ports, docker)

### Changed
- **Design principle**: CLI flags = actions, Config file = preferences
- Simplified CLI to focus on core features
- Improved banner layout with two rows

### Fixed
- Daemon log spam for non-existent directories
- Dead symlink handling in daemon watcher
- Install script properly stops daemon before reinstall
- Broken symlink display (✗→ indicator)
- Full resolved symlink paths

## [0.3.0] - 2024-05-30

### Changed
- **Major simplification**: Removed 15+ commands that duplicate existing tools
- Renamed binary from `fm` to `f` for faster typing
- Focus on core feature: directory listing with instant context

### Removed
- File operations: `cp`, `mv`, `rm`, `trash`, `open` — people have their own tools
- Clipboard: `yank`, `paste`, `clipboard` — niche use case
- Navigation: `pin`, `unpin`, `pins`, `jump`, `root` — redundant with frecency
- Sessions: `save-session`, `load-session`, `sessions`, `delete-session` — over-engineering
- Other: `diff`, `do`, `peek`, `stats`, `config` — niche or over-engineered
- Flags: `--no-build-check`, `--no-todos`, `--no-ports`, `--no-docker`, `--no-metrics` — use env vars

### Added
- `--compact` flag for less info
- `--verbose` flag for more info
- Broken symlink indicator (✗→)
- Full resolved symlink paths

## [0.2.0] - 2024-05-30

### Added
- `--hidden` flag to show dotfiles
- `--filter` flag to filter items by pattern/extension
- `--max N` flag to limit items displayed
- `--group` flag to group items by type
- Config file support
- `fm config` command
- `fm root` and `fm uninstall-hook` commands
- `NO_COLOR` environment variable support
- Path protection for trash, mv, cp commands
- Shared library crate (`fab-lib`)
- Unit and integration tests

### Fixed
- Dry-run flag now works for all destructive commands
- Daemon mutex poisoning recovery
- Shell injection prevention
- Copy verification before deletion
- Symlink loop prevention

## [0.1.1] - 2024-05-25

### Added
- Complete CLI with 28 commands
- Rich terminal output with Unicode icons
- TTY detection (rich/raw/JSON modes)
- State persistence
- Shell integration hooks
- Shell completions
- GitHub Actions CI/CD

## [0.1.0] - 2024-05-24

### Added
- Initial release
- Basic banner display
- Project type detection
- Git integration
