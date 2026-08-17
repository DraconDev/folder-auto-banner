# folder-auto-banner 0.6.32

## Summary
**Chained lazy flags** — `f trc` is now equivalent to `f -t -r -c`.
Every character in the arg must be a valid lazy flag. No fallback:
`f trc` ALWAYS means `-t -r -c`, never a path. To show a banner for
a file or directory, use `./path`, `/abs/path`, or `~/path` (explicit
path indicators).

## What changed

### Routing priority in `src/main.rs`

Before (0.6.31):
1. Number → navigate
2. Known subcommand → use it
3. Single-char lazy flag → expand
4. Otherwise → path

After (0.6.32):
1. Number → navigate
2. Known subcommand → use it
3. Explicit path (starts with `.`, `/`, or `~`) → path
4. All-chars-are-lazy-flags → expand to chain
5. Otherwise → bare-word path (will fail unless valid)

### New functions

- **`expand_lazy_flags(arg) -> Option<Vec<char>>`** — if every
  character in `arg` resolves to a lazy flag (via `resolve_lazy_flag_char`),
  returns the list of canonical flag characters. Otherwise `None`.
- **`resolve_lazy_flag_char(c: char) -> Option<char>`** — checks
  the canonical `LAZY_FLAGS` list first, then the `LOWERCASE_ALIASES`
  map (e.g. `s` → `S`).
- **`is_explicit_path(arg) -> bool`** — returns true if `arg`
  starts with `.`, `/`, or `~`.

### Removed

- **`is_lazy_flag(arg) -> Option<char>`** — replaced by
  `expand_lazy_flags` which handles both single-char and multi-char.

### Examples

| Input | Expands to |
|-------|-----------|
| `f t` | `-t` (sort by time) |
| `f S` | `-S` (sort by size) |
| `f trc` | `-t -r -c` (time + reverse + compact) |
| `f tS` | `-t -S` (time + size) |
| `f tsaG` | `-t -S -a -G` (time + sizesort + hidden + git) |
| `f Downloads` | path `./Downloads` |
| `f ./Downloads` | explicit path |
| `f /abs/path` | explicit path |
| `f ~/Downloads` | explicit path |
| `f 1` | navigate to item 1 |
| `f banner` | subcommand |

### Stale tests disabled

Five integration tests for non-existent subcommands
(`test_stats_help`, `test_mv_help`, `test_rm_help`, `test_root_help`,
`test_do_help`) were disabled. These tests were written for
subcommands that don't exist (`stats`, `mv`, `rm`, `root`, `do`)
and were only passing because the old routing fell through to
banner's `--help`. With chained lazy flags, these all-flag-char
words now correctly expand to flag chains, not fall through. The
tests are commented out with a note explaining they can be
re-enabled when/if those subcommands are added.

## Why "no fallback"

The user established the rule: "we are never never looking for a
path just using lazy flags." This means:
- `f t` → always `-t` (never a path called `t`)
- `f trc` → always `-t -r -c` (never a path called `trc`)
- `f Downloads` → path (Downloads is not all-flag-chars)
- `f ./Downloads` → explicit path
- `f /abs/path` → explicit path

The no-fallback rule makes the behavior predictable: bare words
without `.`, `/`, or `~` are always lazy-flag chains.

## Validation

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (139 tests, down from 144 — 5
  stale tests disabled)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`
- Manual verification: `f trc` ≡ `f -trc`, `f tS` ≡ `f -tS`,
  `f Downloads` still works as path, `f ./Downloads` works as
  explicit path, `f 1` still navigates.

## Preserved behavior

- Single-char lazy flags still work: `f t`, `f S`, `f a`, etc.
- Numbers still navigate: `f 1`, `f 2`, etc.
- Subcommands still work: `f banner`, `f env`, `f install`, etc.
- Explicit paths still work: `f ./path`, `f /abs/path`, `f ~/path`.
- All explicit flags still work: `f -t`, `f -S`, `f -trc`, etc.
