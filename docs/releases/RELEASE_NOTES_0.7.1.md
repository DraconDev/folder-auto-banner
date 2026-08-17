# Release Notes — f 0.7.1

**Date**: 2026-06-15

## "Nothing happens" rule, made literal

In 0.7.0, unknown bare words (e.g. `f t`, `f foo`) were silently
passed to the banner subcommand, which produced the default banner
for cwd. The user clarified: **"nothing happens" means literally
nothing** — exit 0, no output.

## New behavior

| Invocation | 0.7.0 | 0.7.1 |
|------------|-------|-------|
| `f` (no args) | Default banner for cwd | Default banner for cwd |
| `f <unknown-word>` | Default banner for cwd | **Exit 0, no output** |
| `f <number>` | Navigate to item N | Navigate to item N |
| `f <path>` | Banner for path | Banner for path |
| `f <alias>` | Expand and run | Expand and run |
| `f <flag>` | Clap direct | Clap direct |

### Examples

```text
$ f t
(no output, exit 0)

$ f foo
(no output, exit 0)

$ f Downloads
(no output, exit 0)  # Downloads without ./ prefix is not a path

$ f 4
~/Dev/folder-auto-banner/src  # navigates to item 4

$ f tree
benches/  # expands to -R -D

$ f
~/Dev/folder-auto-banner │ [main] │ ✓ clean │ ...  # default banner
```

## Implementation

New helper `should_exit_silently(args) -> bool` in `src/main.rs` is
the single source of truth for the "nothing happens" decision. It
returns true when:
- args is non-empty, AND
- no arg is a flag (starts with `-`), AND
- no arg is an explicit path (starts with `.`, `/`, or `~`), AND
- no arg is a known alias, AND
- no arg parses as a number.

The check runs in `main()` after the known-subcommand check and
before the explicit-flag check, so `f t` exits 0 silently while
`f -e` and `f -V` continue to work normally.

## Test metrics

| Metric | 0.7.0 | 0.7.1 | Change |
|--------|-------|-------|--------|
| Unit tests | 36 | 41 | +5 (should_exit_silently tests) |
| Integration tests (active) | 70 | 72 | +2 (f_t_does_nothing, f_no_args_still_shows_banner) |
| **Total** | **210** | **219** | **+9** |
| **Pass rate** | **100%** | **100%** | — |

## Files changed

- `src/main.rs` — new helper `should_exit_silently`, new `main()`
  routing order, 5 new unit tests.
- `tests/alias_test.rs` — 2 new tests, 4 updated tests, 1 renamed test.
- `Cargo.toml` — version bump 0.7.0 → 0.7.1.
- `f.1` — version bump 0.7.0 → 0.7.1.

## Installation

```bash
cargo install folder-auto-banner --version 0.7.1 --locked --force
```

Or update an existing install:

```bash
cargo install folder-auto-banner --force
```
