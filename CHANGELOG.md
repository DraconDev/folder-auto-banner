## [0.6.13] - 2026-06-10

### Performance
- **Reliable child pre-warming** — warm requests now use one short-lived daemon connection per path and are sent before the CLI exits, so the pre-warmed child directories actually get cached.
- **Wider pre-warm coverage** — the client now warms the parent, grandparent, and up to 30 immediate children of the current directory, which covers large `~/Dev` trees much better than the previous 5-child limit.
- **Tighter cold-size refresh** — directory size refresh keeps a bounded `du` timeout to reduce first-hit latency on very large trees.
- **Cleaner daemon IPC failures** — daemon-side compute errors now return structured IPC errors instead of leaving the client to read a half-closed stream.
- **Clearer missing-path errors** — relative paths that do not exist now fail directly with `No such file or directory: Dev` instead of producing a confusing `send_and_recv` error.

### Notes
- Warm cache hits remain single-digit milliseconds.
- The first cold view of a large directory still has to compute directory sizes, git status, TODOs, ports, and metrics, but the worst-case size refresh is now bounded more tightly.



### Performance
- **Faster first-size refresh** — directory size refresh now uses a tighter `du` timeout, reducing cold banner latency for large trees while preserving cached sizes on subsequent visits.
- **Cleaner daemon IPC failures** — daemon compute errors now return structured IPC errors instead of leaving the client to read a half-closed stream.
- **Clearer missing-path errors** — relative paths that do not exist now fail before daemon IPC, avoiding confusing `send_and_recv` errors.

### Notes
- Warm cache hits remain single-digit milliseconds.
- The first cold view of a large directory still has to compute directory sizes, git status, TODOs, ports, and metrics, but the worst-case size refresh is now bounded more tightly.



### Performance
- **Smarter pre-warming of nearby directories** — after a banner is rendered, the client now warms the parent, the grandparent, and the first few immediate children of the current directory, so moving up or stepping into a sibling/child is served from the daemon cache instead of recomputing.

### Notes
- The total number of background warm requests is bounded to a small set of paths to avoid expensive background scans.
- Preserves banner output, JSON output, and numeric navigation behavior.



### Packaging
- **Corrected crates.io repository metadata** — package metadata now points to `https://github.com/DraconDev/folder-auto-banner` and includes homepage/documentation links.

### Notes
- crates.io does not allow repository metadata to be changed for already-published versions, so older published versions may still show the previous incorrect repository URL. New installs via `cargo install folder-auto-banner` use the corrected latest release.
- No runtime behavior changes.



### Performance
- **Global uid/gid name caches** — `/etc/passwd` and `/etc/group` are loaded once per process instead of reparsing them for every directory scan.
- **Lower-allocation permission formatting** — file mode rendering now builds the 10-character mode string directly instead of using `format!` per row.
- **Leaner active watcher maintenance** — inactive watcher cleanup now reuses the active-root snapshot from the periodic refresh when available and avoids extra mutex work.

### Notes
- Preserves banner output, JSON output, and numeric navigation behavior.
- No new dependencies.



### Performance
- **Tighter git status pathspecs** — filtered git status collection now limits directory status walks to immediate children (`dir/*`) instead of asking libgit2 to scan every nested file under displayed directories.
- **Leaner daemon watcher refreshes** — the active-folder watcher now refreshes watched paths only when the active root set or priority order changes, avoiding repeated recursive directory scans while idle.
- **Skip git work for raw/oneline fallback** — when the daemon is unavailable and output does not need git metadata, direct fallback avoids collecting git status entirely.
- **Expanded benchmarks** — added a manifest-repository git-info benchmark so large-repo status collection regressions are visible.

### Notes
- Preserves rich banner, JSON, and navigation behavior.
- No new dependencies.



### Performance
- **Avoid duplicate project-insight scans** — TODO counts and code metrics now share one bounded tree walk when both are enabled, reducing cold daemon scans and file reads.
- **Reuse rendered item contents** — rich banner rendering now computes directory counts and file content previews once per item, then reuses those values for column sizing and row output.
- **Leaner project-type detection** — marker-file checks use direct path probes before reading directory entries, making repeated project detection faster.

### Notes
- Preserves existing banner output and daemon freshness behavior.
- No new dependencies.



### Fixed
- **`f N` navigation bug** — when running `f N` (e.g. `f 40`), the daemon was being asked for the banner of the path `"40"` (the number string) instead of the current directory. This caused `f N` to return an empty path or open the wrong file when the number didn't match a real directory. The path is now resolved correctly: numeric navigation always uses the current directory, matching how the shell function invokes `f banner N`.

### Notes
- Preserves 0.6.5 behavior in all other respects.
- No new dependencies.


### Fixed
- **No spurious cache invalidations from VCS or build internals** — the inotify watcher now skips `.git`, `.hg`, `.svn`, `target`, `node_modules`, `.next`, `dist`, `build`, `.cache`, `.parcel-cache`, and `.turbo` directories. Previously, the daemon's own git operations (creating `.git/index.lock` and `.git/objects/tmp_object_*` files) would trigger the watcher and invalidate the cached banner within seconds of a request, preventing the size cache from ever persisting.
- **Descendant changes no longer invalidate the parent banner** — the watcher now only invalidates the banner cache when the event is on the root directory itself. Events in descendants (e.g. a test runner cleaning up directories deep inside a child project) only prune the size cache for the affected root, keeping the banner's item listing valid.

### Notes
- Preserves 0.6.4 persistence and active-folder watcher behavior.
- No new dependencies.


### Fixed
- **Fast persisted directory sizes** — directory size data is now persisted with mtimes and reloaded on daemon restart, so large parent folders such as `~/Dev` do not need to recompute every child size after the daemon restarts.
- **Bounded large-folder latency** — displayed directory sizes are computed with a bounded worker pool, so the first cold request for a large folder is kept to a low-seconds worst case instead of serially waiting up to one second per child directory.

### Notes
- Preserves 0.6.3 freshness and active-folder watcher behavior.
- No new dependencies.


### Fixed
- **Snappy fresh daemon** — replaced the expensive shallow validation scan on every daemon cache hit with active-folder inotify watching:
  - Requested folders become active and are watched, with bounded recursive coverage for descendant files and directories.
  - Nested create/delete/modify/move events invalidate the banner cache, so folder contents and displayed directory sizes refresh without waiting for the TTL.
  - A cheap root-mtime guard remains as a fallback for changes that do not emit an actionable inotify event.
  - Newly active folders are prioritized so hot folders get watched promptly even when many folders are cached.
- Persisted daemon caches are treated as expired on daemon restart, so the first request after restart recomputes the banner and refreshes size data.

### Notes
- Preserves the 0.6.2 freshness guarantees while restoring fast cache-hit latency.
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
