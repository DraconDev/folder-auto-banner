# Release Notes — f 0.7.0

**Date**: 2026-06-15

## ⚠️ BREAKING CHANGES

This release **removes the lazy flag system** introduced in 0.6.29.
The replacement is a cleaner **built-in alias system** that does not
require memorizing single-character flag chains.

### Why remove lazy flags?

The lazy flag system grew into a complex mini-DSL:

- 17 single-char flags + 5 case-insensitive aliases
- Chain parsing (`f trc` → `-t -r -c`)
- Value-binding with `:` (`f mLf: 10` → `-f 10`, added in 0.6.37)
- Explicit-flag bypass (added in 0.6.34)
- No-fallback rule for bare words

Each new feature added another special case. The `:` binding was the
breaking point — a kludge that overloaded the chain syntax with a
binding marker. The new system is just **words and flags**.

## New built-in aliases (18)

| Alias     | Expands to        | What it does                          |
|-----------|-------------------|---------------------------------------|
| `tree`    | `-R -D`           | Recursive, only dirs (like `tree`)    |
| `flat`    | `-o`              | One file per line                     |
| `compact` | `-c`              | Compact output                        |
| `verbose` | `-v`              | Verbose output                        |
| `hidden`  | `-a`              | Show hidden files                     |
| `dirs`    | `-D`              | Only directories                      |
| `new`     | `-t`              | Sort by time, newest first            |
| `old`     | `-t -r`           | Sort by time, oldest first            |
| `big`     | `-S`              | Sort by size, largest first           |
| `small`   | `-S -r`           | Sort by size, smallest first          |
| `ext`     | `-X`              | Sort by extension                     |
| `git`     | `-G`              | Sort by git status                    |
| `nosort`  | `-U`              | No sort                               |
| `top`     | `-S -r -m 20`     | Top 20 largest files                  |
| `newest`  | `-t -r -m 20`     | 20 newest files                       |
| `recurse` | `-R`              | Recurse into subdirectories           |
| `edit`    | `-e`              | Force open in editor                  |
| `run`     | `-x`              | Force run file                        |

Aliases compose:

```text
f hidden verbose    →  -a -v
f new recurse       →  -t -R
f tree hidden       →  -R -D -a
```

Aliases compose with explicit flags and paths:

```text
f tree -L 2         →  -R -D -L 2
f top ./src         →  -S -r -m 20  for ./src
```

## New routing logic

| Invocation                    | Behavior                                |
|-------------------------------|------------------------------------------|
| `f`                           | Default banner for cwd                  |
| `f -<flag>` or `f --<flag>`   | Explicit flags (clap)                   |
| `f <number>`                  | Navigate to item N                      |
| `f <alias>`                   | Expand built-in alias                   |
| `f <word>` (not number, not alias) | Default banner for cwd (no error)  |
| `f ./path`, `f /path`, `f ~/path` | Explicit path                        |

## Migration from 0.6.x

| 0.6.x form   | 0.7.0 equivalent                |
|--------------|----------------------------------|
| `f t`        | `f new` (or `f -t`)             |
| `f trc`      | `f new -r -c`                   |
| `f S`        | `f big`                         |
| `f mL 10 2`  | `f -m 10 -L 2`                  |
| `f mLf: 10`  | `f -f 10`                       |
| `f s`        | `f big`                         |
| `f l5`       | `f -L 5`                        |
| `f Downloads`| `f ./Downloads` (bare = alias)  |

For common combinations, add a shell alias:

```bash
# ~/.zshrc or ~/.bashrc
alias ftrc='f -t -r -c'
alias ftree='f -R -D'
alias fbig='f -S -r -m 50'
```

## What was removed

- `LAZY_FLAGS`, `LOWERCASE_ALIASES`, `VALUE_TAKING_FLAGS` constants
- `resolve_lazy_flag_char`, `expand_lazy_flags`,
  `expand_lazy_flags_with_binding` functions
- `ExpandedChain` struct
- The 0.6.37 `:` value-binding syntax
- The "no fallback" error for bare words
- ~60 lazy-flag-related tests
- The `tests/lazy_flags_test.rs` file (replaced with `tests/alias_test.rs`)

## What was preserved

- 0.6.34 flag wiring (`e`/`U`/`x`/`f` short flags in top-level `Cli`)
- Explicit flag parsing (`-t`, `--filter txt`)
- Number navigation (`f 1` → item 1)
- Path handling (`f ./x`, `f /x`, `f ~/x`)
- All 0.6.34 regression fixes

## Test metrics

| Metric | 0.6.37 | 0.7.0 | Change |
|--------|--------|-------|--------|
| Unit tests | 64 | 36 | -28 (lazy flag tests removed) |
| Integration tests (active) | 29 | 62 | +33 |
| Alias tests | 0 | 41 | +41 (new) |
| **Total** | **303** | **210** | **-93** |
| **Pass rate** | **100%** | **100%** | — |

## Installation

```bash
cargo install folder-auto-banner --version 0.7.0 --locked --force
```

Or update an existing install:

```bash
cargo install folder-auto-banner --force
```

The daemon (`fabd`) does not need to be restarted for this change
(the alias logic is entirely client-side).

## Files changed

- `src/main.rs` — full rewrite: removed lazy flag code, added
  `BUILTIN_ALIASES`, `lookup_alias`, `expand_aliases_in_args`,
  new routing logic, alias tests.
- `tests/lazy_flags_test.rs` — **deleted** (lazy flag system removed).
- `tests/alias_test.rs` — **new**, 41 tests for the alias system.
- `Cargo.toml` — version bump to 0.7.0.
- `f.1` — version bump to 0.7.0.
- `CHANGELOG.md` — entry for 0.7.0.
- `LAZY_FLAGS_REMOVAL.md` — new design document.
- `LAZY_FLAGS_AUDIT.md`, `LAZY_FLAGS_MESSINESS.md`,
  `LAZY_FLAGS_VALUE_BINDING.md`, `LAZY_FLAGS_TESTING.md` —
  marked as historical.
