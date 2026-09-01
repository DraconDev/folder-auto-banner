# Release Notes — v0.7.13

**Date**: 2026-09-01

## Summary

Patch release covering 21 commits since 0.7.12: a full-project audit pass (10 FIX + 4 DECIDE) and its three follow-ups — daemon hung-fallback, `output_rich` deduplication, and a new `Security / Trust model` section — plus earlier daemon/cache and banner bounds hardenings.

## Changes

### Audit pass — `a8ad227` audit pass: fix banner leaks, daemon latency, install idempotency
- `BannerOptions` `sort`/`group_dirs` changed from `Option<&'a str>` (with `Box::leak` on every `run_banner`) to `Option<String>` — eliminates per-invocation leak (src/cmd/banner.rs, src/cli/mod.rs).
- Mini-tree `&dir.name[..tree_width-7]` now guarded (`tree_width>=7` + `char_indices` UTF-8 boundary) — no panic on narrow terminals.
- `is_recent` returns false on future mtime (`age_secs<=0`) instead of `max(0)` masking.
- Daemon `handle_client` removed 5 s post-response drain loops; length-prefixed `read_exact` + dropping stream is sufficient — removes per-request latency tax.
- `is_daemon_running` ping timeout 10 s → 2 s.
- `daemon_mgmt` `Stop`/`Restart` now poll `is_daemon_running` 50×100 ms (5 s) after `send_shutdown` before reporting success.
- `install.sh` idempotent `grep -qF` guards for `chpwd`/`PROMPT_COMMAND` hooks + narrowed cleanup seds.
- `get_data_dir` re-chmods `0o700` on every call with `warn!` on failure.
- Removed dead `let _ = SystemTime::now().duration_since(UNIX_EPOCH);` in `banner_data_cache.rs`.

### DECIDE follow-ups
- **DECIDE 2 — `4720d3d` daemon Banner fallback within 3 s on hung daemon**: `get_banner_cached` `set_read_timeout` 30 s → 3 s, error → `None` so `run_banner`/`navigate_by_number` fall through to direct `DirSummary::scan_with_options` within 3 s (connect-failed path already 40×50 ms poll). Large cold scans >3 s correctly fallback and warm on-disk cache.
- **DECIDE 3 — `5761ec0` refactor: extract build_details_row**: `output_rich`'s two branches (~120 lines each) share one `build_details_row(path, summary, git_info, opts, config)` helper. The 3 differing details (clean indicator in row1, port list, docker fallback `containers>0` vs generic `🐳 docker`) plus ordering/build-error-count/test-results are branched via `is_repo`. Single `push_code_metrics` closure, both branches `let details = build_details_row(...)`. Net -137 lines.
- **DECIDE 4 — `436cb34` docs: Security / Trust model**: `README.md` now documents `fabd` as single-user daemon, Unix socket `~/.local/share/fab/fabd.sock` (ProjectDirs, data dir `0700`), socket `0600` at `src/daemon.rs:161`, unauthenticated length-prefixed JSON (`16 MiB`), and guidance for multi-tenant hosts (`f daemon stop` / direct scan fallback).

### Other commits since 0.7.12
- Bounded scans and cache/watcher hardenings across `src/fs`, `src/git`, `src/cmd/banner`, `src/daemon`, `src/utils`, README, and `.gitignore` (`719cd11`, `66324f9`, `619cf57`, `20359f9`, `92dd0ba`, `27304cb`, `2206278`, `bade042`, `6f57c52`, `68e4df8`, `0f568b7`, `3f462a9`, `45a64f1`, `a2bbaf4`, `1f43e78`, `013ec17`, plus `092eef3` `.pi-glla` ignore).

## Validation

- `cargo check --all-targets` — 0 errors
- `cargo test --lib` — 105 passed
- `cargo test --test integration_test` — 19 passed
- `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `bash -n install.sh` — clean
- Manual `f` banners for git and plain directories visually unchanged (post-refactor)
- `dracon-warden` hardened

## Installation

```bash
cargo install folder-auto-banner --version 0.7.13 --locked --force
```

Or from a checkout:

```bash
./install.sh
```
