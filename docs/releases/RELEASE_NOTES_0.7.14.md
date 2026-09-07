# Release notes — v0.7.14 (2026-09-07)

Patch release: 6 commits since v0.7.13. Theme: **small numbers for the
files you actually open** — plus the `f src` bare-path routing fix.

## What's new

**Bottom-up navigation numbers (`[1]` on the bottom row).**
With `sort = "date"` the most recent files sit at the bottom but used to
carry the biggest numbers (`f 156` for the file you just touched).
Numbers now count up from the bottom row, so the likeliest targets get
single-digit numbers. New config `number_from_bottom = true` (default)
restores classic top-down with `false`; `--number-order top|bottom`
overrides per-invocation. `f N` uses the exact inverse mapping, so the
typed number always matches the banner in both modes.

**Dirs-first default (files own the small numbers).**
`group_dirs` default flips `"last"` → `"first"`: directories on top,
files on the bottom rows owning `[1]`, `[2]`, … — `f N` opens files
while `cd`/`z` keep handling directories. Override with
`group_dirs = "last"`/`"none"` or `--group-dirs`. Existing config files
that set `group_dirs` explicitly are unaffected.

**Routing fix — `f src` / `f docs` / `f ./src` show that folder's banner.**
Bare directory names that exist on disk are now recognized as paths
(`is_existing_path` + `is_path_arg`) instead of being silently dropped;
`f -b src` and flag mixes like `f src -t` work too.

**Test isolation fixes (no behavior change).** Two parallel-test races
flaking the suite are serialized: `COLOR_TEST_LOCK` for color-flag
tests, `CACHE_TEST_LOCK` for cache-dir tests. Lib suite 8/8 green over
repeat runs (110 passed).

## Upgrade notes

- New defaults apply to fresh installs. If your config file sets
  `group_dirs` or you prefer the old look: `number_from_bottom = false`
  and/or `group_dirs = "last"` restore it.
- If the zsh auto-banner hook in `~/.zshrc` is still the old commented-out
  "intentionally disabled" block, re-run `./install.sh` (idempotent) after
  upgrading to enable `chpwd`-triggered banners.

## Validation

`cargo check --all-targets` 0 errors · `cargo test --lib` 110 passed ·
`cargo test --bin f` 56 passed · `cargo test --test integration_test`
19 passed · `cargo fmt --check` / `cargo clippy -D warnings` /
`bash -n install.sh` clean · manual banner + `f N` verified in both
numbering modes and both grouping modes.
