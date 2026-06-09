## [0.6.3] - 2026-06-09

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
