# Lazy Flags Removal — folder-auto-banner

**Date**: 2026-06-15
**Status**: Approved, implementation in progress
**Version**: 0.7.0 (breaking change from 0.6.x)

## Why remove lazy flags?

The lazy flag system grew from a simple "save typing" optimization (0.6.29)
into a complex mini-DSL with chain parsing, case-insensitive aliases,
value-binding with `:` (0.6.37), and a no-fallback rule. Each new feature
added another special case:

- `-1`→`-o` rename (0.6.30) because `f 1` navigates
- Case-insensitive aliases (0.6.29) for the 5 chars without conflicts
- `:` value-binding (0.6.37) to disambiguate which flag gets the value
- Explicit-flag bypass (0.6.34) to keep `f -t` working
- No-fallback rule: bare words are always chains, never paths

The 0.6.37 `:` binding was the breaking point. A user asked
"what if we want to give the argument to the last one?" and the answer
was a syntax (`mLf:`) that overloaded the chain with a binding marker.
That's a sign the system had outgrown its design.

The replacement: **built-in word aliases**. If you always use
`f -t -r -c`, type `f tree` instead. No chain parsing, no value-binding,
no case-insensitive ambiguity. Just words and flags.

## New routing logic

| Invocation              | Behavior                                                |
|-------------------------|---------------------------------------------------------|
| `f`                     | Default banner for cwd                                  |
| `f -<flag>`             | Explicit short flag                                     |
| `f --<flag>`            | Explicit long flag                                      |
| `f <number>`            | Navigate to item N (existing)                           |
| `f <alias>`             | Expand built-in alias, then run                         |
| `f <word>` (not number, not alias) | Exit 0, no output (the "nothing happens" rule) |
| `f ./path`              | Explicit path                                           |
| `f /path`               | Explicit path                                           |
| `f ~/path`              | Explicit path                                           |
| `f <alias1> <alias2>`   | Expand both, concatenate flags                          |
| `f <alias> -<flag>`     | Expand alias, then apply explicit flag                  |
| `f <alias> <path>`      | Expand alias, then apply to path                        |

The "nothing happens" rule for unknown bare words replaces the
0.6.x no-fallback error. If you want to open a folder called
`foo`, use `./foo`. To see a banner for cwd, just type `f` (with
no args). To see a banner for a folder, use `./foldername` or
`/path/to/folder`. Unknown bare words like `f t` or `f foo` exit
0 with no output. This is simpler and more forgiving.

## Built-in aliases (19)

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

### Excluded on purpose

- `all` — too ambiguous (all hidden? all flags? all of the above?)
- Value-taking aliases (`tree2`, `top50`) — adds complexity, users can
  use explicit flags for custom values: `f -R -L 2`, `f -S -r -m 50`

### Alias composition

Aliases are just flag lists. They concatenate:

```text
f hidden verbose    →  -a -v
f new recurse       →  -t -R
f tree hidden       →  -R -D -a
f top ./src         →  -S -r -m 20  applied to ./src
f tree -L 2         →  -R -D -L 2    (alias + explicit flag)
```

## What gets removed

- `LAZY_FLAGS` constant (17 chars: a, c, D, e, f, G, L, m, o, r, R, S, t, U, v, x, X)
- `LOWERCASE_ALIASES` constant (s, g, d, l, u → S, G, D, L, U)
- `VALUE_TAKING_FLAGS` constant (m, f, L)
- `resolve_lazy_flag_char` function
- `expand_lazy_flags` function
- `expand_lazy_flags_with_binding` function
- `ExpandedChain` struct
- `has_explicit_flag` bypass logic (replaced by alias-aware routing)
- The 0.6.37 `:` value-binding syntax
- All ~60 lazy-flag-related unit and integration tests

## What stays

- 0.6.34 flag wiring (`e`/`U`/`x`/`f` short flags in top-level `Cli`)
- Explicit flag parsing (`-t`, `--filter txt`)
- Number navigation (`f 1` → item 1)
- Path handling (`f ./x`, `f /x`, `f ~/x`)
- The "no fallback" rule, reinterpreted: bare words are aliases, not paths

## Migration from 0.6.x

If you used lazy flag chains in 0.6.x:

| 0.6.x form        | 0.7.0 equivalent                        |
|-------------------|------------------------------------------|
| `f t`             | `f new` (or `f -t`)                     |
| `f trc`           | `f new -r -c` (or `f -t -r -c`)         |
| `f S`             | `f big` (or `f -S`)                     |
| `f mL 10 2`       | `f -m 10 -L 2`                          |
| `f mLf: 10`       | `f -f 10`                               |
| `f s`             | `f big`                                 |
| `f l5`            | `f -L 5`                                |
| `f Downloads`     | `f ./Downloads` (no longer a chain)     |

For common combinations, add a custom alias to your shell profile:

```bash
# ~/.zshrc or ~/.bashrc
alias ftrc='f -t -r -c'
alias ftree='f -R -D'
alias fbig='f -S -r -m 50'
```

## Version

Bump to **0.7.0** (breaking change).
