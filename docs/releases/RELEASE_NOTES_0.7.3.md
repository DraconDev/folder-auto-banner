# Release Notes — f 0.7.3

**Date**: 2026-06-15

## `-b` flag: banner mode, allows paths

In 0.7.2, paths were dropped entirely. The user wanted a way to
get a banner for a specific path without typing the subcommand
name. The new `-b` flag switches to banner mode, which allows paths.

## New behavior

| Invocation | Result |
|------------|--------|
| `f -b` | Default banner for cwd |
| `f -b ./src` | Banner for `./src` |
| `f -b /tmp` | Banner for `/tmp` |
| `f -b ~/Downloads` | Banner for `~/Downloads` |
| `f -b tree` | Tree banner (alias expands) |
| `f -b tree ./src` | Tree banner for `./src` |
| `f -b -t` | Banner with `-t` flag |
| `f -b 5` | Navigate to item 5 |
| `f -b foo` | Drop `foo`, run default banner |

## Alias audit (clean)

All 18 built-in aliases reviewed and confirmed clean:

| Alias | Expands to | Notes |
|-------|-----------|-------|
| `tree` | `-R -D` | Recursive, only dirs |
| `flat` | `-o` | One file per line |
| `compact` | `-c` | Compact output |
| `verbose` | `-v` | Verbose output |
| `hidden` | `-a` | Show hidden files |
| `dirs` | `-D` | Only directories |
| `new` | `-t` | Sort by time, newest first |
| `old` | `-t -r` | Sort by time, oldest first |
| `big` | `-S` | Sort by size, largest first |
| `small` | `-S -r` | Sort by size, smallest first |
| `ext` | `-X` | Sort by extension |
| `git` | `-G` | Sort by git status |
| `nosort` | `-U` | No sort |
| `top` | `-S -r -m 20` | Top 20 largest |
| `newest` | `-t -r -m 20` | 20 newest |
| `recurse` | `-R` | Recurse into subdirectories |
| `edit` | `-e` | Force open in editor |
| `run` | `-x` | Force run file |

The alias table is a simple `&[(&str, &[&str])]` constant. Adding
a new alias is one line. Naming convention: lowercase, single word,
no abbreviations. Aliases that need a value (`top` and `newest`)
have the value baked in (`-m 20`).

## Flag audit (clean)

All 25 top-level Cli flags reviewed and confirmed clean. Short
forms use sensible single letters: `t` (time), `S` (size), `X`
(extension), `G` (git), `r` (reverse), `a` (all/hidden), `o`
(oneline), `f` (filter), `m` (max), `L` (level), `c` (compact),
`v` (verbose), `U` (no-sort), `e` (edit), `x` (run), `R` (recursive),
`D` (dirs). The new `-b` is a routing switch, not a Cli flag.

## Implementation

- New `expand_args_for_banner(args)` function: pass through flags,
  paths, and numbers; expand aliases; drop unknown words.
- New `expand_args_strict(args)` function: pass through flags and
  numbers; expand aliases; drop paths and unknown words.
- New `is_path_like(arg)` helper: returns true if arg starts with
  `.`, `/`, or `~`.
- New routing branch in `main()`: if `-b` is in args, use
  `expand_args_for_banner`; otherwise use `expand_args_strict`.

## Test metrics

| Metric | 0.7.2 | 0.7.3 | Change |
|--------|-------|-------|--------|
| Unit tests | 39 | 49 | +10 (-b tests) |
| Integration tests | 73 | 79 | +6 (-b tests) |
| **Total** | **217** | **235** | **+18** |
| **Pass rate** | **100%** | **100%** | — |

## Files changed

- `src/main.rs` — `expand_args_for_banner`, `expand_args_strict`,
  `is_path_like` functions added; routing updated; tests added.
- `tests/alias_test.rs` — 6 new tests, 1 updated test.
- `Cargo.toml` — version bump 0.7.2 → 0.7.3.
- `f.1` — version bump 0.7.2 → 0.7.3.

## Installation

```bash
cargo install folder-auto-banner --version 0.7.3 --locked --force
```

Or update an existing install:

```bash
cargo install folder-auto-banner --force
```
