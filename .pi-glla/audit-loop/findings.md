# Audit Loop Findings — folder-auto-banner-fab

Ledger: append-only. Each finding listed once. FIX findings get fixed and checked with a commit; DECIDE findings are raised to the user and recorded.

Audit pass 1 — 2026-08-16 (4 parallel Explore agents: daemon/state, cli/cmd, feature modules, tests/scripts/docs)

## Repo hygiene / tests / docs

- [x] FIX: high: 12.1MB ELF `micro` committed at repo root, never installed or referenced by any build/install path (micro, added 57d0ca1 2026-05-28) — fixed in 620383c
- [x] FIX: med: scripts/test_lazy_flags.sh tests the lazy-flag DSL removed in v0.7.0; passes spuriously (compares only exit codes) (scripts/test_lazy_flags.sh:6-8) — fixed in 620383c
- [x] FIX: med: repo-root fab-shell.bash/.zsh are stale copies vs the compiled-in src/shell_wrapper.rs constants that `f install` ships; zsh copy uses bash 0-indexing on args (breaks `f N` cd); README claims they are generated from the constants (fab-shell.zsh:48-53, README.md:125) — fixed in 620383c, 2ee24f8
- [x] FIX: med: README flag tables document top-level flags that only exist on the banner subcommand — `f --versionsort` exits 2 (README.md:80,251-289 vs src/cli/mod.rs) — fixed in 49bc669, cbf9085 (tables qualified as banner flags; ‡ marks subcommand-only rows)
- [x] FIX: med: shell wrapper defaults EDITOR to `micro` but install.sh/`f install` never install micro — file open dies "command not found: micro" on fresh installs (src/shell_wrapper.rs:53,111) — fixed in ef84ff7
- [x] FIX: med: 14 no-op tests in tests/integration_test.rs with fully commented-out bodies, NOTE comments describe the removed lazy-flag system as current (tests/integration_test.rs:16-137) — fixed in 6187365, c857167, c5323d5
- [x] FIX: low: man page documents `-1, --oneline`; actual flag is `-o/--oneline` (f.1:79) — fixed in 49bc669
- [x] FIX: low: CI runs `cargo test --all-features` without `-- --test-threads=1`, contradicting the documented requirement in tests/alias_test.rs:7-9 (.github/workflows/ci.yml:41) — fixed in 49bc669
- [x] FIX: low: scratch file 1.txt ("asdfasdfsadf") committed at tag v0.6.5 (1.txt, 66c72f5) — fixed in 620383c
- [x] FIX: low: note.md tracked with scratch working notes (note.md:1-5) — fixed in 620383c
- [x] FIX: low: alias_test `b_flag_alone_shows_default_banner` computes output and discards it — banner unasserted (tests/alias_test.rs:169-174) — fixed in 6187365
- [x] FIX: low: ALIASES.md heading says 19 aliases, table lists 18 (ALIASES.md:111) — fixed in 49bc669

## CLI & commands

- [x] FIX: high: 5 files in src/cmd/ never declared in cmd/mod.rs — never compiled; `f open`/`f completion`/`f uninstall`/`f install-hook`/`f config` silently no-op with exit 0 (src/cmd/mod.rs:1-7; completion.rs, config.rs, install_hook.rs, open.rs, uninstall_hook.rs) — fixed in ec37e80
- [x] FIX: med: dead completion script emits phantom subcommands (yank, paste, mv, cp, stats, pins, sessions, diff, ...) and omits real install/daemon (src/cmd/completion.rs:11-62 vs src/cli/mod.rs:132-213) — fixed in ec37e80
- [x] FIX: med: `f env --format json` is a no-op — run_env ignores format and always prints shell aliases (src/cmd/env.rs:7-10, src/cli/mod.rs:253-255) — fixed in dab6a03
- [x] FIX: med: `f env` emits `alias fab_clean='cargo clean && fm banner'` referencing the pre-rename `fm` binary (src/cmd/env.rs:35) — fixed in dab6a03
- [x] FIX: med: `f install` appends rc lines marked `# f shell function (for cd support)` that no code ever removes; dead uninstall_hook.rs strips a different block (src/cmd/install.rs:31-56, src/cmd/uninstall_hook.rs:14-33) — fixed in de4c63e, 16e6717, 8424f99
- [x] FIX: med: FAB_ICONS="nerd" disables icons entirely — banner.rs gates `v == "1"`, icon.rs selects glyph mode on "nerd" (src/icon.rs:3-11 vs src/cmd/banner.rs:474-476) — fixed in dab6a03
- [x] FIX: med: `f daemon clear-cache` deletes banner_cache.json/dir_sizes.json/socket but leaves data_dir/banner_data/*.json — disk fast path still serves cached banners (src/cmd/daemon_mgmt.rs:60-86) — fixed in dab6a03
- [x] FIX: med: every `f daemon` action returns Ok(()) — failed start/stop/restart exits 0 (src/cmd/daemon_mgmt.rs:16-44) — fixed in dab6a03
- [x] FIX: low: top-level Cli exposes --git-ignore/--highlight-recent/--highlight-old that the banner subcommand lacks — `f banner --git-ignore` is a clap parse error (src/cli/mod.rs:104-106,149-152) — fixed in dab6a03
- [x] FIX: low: install.rs writes `source {}/{}` unquoted into rc file — breaks when $HOME contains spaces (src/cmd/install.rs:54) — fixed in dab6a03

## Daemon & state

- [x] FIX: med: client disk-cache fast path bypasses daemon and serves stale banners for nested (depth >= 2) changes up to 300s — max_descendant_mtime skips directories (src/cmd/banner_data_cache.rs:152-215, src/daemon_client.rs:72-78) — fixed in 4b64e8a, ca596b5
- [x] FIX: med: socket-unlink races — client unlinks on busy/timeout (10s non-Pong), daemon unlinks unconditionally at start; a dying daemon can remove a live daemon's socket, producing two daemons writing banner_cache.json/dir_sizes.json concurrently (src/daemon.rs:122, src/daemon_client.rs:91,145,157) — fixed in 1ce7b80, fe1e8bf
- [x] FIX: med: IPC socket and data dir created with default perms, no peer authentication — any local user who can reach the socket can read arbitrary listings/git status and send unauthenticated Shutdown (DoS) (src/daemon.rs:99-142,765,1060) — fixed in 1ce7b80
- [x] FIX: med: background size refresh updates only the in-memory cache, never the per-path disk cache — disk-cache clients see pre-refresh sizes until TTL (src/daemon.rs:1442-1483 vs 1105) — fixed in 1ce7b80
- [x] FIX: med: daemon auto-start retries connect exactly once after a fixed 50ms — cold daemon exceeds it, failure silently swallowed, caller falls back to local scan (src/daemon_client.rs:94-96) — fixed in fe1e8bf
- [x] FIX: low: Request::DirSize/Response::DirSize are dead protocol variants (implemented in daemon, no client) (src/daemon_types.rs:12, src/daemon.rs:1006) — fixed in f17799d, 73bc3d1
- [x] FIX: low: unreachable Warm guard after the match arm already returns (src/daemon.rs:1068-1070) — fixed in f17799d
- [x] FIX: low: redundant `expired` filter makes its log branch dead (src/daemon.rs:797-821) — fixed in 1ce7b80
- [x] FIX: low: failed_watches entries never retried — a path that becomes watchable stays unwatched until restart (src/daemon.rs:560,618-620) — fixed in 1ce7b80
- [x] FIX: low: du invocation lacks `--` separator (dir starting with `-` misparsed); non-UTF8 paths lossy-converted (src/daemon.rs:1608-1611) — fixed in 1ce7b80
- [x] FIX: low: cache uses world-writable /tmp/f-cache with predictable DefaultHasher keys — pre-create/symlink redirection attack on daemon writes (src/cache/mod.rs:18-32) — fixed in 1ce7b80, 0bd8287, b5e74e9 (moved to per-user ~/.cache/fab 0700 w/ atomic temp+rename; symlink refusal)
- [x] FIX: low: state persistence is non-atomic fs::write with no locking — torn JSON on crash, lost updates on concurrent invocations (src/state/mod.rs:68,115,189,347) — fixed in 7cec5f6
- [x] FIX: low: clipboard/pins/sessions public API entirely dead, all #[allow(dead_code)] (src/state/mod.rs:26-165) — fixed in b22afa9, 6d5cf7c, 592bec4
- [x] FIX: low: TestResults::save/format_duration dead; load expiry `now - timestamp > 300` never expires future timestamps (src/test_cache.rs:43-63) — fixed in 81e8ede, 31eca32

## Feature modules

- [x] FIX: high: python build check always fails — py_files collected but never passed to `python3 -m py_compile` (zero args → exit 2) (src/build_status/mod.rs:128-146) — fixed in 7ba569f
- [x] FIX: high: lsof fallback ORs selection options (missing -a) — reports every network file system-wide as project-owned; `+D` comment (cwd claim) false (src/port_usage/mod.rs:132-169) — fixed in 1b2a297
- [x] FIX: med: `git status --porcelain -z` rename/copy entries emit bare orig-path NUL records that the parser treats as XY status records — staged inflated by 1 per rename, garbage map key inserted (src/git/mod.rs:170-214) — fixed in 2dfdf77
- [x] FIX: med: "UU" conflicts counted as staged/Added — FileStatus::Conflict never constructed in production, banner conflict logic (priority 5) dead (src/git/mod.rs:187-213) — fixed in 2dfdf77
- [x] FIX: med: `dir/*` pathspec comment false (plain pathspecs match recursively) — deep paths inflate header counts AND bloat file_statuses (36K untracked entries under target/, ~3.4MB serialized per IPC request) while display depth strips them (src/git/mod.rs:41-57, known issue in tasks.md) — fixed in 2dfdf77
- [x] FIX: med: `subject[..80]` and truncate_output[..500] panic when a multibyte char is split at the byte boundary (src/git/mod.rs:255, src/build_status/mod.rs:174) — fixed in 2dfdf77, 1b2a297
- [x] FIX: med: fs scan breaks at MAX_ITEMS=500 but comment claims totals still count all entries — sizes/counts undercount on big dirs (src/fs/mod.rs:202-219) — fixed in 1b2a297
- [x] FIX: low: dead + broken format_size (strips unit suffix); production uses format_size_compact (src/fs/mod.rs:508-523) — fixed in 16e4077
- [x] FIX: low: todo_scanner::scan_todos and code_metrics::scan_metrics have zero callers; scan logic duplicated inside project_insights — two copies can drift (src/todo_scanner/mod.rs:32, src/code_metrics/mod.rs:29) — fixed in 81e8ede
- [x] FIX: low: git_ahead_behind doc claims origin/<branch> fallback; code returns (0,0) on any @{upstream} failure (src/git/mod.rs:229-241) — fixed in 2dfdf77
- [x] FIX: low: extract_pid parses only the first pid= in multi-process SO_REUSEPORT socket entries — false negatives (src/port_usage/mod.rs:113-120) — fixed in 43971f5, 40da1ae

## DECIDE findings

None this pass — every finding above has one unambiguous durable fix (the removed command set was an intentional v0.3.0 simplification per tasks.md; no direction questions arose).
- [x] FIX: med: build-status feature dead in production — `config.build_status` (default true, state/mod.rs:243,290) never read; `check_build` hardcoded false at both call sites, so the "✓ builds" pod can never render (src/cmd/banner.rs:592, src/daemon.rs:1111) — fixed in 8a84a68
