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
- [x] `README.md` — mentions both zsh and bash
- [x] `release.yml` — fixed step ordering, switched to `softprops/action-gh-release@v2`

---

## 🔴 Critical Fixes

### Dry-run flag silently discarded
- [x] In `cli/mod.rs:462-492`, the `--dry_run` field is bound as `_` and never passed to command functions
- [x] Wire `dry_run` through to `run_mv`, `run_cp`, `run_rm`, `run_trash`, `run_open`, `run_do_cmd`
- [x] Verify dry-run actually prevents file mutations in each command

### Daemon mutex poisoning
- [x] Replace all 18 `.unwrap()` on `Mutex::lock()` in `daemon.rs` with `unwrap_or_else(|e| e.into_inner())` or proper error propagation
- [ ] Audit `Mutex` usage for potential deadlocks (nested locks, lock ordering)
- [ ] Add logging when mutex poisoning occurs so failures are visible

### Shell injection vectors
- [x] `do_cmd.rs`: sanitize `{}` replacement — reject or escape filenames containing shell metacharacters
- [ ] `banner.rs:count_sqlite_tables`: quote or escape paths passed to `sqlite3` command
- [ ] Review all `Command::new()` call sites for user-controlled argument injection

### Data loss risk in mv/trash
- [x] `mv.rs` and `trash.rs`: verify `copy_dir_recursive` succeeds fully before calling `delete_recursive`
- [x] Add rollback or partial-failure handling for cross-device moves

---

## 🟠 Architecture

### Extract shared library crate (`cfm-lib`)
- [ ] Create `cfm-lib/` with `Cargo.toml` as a library crate
- [ ] Move shared modules to lib: `build_status`, `cache`, `code_metrics`, `daemon_types`, `docker`, `fs`, `git`, `icon`, `port_usage`, `state`, `todo_scanner`
- [ ] Update `fm` binary to depend on `cfm-lib`
- [ ] Update `cfmd` binary to depend on `cfm-lib`
- [ ] Remove redundant `mod` declarations from `daemon.rs`
- [ ] Remove `#![allow(dead_code)]` from both `main.rs` and `daemon.rs`
- [ ] Fix any dead code warnings that surface

### Decouple banner rendering from I/O
- [ ] Extract data extraction (image headers, ZIP inspection, SQLite queries, MP4 parsing) from `banner.rs` into a `banner_data.rs` or per-format modules
- [ ] Make `output_rich()` consume pre-extracted data, not read files directly
- [ ] Split `banner.rs` (1,167 lines) into smaller focused modules

### Decouple daemon from binary entry point
- [ ] Move daemon implementation out of `daemon.rs` into `daemon/` module (or `cfm-lib`)
- [ ] Keep `daemon.rs` as a thin entry point that calls into the library

### Fix `DirSummary::scan_with_options` god function
- [ ] This 250-line function orchestrates 5 subsystems directly
- [ ] Extract subsystem calls into a `ProjectScanner` or similar coordinator
- [ ] Make each subsystem call independent and composable

### Consolidate `Session` types
- [ ] `state/mod.rs`, `cmd/save_session.rs`, `cmd/sessions.rs` each define a different `Session` struct
- [ ] Create one canonical `Session` struct with all fields, use it everywhere
- [ ] Add proper serde support for serialization/deserialization

### Fix `cli/mod.rs` separation of concerns
- [ ] Extract `run_daemon()` out of `cli/mod.rs` into a daemon command module
- [ ] Reduce `run_banner` parameter count (11 params → struct)
- [ ] Reduce `run_cp` parameter count (6 params → struct)

---

## 🟡 Code Quality

### Extract duplicated functions (~500 lines)
- [x] `copy_dir_recursive`: exists in `cp.rs`, `mv.rs`, `trash.rs` → move to `fs/mod.rs`
- [x] `delete_recursive`: exists in `mv.rs`, `trash.rs` → move to `fs/mod.rs`
- [x] `sanitize_filename`: exists in `save_session.rs`, `load_session.rs`, `delete_session.rs` → move to `state/mod.rs` or new `utils.rs`
- [x] `generate_unique_name`: exists in `cp.rs`, `mv.rs` → move to `fs/mod.rs`
- [x] `format_size`: exists in `fs/mod.rs`, `diff.rs` → use the canonical version in `fs/mod.rs`, remove duplicate from `diff.rs`
- [x] `BINARY_EXTS` constant: exists in `todo_scanner/mod.rs` and `code_metrics/mod.rs` → single shared constant
- [x] `SKIP_DIRS` constant: exists in `todo_scanner/mod.rs` and `code_metrics/mod.rs` → single shared constant
- [x] `run_with_timeout`: exists in `build_status/mod.rs`, `port_usage/mod.rs`, `docker/mod.rs` → single shared utility
- [x] `print_summary`: exists in `mv.rs`, `cp.rs` → single shared utility

### Fix error handling
- [x] Audit all 57 `.ok()` calls — categorize as intentional (cache writes) vs accidental (masking real errors)
- [x] Replace accidental `.ok()` discards with `?` propagation or `eprintln!` logging
- [x] `daemon_client.rs:98,110`: replace `let _ = serde_json::to_writer(...)` and `let _ = send_and_recv(...)` with error logging
- [x] `fs/mod.rs`: audit 15 `.ok()` calls, add logging for non-cache failures
- [ ] `cmd/banner.rs`: audit 10 `.ok()` calls, surface filesystem permission errors
- [x] Remove unused `thiserror` dependency from `Cargo.toml` (or start using it for custom error types)

### Remove dead code
- [ ] Remove `#![allow(dead_code)]` from `main.rs` and `daemon.rs`
- [ ] Fix all resulting warnings — delete unused functions/structs or add `#[allow(dead_code)]` to specific items
- [ ] Audit `cmd/` modules for unused helper functions
- [x] Remove `cmd/uninstall_hook.rs` stub (or implement it)
- [x] Remove `cmd/root.rs` stub (or implement the non-print-cd path)

### Wire up Config
- [x] `cmd/config.rs` is entirely a stub — implement actual config reading/writing
- [x] `--edit` should open `$EDITOR` with the config TOML file
- [x] `--get <key>` should read from the config file
- [x] `--set <key> <value>` should write to the config file
- [ ] Consume config values (`icons`, `colors`, `compact`, `max_display_items`) in banner rendering
- [ ] Centralize ad-hoc env var overrides (`CFM_NO_BUILD_CHECK`, `CFM_NO_TODOS`, etc.) into config loading
- [x] Remove `thiserror` from Cargo.toml if unused, or adopt it for `ConfigError`

### Fix completion drift
- [ ] `completion.rs` manually rebuilds the clap command tree instead of reusing the `Cli` struct
- [ ] Refactor to derive completions from the actual `Cli` definition

---

## 🔵 Features

### Sorting & Filtering
- [ ] `--sort name|size|modified|type` flag
- [ ] `--filter` flag (type, size range, name pattern)
- [ ] `--max N` flag (limit items shown)
- [ ] `--hidden` flag to always show dotfiles
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

### Implement stub commands
- [x] `cmd/root.rs`: implement the non-print-cd path
- [x] `cmd/uninstall_hook.rs`: implement hook removal

---

## 🟣 Safety

### Filesystem operation hardening
- [x] Add symlink-following guards in recursive operations (prevent symlink loops)
- [x] Extend `is_protected_path()` in `rm.rs` to cover `~/.ssh`, `~/.gnupg`, `~/.config`
- [ ] Apply protection checks to `trash`, `mv`, `cp`, and `paste` (currently only `rm` has them)
- [x] Verify copy success before deleting source in cross-device moves
- [ ] Handle broken symlinks gracefully in listing

### Install script robustness
- [ ] Deduplicate `autoload -U add-zsh-hook` in install.sh
- [ ] Test install.sh idempotency (run twice = no duplicates)
- [ ] Add error handling for partial writes to shell config files

---

## 🟤 Testing

### Unit tests
- [ ] Add tests for all `cmd/` implementations (banner, mv, cp, rm, trash, yank, paste, pin, jump, etc.)
- [ ] Add tests for `state/mod.rs` (save/load clipboard, pins, sessions, config)
- [ ] Add tests for `cache/mod.rs` (TTL expiration, set/get, concurrent access)
- [ ] Add tests for `fs/mod.rs` (DirSummary scanning, format_size, project detection)
- [ ] Add tests for `build_status/mod.rs`, `todo_scanner/mod.rs`, `code_metrics/mod.rs`, `port_usage/mod.rs`, `docker/mod.rs`

### Integration tests
- [ ] Test file operations (copy, move, delete, trash) end-to-end
- [ ] Test state persistence across invocations
- [ ] Test daemon IPC protocol
- [ ] Test banner rendering (rich, JSON, raw modes)
- [ ] Test error handling paths (permission denied, missing files, etc.)

### Edge cases
- [ ] Test with 1000+ items in directory
- [ ] Test with very long filenames
- [ ] Test with CJK/emoji filenames
- [ ] Test with symlinks + broken symlinks
- [ ] Test git status in repos with many changes
- [ ] Test in non-tty contexts (piped output)
- [ ] Test empty directories
- [ ] Test cross-device moves
- [ ] Test concurrent fm invocations (state file locking)

### Performance
- [ ] Performance benchmark: <50ms for 10k files
- [ ] Cache directory scan results (TTL-based) — partially done via daemon
- [ ] Parallel stat calls for large directories
- [ ] `--depth N` for shallow recursive view

---

## ⚪ Polish

### Documentation
- [ ] Add architecture overview to README.md
- [ ] Document all 28 commands with usage examples
- [ ] Add CONTRIBUTING.md
- [ ] Add man pages for `fm` and `cfmd`
- [ ] Update README test count (currently claims "10/10" but there are 14)
- [ ] Remove development artifacts from repo (`note.md`, `session-ses_18cd.md`)
- [ ] Document the `Config` struct fields and their effects

### CI/CD
- [ ] Add cross-compilation matrix (macOS, Linux ARM)
- [ ] Ship `cfmd` binary in GitHub releases (currently only `fm` is shipped)
- [ ] Add `--locked` flag to `cargo build` in CI for reproducibility
- [ ] Add MSRV (minimum supported Rust version) to Cargo.toml

### Code hygiene
- [ ] Run `cargo clippy` and fix all warnings
- [ ] Run `cargo fmt` and fix all formatting
- [ ] Audit public `pub` visibility — restrict to only what needs to be public
- [ ] Remove unused `clap_complete` if completions are refactored
- [ ] Resolve `thiserror` version mismatch (Cargo.toml says "2", lockfile has 1.0.69)
