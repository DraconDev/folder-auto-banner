# Release Notes — f 0.6.34

**Date**: 2026-06-15
**Type**: Bug fixes + error message improvements

## Summary

Empirical audit of 68 lazy flag examples revealed 5 real bugs (all score 5/5 for messiness) and 3 confusing error messages (score 4/5). This release fixes all of them.

## Bug Fixes

### Flag duplication bug (5 examples)

The flags `e` (--edit), `U` (--no-sort), `x` (--run), and `f` (--filter) were defined in the `Banner` subcommand but NOT in the top-level `Cli` struct. This meant:

- `f e` (lazy) worked → `f -e` (explicit) **failed** with "unexpected argument '-e' found"
- `f U` (lazy) worked → `f -U` (explicit) **failed**
- `f x` (lazy) worked → `f -x` (explicit) **failed**
- `f f txt` (lazy) worked → `f -f txt` (explicit) **failed** with "value required for --filter"

**Fix**: Added the missing short flags to the top-level `Cli` struct in `src/cli/mod.rs`:
- `#[arg(short = 'e', long = "edit")]`
- `#[arg(short = 'U', long = "no-sort")]`
- `#[arg(short = 'x', long = "run")]`

The `f` flag was already at the top level, but the routing in `main.rs` was intercepting explicit `-f` invocations and trying to rewrite them. Now when any explicit flag is present, the routing bypasses and lets clap handle parsing directly.

**Verification** (all now byte-identical to lazy form):
```
$ f -e   → banner (edit mode)
$ f -U   → banner (no sort)
$ f -x   → banner (run mode)
$ f -f txt → banner (filter=txt)
```

### Routing bypass for explicit flags

When the user passes any explicit flag (starting with `-`), the lazy flag chain system now bypasses and lets clap handle the parsing directly. This prevents the routing from incorrectly rewriting explicit-flag invocations.

**Before**: `f -f txt` → routing sees `txt` as first non-flag, tries to expand as chain `t-x-t` → duplicate flag error
**After**: `f -f txt` → routing detects explicit flag → lets clap parse `-f txt` directly → works

### Improved error messages (3 examples)

**Before** (`f z`, `f tz`, `f xyz`):
```
Error: No such file or directory: z
```

**After**:
```
error: 'z' is not a valid lazy flag. Valid flags: a, c, D, e, f, G, L, m, o, r, R, S, t, U, v, x, X. Use './z' to treat it as a path.

error: 'tz' is not a valid lazy flag chain. Valid flags: a, c, D, e, f, G, L, m, o, r, R, S, t, U, v, x, X. Use './tz' to treat it as a path.
```

The new error message:
1. Names the invalid char(s)
2. Lists all valid lazy flags
3. Suggests `./` to treat as a path

## Scoring Results

### Before fixes
- Mean score: 1.59
- 8/68 examples (11.8%) scored 4-5
- 5 examples scored 5 (all flag duplication bugs)
- 3 examples scored 4 (all error messages)

### After fixes (expected)
- Mean score: ~1.15
- 0/68 examples score 4-5
- All 5 score-5 examples now score 1 (byte-identical to explicit)
- All 3 score-4 examples now score 1 (clear error message)

## Validation

- `cargo fmt --all -- --check` — pass
- `cargo check --all-targets` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo test --all-features` — 130 pass, 9 pre-existing failures (unchanged)
- `cargo doc --no-deps` — pass
- `cargo build --release --locked` — pass
- `cargo publish --dry-run --locked` — pass

The 9 pre-existing test failures are for non-existent subcommands (`pins`, `clipboard`, `sessions`, `diff`, `completion`, `cp`, `trash`, `open`, `peek`) that were never implemented. These are not regressions from this release.

## Files Changed

- `src/cli/mod.rs` — added `e`, `U`, `x` short flags to top-level `Cli`
- `src/main.rs` — bypass routing when explicit flags present; improved error messages for invalid lazy chars
- `Cargo.toml` — version bump to 0.6.34
- `f.1` — version bump to 0.6.34
- `CHANGELOG.md` — added 0.6.34 entry
- `LAZY_FLAGS_MESSINESS.md` — full audit with 68 examples and scoring
- `RELEASE_NOTES_0.6.34.md` — this file
