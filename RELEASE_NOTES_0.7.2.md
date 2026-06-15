# Release Notes — f 0.7.2

**Date**: 2026-06-15

## Paths are dropped too

In 0.7.1, paths like `f ./src`, `f /tmp`, `f ~/Downloads` were
treated as useful input and routed to the banner subcommand. The
user clarified: **"we only take numbers, aliases, and flags, not
folders and files by name."** Paths are now dropped, the same as
unknown bare words.

## New behavior

| Invocation | 0.7.1 | 0.7.2 |
|------------|-------|-------|
| `f` (no args) | Default banner for cwd | Default banner for cwd |
| `f <number>` | Navigate to item N | Navigate to item N |
| `f <alias>` | Expand and run | Expand and run |
| `f <flag>` | Clap direct | Clap direct |
| `f ./src`, `f /tmp`, `f ~/Downloads` | Banner for path | **Exit 0, no output** |
| `f <unknown-word>` | Exit 0, no output | Exit 0, no output |
| `f banner <path>` | Banner for path (via subcommand) | Banner for path (via subcommand) |

### Examples

```text
$ f ./src
(no output, exit 0)

$ f /tmp
(no output, exit 0)

$ f tree ./src
benches/  # alias expands, path is dropped

$ f banner ./src
~/Dev/folder-auto-banner/src │ [main] │ ...  # subcommand bypasses routing
```

## Implementation

- Removed `is_explicit_path` from `src/main.rs` (no longer needed).
- `args_contain_something_useful` now checks only flags, aliases,
  and numbers — paths no longer count as "useful".
- `expand_aliases_in_args` no longer passes paths through; paths
  are dropped alongside unknown bare words.
- The `should_exit_silently` check now catches paths the same way
  it catches unknown words.

## Workaround for path-specific banners

Two ways to get a banner for a specific path:

1. `cd <path> && f` — use the cwd to specify the path
2. `f banner <path>` — bypass the alias routing with the subcommand

The user can pick whichever they prefer.

## Test metrics

| Metric | 0.7.1 | 0.7.2 | Change |
|--------|-------|-------|--------|
| Unit tests | 41 | 39 | -2 (is_explicit_path tests deleted, others reorganized) |
| Integration tests (active) | 72 | 73 | +1 (f_subcommand_path_still_works) |
| **Total** | **219** | **217** | **-2** |
| **Pass rate** | **100%** | **100%** | — |

The unit-test count dropped by 2 because the 4 is_explicit_path tests
were deleted and replaced with 2 new tests for the path-dropping
behavior. The net change is small.

## Files changed

- `src/main.rs` — `is_explicit_path` deleted, helper functions updated,
  expand logic updated, tests reorganized.
- `tests/alias_test.rs` — 3 tests renamed and updated, 2 new tests.
- `LAZY_FLAGS_REMOVAL.md` — routing table updated to reflect new rule.
- `Cargo.toml` — version bump 0.7.1 → 0.7.2.
- `f.1` — version bump 0.7.1 → 0.7.2.

## Installation

```bash
cargo install folder-auto-banner --version 0.7.2 --locked --force
```

Or update an existing install:

```bash
cargo install folder-auto-banner --force
```
