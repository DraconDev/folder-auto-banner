# cfm tasks

---

## ✅ Done

### Header redesign
- [x] Replace `12M` with `💾 12MB` (add icon + full unit for clarity)
- [x] Replace `15 files` with `📄 15 files` (correct file icon)
- [x] Replace `5 dirs` with `📂 5 dirs` (correct folder icon)
- [x] Change `16k LOC` to `16k lines` (clearer than jargon)
- [x] Drop LOC breakdown `(md: 10k, rs: 5.9k, no-ext: 149)` from header
- [x] Drop commit hash from header
- [x] Drop `FILES:1 DIRS:src` from header (unclear meaning)
- [x] Drop `DELTA:` label from git delta, keep just `+13 -6`
- [x] Show `*` suffix on branch name when repo is dirty
- [x] Add `✓ clean` when repo is clean
- [x] Fix symlinks: show target's contents (line count for files, item count for dirs)
- [x] Build, install, verify output

### Daemon (cfmd)
- [x] Design daemon architecture (IPC, lifecycle, data structures)
- [x] Create daemon binary (cfmd) with Unix socket server
- [x] Implement inotify-based directory watching (recompute on change)
- [x] Implement pre-computation: directory sizes, git status, TODO/LOC
- [x] Implement IPC protocol (request/response for cached data)
- [x] Modify banner to read from daemon cache (fallback to direct scan)
- [x] Add daemon management commands (start, stop, status, restart, clear-cache)
- [x] Add systemd/user service file for auto-start
- [x] Add resource limits (nice=10, ionice=idle)
- [x] Test: daemon startup, cache population, banner speed

### UX improvements
- [x] Alternating row tints (subtle gray on odd/even rows)
- [x] Color scheme: dirs=blue, scripts=red, size/contents=orange
- [x] Directory size: show `-` instead of misleading inode size
- [x] Symlinks: follow target for contents and metadata

### Table / Display
- [x] Replaced manual box-drawing with comfy-table v6
- [x] Custom preset matching `│`, `─`, `├`, `┼`, `┤` style
- [x] Responsive column widths (caps at 120, min 60)
- [x] Merge icon into name column
- [x] `max_height(1)` prevents row wrapping; `truncate_ansi()` handles overflow
- [x] Switched to compact lsd/exa-style layout (no borders, aligned text)
- [x] Column order: `PERM OWNER GROUP SIZE DATE NAME` (matches lsd/exa)

### File-Type Icons
- [x] 3-tier icon lookup: exact filename → extension → type fallback
- [x] 100+ mappings: `Cargo.toml` → 🦀, `install.sh` → 🐚, `.lock` → 🔒, etc.
- [x] Separate `src/icon.rs` module with unit tests

### Permissions & Owner/Group
- [x] `DirEntry` fields: `is_exec`, `perms`, `owner`, `group`, `symlink_target`
- [x] Unix permissions via `std::os::unix::fs::PermissionsExt::mode()` → `drwxr-xr-x`
- [x] Owner/group resolved from `/etc/passwd` and `/etc/group`
- [x] Symlink target resolution via `std::fs::read_link`

### Compact Size/Date Formats
- [x] Size: exa-style `4.3k`, `1.1k`, `983` (no "B" suffix)
- [x] Date: exa-style `27 May 23:42` (day month hour:minute)

### Git Integration
- [x] Header shows branch name + dirty state (yellow when dirty, blue when clean)
- [x] Header shows `*N` modified, `+N` staged, `?N` untracked, `↑N` ahead, `↓N` behind
- [x] `FileStatus` enum with per-file icons and colors
- [x] `file_statuses: HashMap<String, FileStatus>` populated during scan
- [x] Per-file git status lookup by relative path

### TTY Detection
- [x] Colors only emit when stdout is a real terminal
- [x] Non-tty output is plain text (no broken escape codes)

### Install / Shell
- [x] Consolidated 4 redundant install scripts into 1
- [x] `install.sh` tears down old binary before copying new one
- [x] Cleanup handles all hook-name variants
- [x] Hook fires on new tabs/shell startup (not just `cd`)
- [x] Bash `PROMPT_COMMAND` support
- [x] Hook names unified to `_cfm_hook` everywhere

### Docs / CI
- [x] `INSTALL.md` — removed hardcoded paths, added bash section
- [x] `README.md` — updated with 30 commands, architecture overview, 76 tests
- [x] `release.yml` — fixed step ordering, switched to `softprops/action-gh-release@v2`

### Critical Fixes (audit-driven)
- [x] Dry-run flag: wired `--dry_run` through to `run_mv`, `run_cp`, `run_rm`, `run_trash`, `run_open`, `run_do_cmd`
- [x] Daemon mutex poisoning: replaced 18 `.unwrap()` on `Mutex::lock()` with `unwrap_or_else(|e| e.into_inner())`
- [x] Shell injection: fixed `do_cmd.rs` to pass paths as single args instead of splitting on whitespace
- [x] Data loss risk: added copy verification (size check for files, item count for dirs) before deletion in cross-device moves

### Architecture (audit-driven)
- [x] Created `cfm-lib/` library crate with 12 shared modules
- [x] Extracted file metadata I/O from `banner.rs` into `file_metadata.rs`
- [x] Extracted `run_daemon()` from `cli/mod.rs` into `cmd/daemon_mgmt.rs`
- [x] Created `BannerOptions` struct (11 params → 1 struct)
- [x] Created `CpOptions` struct (7 params → 1 struct)
- [x] Consolidated 3 duplicate `Session` types into one canonical struct in `state/mod.rs`

### Code Quality (audit-driven)
- [x] Extracted ~500 lines of duplication into `src/utils.rs` (copy, delete, sanitize, timeout, constants)
- [x] Fixed error handling: replaced `let _ =` discards in `daemon_client.rs` with logging
- [x] Added cache write failure logging in `fs/mod.rs`
- [x] Removed unused `thiserror` dependency from `Cargo.toml`
- [x] Wired up Config command: `fm config` now reads/writes `~/.config/cfm/config.toml`

### Safety (audit-driven)
- [x] Symlink loop prevention: `copy_dir_recursive` now skips symlinks
- [x] Extended `is_protected_path()` to cover `~/.ssh`, `~/.gnupg`, `~/.config`, `~/.mozilla`, `~/.docker`
- [x] Cross-device moves verify copy success before deleting source

### Testing (audit-driven)
- [x] Grew test count from 14 to 76 (28 unit + 29 unit + 19 integration)
- [x] Added unit tests for `utils.rs`, `cache/mod.rs`, `state/mod.rs`
- [x] Added integration tests for config, mv, cp, rm, trash, open, do, peek, root, daemon

### Features (audit-driven)
- [x] Implemented `fm root` — finds git repo root
- [x] Implemented `fm uninstall-hook` — removes shell hooks from config files

---

## 🔴 Remaining Critical

### Daemon mutex (partial)
- [x] Audit `Mutex` usage for potential deadlocks (nested locks, lock ordering) — all safe, drops before re-acquire
- [x] Add logging when mutex poisoning occurs so failures are visible

### Shell injection (partial)
- [x] `file_metadata.rs:count_sqlite_tables`: uses `.arg()` not shell interpolation — safe
- [x] Review all `Command::new()` call sites — all use hardcoded command names

---

## 🟠 Remaining Architecture

### Library crate migration (partial)
- [x] Update `cfmd` binary to use `cfm-lib` instead of duplicate modules
- [x] Remove redundant `mod` declarations from `daemon.rs`
- [x] Remove `#![allow(dead_code)]` from both `main.rs` and `daemon.rs`
- [x] Fix any dead code warnings that surface

### Banner decoupling (partial)
- [ ] Make `output_rich()` consume pre-extracted data, not read files directly
- [ ] Split `banner.rs` into smaller focused modules

### Daemon decoupling
- [ ] Move daemon implementation out of `daemon.rs` into `daemon/` module (or `cfm-lib`)
- [ ] Keep `daemon.rs` as a thin entry point that calls into the library

### DirSummary god function
- [ ] Extract `scan_with_options` subsystem calls into a `ProjectScanner` coordinator
- [ ] Make each subsystem call independent and composable

---

## 🟡 Remaining Code Quality

### Error handling (partial)
- [ ] `cmd/banner.rs`: audit `.ok()` calls, surface filesystem permission errors

### Dead code
- [ ] Remove `#![allow(dead_code)]` and fix resulting warnings
- [ ] Audit `cmd/` modules for unused helper functions

### Config integration
- [x] Consume config values (`icons`, `colors`, `compact`, `max_display_items`) in banner rendering
- [x] Centralize ad-hoc env var overrides (`CFM_NO_BUILD_CHECK`, etc.) into config loading

### Features
- [x] `--sort name|size|modified|type` flag (already implemented)
- [x] `--hidden` flag to always show dotfiles

### Completion drift
- [ ] `completion.rs`: refactor to derive completions from the actual `Cli` definition

---

## 🔵 Features

### Sorting & Filtering
- [x] `--sort name|size|modified|type` flag
- [x] `--filter` flag (type, size range, name pattern)
- [ ] `--max N` flag (limit items shown)
- [x] `--hidden` flag to always show dotfiles
- [ ] `--group` flag to group by type (dirs, files, symlinks)

### Table Views
- [ ] `--tree` recursive view (like `lsd --tree`)
- [ ] `--grid` compact multi-column layout (like `ls` default)
- [ ] `--long` full mode (like `ls -l` with all columns)
- [ ] `--short` minimal mode (just name + icon)

### More Git Info
- [ ] Show last commit date per file
- [ ] Show which branch a file was last changed on
- [ ] Show conflict markers during merge (`!` icon)
- [ ] Show stash count in header (`⚑2`)
- [ ] Show worktree state (rebase, cherry-pick, etc.)

### File Metadata
- [ ] Show number of hard links (like `ls -l`)
- [ ] Show file size as a mini bar (like `dust` or `duf`)
- [ ] Show file age relative to git repo creation
- [ ] Show MIME type or category (Code, Config, Document, Media, etc.)

### Color Themes
- [ ] Support `LS_COLORS` environment variable
- [ ] Support `NO_COLOR` environment variable
- [ ] Configurable color scheme via config file
- [ ] Dark/light theme support
- [ ] Color the whole row like exa/lsd (permissions colored by type, size bold if >1MB, date bright if recent)

---

## 🟣 Safety

### Filesystem operation hardening
- [ ] Apply protection checks to `trash`, `mv`, `cp`, and `paste` (currently only `rm` has them)
- [ ] Handle broken symlinks gracefully in listing

### Install script robustness
- [ ] Deduplicate `autoload -U add-zsh-hook` in install.sh
- [ ] Test install.sh idempotency (run twice = no duplicates)
- [ ] Add error handling for partial writes to shell config files

---

## 🟤 Testing

### Unit tests
- [ ] Add tests for `cmd/` implementations (mv, cp, rm, trash, yank, paste, pin, jump)
- [ ] Add tests for `fs/mod.rs` (DirSummary scanning, project detection)

### Integration tests
- [ ] Test file operations (copy, move, delete, trash) end-to-end
- [ ] Test state persistence across invocations
- [ ] Test daemon IPC protocol
- [ ] Test banner rendering (rich, JSON, raw modes)

### Edge cases
- [ ] Test with 1000+ items in directory
- [ ] Test with CJK/emoji filenames
- [ ] Test with symlinks + broken symlinks
- [ ] Test concurrent fm invocations (state file locking)

### Performance
- [ ] Performance benchmark: <50ms for 10k files
- [ ] Parallel stat calls for large directories

---

## ⚪ Polish

### Documentation
- [ ] Add CONTRIBUTING.md with architecture overview
- [ ] Document all 30 commands with usage examples
- [ ] Add man pages for `fm` and `cfmd`
- [ ] Document Config struct fields and env var overrides
- [ ] Remove development artifacts from repo (`note.md`, `session-ses_18cd.md`)

### CI/CD
- [ ] Add cross-compilation matrix (macOS, Linux ARM)
- [ ] Ship `cfmd` binary in GitHub releases
- [ ] Add `--locked` flag to `cargo build` in CI
- [ ] Add MSRV to Cargo.toml

### Code hygiene
- [ ] Audit public `pub` visibility — restrict to only what needs to be public
- [ ] Remove unused `clap_complete` if completions are refactored
