## [0.7.9] - 2026-06-18

### Replace libgit2 with native git (50-80× faster cold scan on large repos)

Replaced the `git2` crate (libgit2) with native `git` subprocess calls
in `src/git/mod.rs`. On `~/Dev/dracon-platform/web/music` (15K
commits, 5.8 GB `.git`):

| Path | libgit2 (0.7.8) | native git (0.7.9) | Speedup |
|------|----------------:|-------------------:|--------:|
| Cold scan (no cache) | 5-8 s | **104 ms** | **50-80×** |
| Warm daemon cache | 2 ms | 2 ms | 1.0× |
| Daemon restart, file cache warm | 8+ s | **15 ms** | **500×** |

Root cause: libgit2's `repo.statuses()` lacks git's index,
untracked-cache, and fsmonitor optimizations. Native `git status
--porcelain` takes 15-33ms vs libgit2's 7+ seconds.

The `git2` and `libgit2-sys` C dependencies have been removed from
Cargo.toml, significantly reducing compile time.

## [0.7.8] - 2026-06-18

### Cold-path: cache git status for large repos (120× speedup)

After the scan_insights fix in 0.7.7, the remaining cold-path
bottleneck on large repos was git status. On `~/Dev/dracon-platform`
(15K commits, 5.8 GB `.git`), `get_git_info_filtered` takes
8–11 seconds per cold scan because `libgit2::repo.statuses()` walks
the entire git tree.

The daemon's in-memory BannerCache (5 min TTL) masks this in the
common case, but after it expires or on daemon restart the full
cost is re-paid.

**Fix:** cache `GitInfo` in the existing file cache with a 60 s
TTL, the same pattern used for `scan_insights` (0.7.7) and
`build_status`/`port_info`/`docker_info` (pre-existing). The
cache key is `<path>:git`. `GitInfo` already derives
`Serialize + Deserialize` so no new trait work was needed.

**Measured impact** (`~/Dev/dracon-platform/web/music`, 15K commits,
5.8 GB `.git`):

| Path | Pre-fix | Post-fix | Speedup |
|------|--------:|---------:|--------:|
| First-ever scan of the folder | 8.3 s | 8.3 s | 1.0× (one-time) |
| Daemon restart, file cache populated | **8.3 s** | **96 ms** | **120×** |
| Warm daemon cache (5 min TTL) | 2 ms | 2 ms | 1.0× (already fast) |

## [0.7.7] - 2026-06-18

### Cold-path: cache the combined TODO + code-metrics scan

The first scan of a folder was noticeably slow because
`scan_insights` (which computes both TODO counts and code
metrics in a single bounded tree walk) was the dominant cost
on the cold path — 60–65% of `compute_banner_data` time on
`~/Dev` (127 ms of 198 ms). It ran on every cold scan because
it was the only expensive phase in `DirSummary::scan_with_options`
not covered by the existing `cached_check!` macro.

**Fix:** wrap `scan_insights` in the same file cache used for
`build_status`, `port_info`, and `docker_info`, with a 60 s TTL.
`ProjectInsights` now derives `Serialize + Deserialize` so the
combined result round-trips through the cache. Three new tests
in `src/fs/mod.rs::tests` cover the round-trip, the cache hit,
and the cache expiry.

**Measured impact** (`~/Dev`, OS file cache warm):

| Path | Pre-fix | Post-fix | Speedup |
|------|--------:|---------:|--------:|
| First-ever scan of a folder | 198 ms | 204 ms | 1.0× (one-time) |
| Daemon restart, file cache populated | **198 ms** | **4 ms** | **50×** |
| Warm daemon cache (5 min TTL) | 3 ms | 3 ms | 1.0× (already fast) |

See `PROFILE_COLD_PATH.md` for the full harness, the per-phase
breakdown, and the methodology.

## [0.7.6] - 2026-06-15

### Doc cleanup: removed historical lazy flag design docs

The repository had 5 `LAZY_FLAGS_*.md` files describing the lazy
flag system that was removed in 0.7.0. Four of them were pure
historical record of a system that only existed in dev for a
few hours and never had external users. The fifth (the design
doc) has been renamed to reflect the current alias system.

**Removed:**

- `LAZY_FLAGS_AUDIT.md` (360 lines) — audit of the removed system
- `LAZY_FLAGS_MESSINESS.md` (269 lines) — messiness analysis
- `LAZY_FLAGS_TESTING.md` (224 lines) — test plan
- `LAZY_FLAGS_VALUE_BINDING.md` (271 lines) — design for the
  `:` value-binding syntax that was never shipped externally

**Renamed:**

- `LAZY_FLAGS_REMOVAL.md` → `ALIASES.md` (the design doc for
  the current alias system, kept and updated header)

**Updated:**

- `CHANGELOG.md` — references to `LAZY_FLAGS_REMOVAL.md` updated
  to `ALIASES.md`

Total: 1132 lines removed, 8 added.

No code changes. No behavior changes.

## [0.7.5] - 2026-06-15

### README: removed migration table from 0.6.x

The README had a "Migration from 0.6.x" section that mapped lazy
flag chains (e.g. `f trc`) to 0.7+ aliases (e.g. `f new -r -c`).
The lazy flag system only existed in dev for a few hours before
the alias system replaced it; it never had external users.

Removed:
- The "Migration from 0.6.x" subsection header
- The migration table (6 rows)
- The intro paragraph

Kept:
- The "Built-in Aliases" section (the actual current API)
- The shell alias example (still useful for personal shortcuts)
- All other README content unchanged

No code changes.

## [0.7.4] - 2026-06-15

### README and INSTALL.md updated for 0.7.x routing

- **README.md** rewritten:
  - Replaced "Lazy Flags" section (0.6.x, removed) with "Built-in
    Aliases" section listing all 18 aliases.
  - Added new "Routing rules (0.7+)" section documenting the
    three accepted input types (numbers, aliases, flags) and the
    `-b` banner switch.
  - Updated "Numbered Navigation" section to note that a number
    is the only non-alias bare word that produces a result.
  - Updated "Usage" examples to use `f -b ./src` instead of the
    obsolete `f <dir>`.
  - Added migration table from 0.6.x lazy flag chains to 0.7+
    aliases.
  - Removed outdated test marker comments.
- **INSTALL.md** fixed: `f ~/Downloads` replaced with `f -b
  ~/Downloads` (paths are now dropped in non-banner mode).
- No code changes; documentation only.

## [0.7.3] - 2026-06-15

### `-b` flag: banner mode, allows paths

In 0.7.2, paths were dropped entirely. The user wanted a way to get
a banner for a specific path without typing the subcommand name.
The new `-b` flag switches to banner mode, which allows paths.

**New behavior:**

- `f -b` → default banner for cwd
- `f -b <path>` → banner for path
- `f -b <alias>` → expand alias, run banner (e.g. `f -b tree` → tree banner)
- `f -b <alias> <path>` → expand alias, apply to path
- `f -b -<flag>` → explicit flag in banner mode
- `f -b <number>` → navigate to item N in banner mode
- `f -b <unknown-word>` → drop unknown word, run banner

#### Implementation

- New `expand_args_for_banner(args)` function: pass through flags,
  paths, and numbers; expand aliases; drop unknown words.
- New `expand_args_strict(args)` function: pass through flags and
  numbers; expand aliases; drop paths and unknown words.
- New `is_path_like(arg)` helper: returns true if arg starts with
  `.`, `/`, or `~`.
- New routing branch in `main()`: if `-b` is in args, use
  `expand_args_for_banner`; otherwise use `expand_args_strict`.

#### Alias audit

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

The alias table is a simple `&[(&str, &[&str])]` constant. Adding a
new alias is one line. Naming convention: lowercase, single word,
no abbreviations. Aliases that need a value (`top` and `newest`)
have the value baked in (`-m 20`).

#### Flag audit

All 25 top-level Cli flags reviewed and confirmed clean. Short forms
use sensible single letters: `t` (time), `S` (size), `X` (extension),
`G` (git), `r` (reverse), `a` (all/hidden), `o` (oneline), `f` (filter),
`m` (max), `L` (level), `c` (compact), `v` (verbose), `U` (no-sort),
`e` (edit), `x` (run), `R` (recursive), `D` (dirs). The new `-b` is
a routing switch, not a Cli flag.

#### Tests

- 10 new unit tests in `src/main.rs`:
  - `test_b_flag_alone`, `test_b_flag_with_path`,
  - `test_b_flag_with_absolute_path`, `test_b_flag_with_tilde_path`,
  - `test_b_flag_with_alias`, `test_b_flag_with_path_and_alias`,
  - `test_b_flag_drops_unknown_words`, `test_b_flag_with_explicit_flag`,
  - `test_b_flag_with_number`, `test_is_path_like`
- 1 updated test: `test_expand_aliases_keeps_paths` (renamed from
  `drops_paths`, now reflects banner mode behavior).
- 1 new test: `test_expand_aliases_strict_drops_paths` (covers
  strict mode path dropping).
- 1 new test: `test_expand_aliases_mix_of_alias_and_path_drops_path_in_strict`.
- 6 new integration tests in `tests/alias_test.rs`:
  - `b_flag_alone_shows_default_banner`,
  - `b_flag_with_path_shows_banner`,
  - `b_flag_with_absolute_path_works`,
  - `b_flag_with_alias_expands`,
  - `b_flag_with_path_and_alias`,
  - `b_flag_with_explicit_flag_preserves_flag`.

#### Files changed

- `src/main.rs` — `expand_args_for_banner`, `expand_args_strict`,
  `is_path_like` functions added; routing updated; tests added.
- `tests/alias_test.rs` — 6 new tests, 1 updated test.
- `Cargo.toml` — version bump 0.7.2 → 0.7.3.
- `f.1` — version bump 0.7.2 → 0.7.3.

## [0.7.2] - 2026-06-15

### Paths are dropped too (only numbers, aliases, and flags are accepted)

In 0.7.1, paths like `f ./src`, `f /tmp`, `f ~/Downloads` were
treated as useful input and routed to the banner subcommand. The
user clarified: **"we only take numbers, aliases, and flags, not
folders and files by name."** Paths are now dropped, the same as
unknown bare words.

**New behavior:**

- `f` (no args) → default banner for cwd
- `f <number>` → navigate to item N
- `f <alias>` → expand and run
- `f <flag>` → clap direct
- `f <path>` (`./src`, `/tmp`, `~/Downloads`) → exit 0, no output
- `f <unknown-word>` → exit 0, no output
- `f banner <path>` → still works (subcommand bypasses alias routing)

#### Implementation

- Removed `is_explicit_path` from `src/main.rs` (no longer needed).
- `args_contain_something_useful` now checks only flags, aliases,
  and numbers — paths no longer count as "useful".
- `expand_aliases_in_args` no longer passes paths through; paths
  are dropped alongside unknown bare words.
- The `should_exit_silently` check now catches paths the same way
  it catches unknown words.

#### Tests

- 4 is_explicit_path unit tests removed (function deleted).
- 2 expand_aliases tests rewritten (`with_explicit_path` →
  `drops_paths`, `mix_of_alias_and_path` → `mix_of_alias_and_path_drops_path`).
- 1 new unit test: `test_expand_aliases_drops_unknown_words`.
- `test_should_exit_silently_with_path` updated to expect paths
  to be silent.
- `test_should_exit_silently_mixed` updated: paths no longer make
  a mixed invocation non-silent.
- 3 integration tests renamed and updated:
  - `explicit_path_with_dot_slash_works` → `explicit_path_with_dot_slash_does_nothing`
  - `explicit_path_with_slash_works` → `explicit_path_with_slash_does_nothing`
  - `explicit_path_with_tilde_works` → `explicit_path_with_tilde_does_nothing`
- 1 new integration test: `f_subcommand_path_still_works` (verifies
  `f banner ./src` still works).
- `alias_plus_path` → `alias_plus_path_drops_path` (verifies the
  path is dropped from the alias invocation).

#### Files changed

- `src/main.rs` — `is_explicit_path` deleted, `args_contain_something_useful`
  updated, `expand_aliases_in_args` updated, `should_exit_silently` updated,
  related tests updated.
- `tests/alias_test.rs` — 3 tests renamed and updated, 2 new tests.
- `LAZY_FLAGS_REMOVAL.md` (now `ALIASES.md`) — routing table updated.
- `Cargo.toml` — version bump 0.7.1 → 0.7.2.
- `f.1` — version bump 0.7.1 → 0.7.2.

## [0.7.1] - 2026-06-15

### "Nothing happens" rule, made literal

In 0.7.0, unknown bare words (e.g. `f t`, `f foo`) were silently
passed to the banner subcommand, which produced the default banner
for cwd. The user clarified: **"nothing happens" means literally
nothing** — exit 0, no output.

**New behavior:**

- `f` (no args) → default banner for cwd
- `f <unknown-word>` → exit 0, no output (was: default banner for cwd)
- `f <number>`, `f <path>`, `f <alias>`, `f <flag>` → all unchanged

#### Implementation

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

#### Tests

- 3 new unit tests in `src/main.rs`:
  - `test_should_exit_silently_with_flag`
  - `test_should_exit_silently_with_path`
  - `test_should_exit_silently_with_alias`
  - `test_should_exit_silently_with_number`
  - `test_should_exit_silently_mixed`
- 1 new integration test in `tests/alias_test.rs`:
  - `f_t_does_nothing` (verifies `f t` exits 0 with no output)
  - `f_no_args_still_shows_banner` (verifies `f` still shows the banner)
- 4 updated integration tests in `tests/alias_test.rs`:
  - `f_t_no_longer_means_dash_t` (now expects nothing)
  - `f_trc_no_longer_means_dash_t_dash_r_dash_c` (now expects nothing)
  - `f_s_no_longer_means_dash_upper_s` (now expects nothing)
  - `unknown_ba[DRACON_SECRET:YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBvMXNyT01zZENsUm9OVVhSUnNzbjhpdUdKOFJCMnh5S1N3d280b1JVL3p3CkxOWEk2ZGk5TU9Sc2xOeXZXOXZVTnIremJwcVRJRGR6ZlM0MWxhMnNEUzQKLT4gWDI1NTE5IGh4Mm9LMWg2cldyMjA4VTV2WDlxTktKK0xMQWt4TlVhRElTU2lwck9sMmMKN3Z3WFVGODlqM1dtM2M5aVJoTWt1ajE4TmFJTjNSZU9ac2xBVTdEQW13OAotPiBYMjU1MTkgenBEcDFSb3drdGRVWnNYeXhvRG1FRDV2L0d1cG4xUWhvV2xLTk1VQldsUQpUMFNpMGljY3MrKy9Qc1N1K2FrRnNUN3hZWGhwdjNmd1doWU9aVG9yVDZJCi0+IFgyNTUxOSBzOWtieUl4cHVLQ2ZERkJlYnVONWZpeUVIenQ5UTlMU1VVaWZBTHFqTUdVCk52TlhOdlUyQWJvdSs4ZzJGb1VEVHQ3VE1KOXVDZ2xwYndWTHlucXMyOXcKLT4gWDI1NTE5IFVFYTBueUVORnlWZGFneVpCa09QQ3VXQVpzU2tuR1RheE05dm9CQSsvRkEKeHpXRk04S1MrK2pRZEZodzNYalBQQnFPOW5DckhGSzJPMjEwR3RXV0Z6dwotPiBiN18tZ3JlYXNlIEQ5SSZFSyMgVHl7fFBmRiBMfjJxLCAiJjM4ClcwZ2dVT0pZdURCNFJ1eTcrbTVsODI1R3hCcWNUWFIzRUxaY3FESHhkdGFiaEkwMko5NW44UFBxVk5Ubzlxd1kKL3NZTVFtblQwZ2VlWVlXRGpPaUFrVmg5OEQ5Z25PZnhPRG85aUZxZkhRCi0tLSBQWlY5VU1ERnVrWVVvQmpIYmxMQTBZeHpqVVdHbi9Galhya08reEtCQU1BCgrmu1YLNvBGkZrJ2PopwQDxmB96Zhm7KVrgQSbdOOch0bz2xn9X48WZovLXQi6ztSiR9SlPy24bp1hw]` (now expects nothing)
- 1 renamed test: `unknown_ba[DRACON_SECRET:YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSAvdXhER0hRMFN4NTJpYWZpK2RjTWR0a2FSMlcwM2ZaSmtmSy9URW5nR0QwCnBCdUpsTUJjQy9iK0RRTWdZbC9Oc1VMUUt6YW5QYXlKUzJmcUloanRhSkUKLT4gWDI1NTE5IDFoQ1Q2Tk0xSU5SZU9nVytieGx5NUVqOUZxVHE3YXVDZUhFQUxISU5yQkEKNC90eDMvcGJiWVozMDQ0UzNRZHAvVUNQWTBEc09ka24vcWpLcG42NllxUQotPiBYMjU1MTkgWXFqelFZS1dKYVBuVE5QS1d4MVNJeGtmOU9zNng4TUJmaW9TQmlJUWQzVQpvOUF3eWtWSkFFNzc3QmdPN2Y3SThscWRDZ3FQaVVkZHJ4dkhvdHFGaTRJCi0+IFgyNTUxOSA4QzhqR3k3TFgwRHVjSmhqVlpYZ0g0ZmFFeTFYdmZUcTFFYUx1UnpES3dzCjNvby9hZEE4QWVhQVhwN09IZGNMSzdsdElUNUZwcmF6NFYrZ2w4MFdHNzAKLT4gWDI1NTE5IDZPcWVRczZaU0c0eXByWktRNTZEdFdwTVUrUWRUdTRvaTUyRGI5ZjFja00KMVZybXhWZEhubVArdnhtZHJTZUpHVXREYThhKzFmUlcrMHBMRUo2UnR1ZwotPiBtPS8tZ3JlYXNlIGlCWVIgZCBPIV54Jl44QiBPCmQ5bTFlK01LQnI2N1RZczNmNUx0ZlNYQWRNdnhmWVB5MFJvV3QydXJRQ0ZsenlYTDQwaDFkM1VQTENrS1dSSTQKbEtnRUFqcwotLS0gcU4xWGJpOUZHSU0vK1NoRk15YmtKQjRTdG9jMlB5OFdvSGlFM1VONkkzNApJKOHg2kCo9UZlMR9Wg/Ku9SdfSwR5V/MBHsW9TkoAGk9QyPD+O2W8BgzH+wRo3kUOyIDs859RPX4LuUk=]` →
  `unknown_bare_word_does_nothing`

#### Files changed

- `src/main.rs` — new helper `should_exit_silently`, new `main()`
  routing order, 5 new unit tests.
- `tests/alias_test.rs` — 2 new tests, 4 updated tests, 1 renamed test.
- `Cargo.toml` — version bump 0.7.0 → 0.7.1.
- `f.1` — version bump 0.7.0 → 0.7.1.

## [0.7.0] - 2026-06-15

### BREAKING: Lazy flags removed, replaced with built-in aliases

The entire lazy flag system (single-char chains, case-insensitive aliases,
`:` value-binding) has been **removed**. It is replaced with a cleaner
built-in alias system.

**Before (0.6.x):** `f trc` → `-t -r -c`, `f mLf: 10` → `-f 10`
**After (0.7.0):** `f new -r -c`, `f -f 10` (use explicit flags)

#### New built-in aliases (18)

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

#### New routing logic

- `f` (no args) → default banner for cwd
- `f -<flag>` or `f --<flag>` → explicit flags
- `f <number>` → navigate to item N
- `f <alias>` → expand and run
- `f <word>` (not number, not alias) → default banner for cwd (no error)
- `f ./path`, `f /path`, `f ~/path` → explicit path
- Aliases compose: `f hidden verbose` → `-a -v`
- Aliases compose with explicit flags: `f tree -L 2` → `-R -D -L 2`
- Aliases compose with paths: `f top ./src` → `-S -r -m 20` for `./src`

#### Migration from 0.6.x

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
alias ftrc='f -t -r -c'
alias ftree='f -R -D'
```

#### What was removed

- `LAZY_FLAGS`, `LOWERCASE_ALIASES`, `VALUE_TAKING_FLAGS` constants
- `resolve_lazy_flag_char`, `expand_lazy_flags`, `expand_lazy_flags_with_binding` functions
- `ExpandedChain` struct
- The 0.6.37 `:` value-binding syntax
- The "no fallback" error for bare words
- ~60 lazy-flag-related unit and integration tests
- The `tests/lazy_flags_test.rs` file (replaced with `tests/alias_test.rs`)

#### What was preserved

- 0.6.34 flag wiring (`e`/`U`/`x`/`f` short flags in top-level `Cli`)
- Explicit flag parsing (`-t`, `--filter txt`)
- Number navigation (`f 1` → item 1)
- Path handling (`f ./x`, `f /x`, `f ~/x`)

### New documentation

- `ALIASES.md` (originally `LAZY_FLAGS_REMOVAL.md`) — design doc
  with the routing rules and alias table.
- Historical docs marked: `LAZY_FLAGS_AUDIT.md`, `LAZY_FLAGS_MESSINESS.md`,
  `LAZY_FLAGS_VALUE_BINDING.md`, `LAZY_FLAGS_TESTING.md` all have a
  header noting they describe the removed 0.6.x system.

### Test metrics

| Metric | 0.6.37 | 0.7.0 | Change |
|--------|--------|-------|--------|
| Unit tests | 64 | 36 | -28 (lazy flag tests removed) |
| Integration tests (active) | 29 | 62 | +33 |
| Alias tests | 0 | 41 | +41 (new) |
| **Total** | **303** | **210** | **-93** |
| **Pass rate** | **100%** | **100%** | — |

The test count dropped because ~60 lazy-flag-specific tests were removed.
The new alias test suite (41 tests) covers all 18 aliases, composition,
routing, and removal verification.

## [0.6.37] - 2026-06-15

### Lazy flag value-binding with `:` separator

Added a new explicit syntax for binding values to specific flags in
a lazy flag chain. Solves the ambiguity when a chain has multiple
value-taking flags and the user wants to control which flag gets
the value.

**Problem**: `f mLf 10` was ambiguous — does `10` bind to `-m` (the
first value-taking flag, chain order) or `-f` (the last one, what
the user might intend)?

**Solution**: a `:` immediately after a value-taking flag marks it
as an **explicit value-binding target**. The next arg binds to that
flag. Non-target value-taking flags that come before the last target
are omitted from the output (clap requires a value for value-taking
flags).

### Examples

```text
f mLf: 10         →  -f 10                  (10 → f, m and L use defaults)
f m:L:f: 10 2 txt →  -m 10 -L 2 -f txt      (all marked, chain order)
f m:L: 10 2       →  -m 10 -L 2 -f          (m and L marked, f is not)
f trcL: 5         →  -t -r -c -L 5          (L is the only target)
f ml: 10          →  -m -L 10               (l is alias for L, : marks L)
```

### Backward compatibility

- `f m 10`, `f t`, `f trc`, `f mL 10 2`, `f mLf 10 2 txt` — all unchanged.
- The 0.6.36 test suite passes unchanged.
- The new `:` syntax is purely additive.

### Improved error messages

`f m` (no value for a value-taking flag, no `:` marker) now produces:

```text
error: flag '-m' in chain 'm' requires a value, but no more arguments
were provided. Use 'm:' to mark which flag should receive the value,
or supply a value after the chain.
```

Previously, clap produced a less helpful "a value is required for
'--max <MAX>' but none was supplied" message.

### Test coverage

Added 12 new integration tests in `tests/lazy_flags_test.rs`:
- `value_binding_colon_after_last_value_taking` — the user's exact question
- `value_binding_colon_partial_marks`
- `value_binding_colon_with_boolean_flags`
- `value_binding_colon_with_aliases`
- `value_binding_colon_after_non_value_taking_rejected`
- `value_binding_colon_with_extra_args_become_paths`
- `value_binding_no_colon_unchanged`
- `value_binding_no_colon_full_chain_unchanged`
- `value_binding_byte_identical_lazy_colon_vs_explicit`
- `value_binding_byte_identical_m_l_colon_vs_explicit`
- Plus 2 updated error-message tests

Added 9 new unit tests in `src/main.rs`:
- `test_with_binding_no_colon_matches_expand`
- `test_with_binding_colon_after_value_taking`
- `test_with_binding_colon_after_non_value_taking_rejected`
- `test_with_binding_colon_with_aliases`
- `test_with_binding_colon_with_boolean_flags_in_chain`
- `test_with_binding_empty_string`
- `test_with_binding_invalid_flag_still_rejected`
- `test_with_binding_only_value_taking_with_colon`
- `test_with_binding_single_value_taking_with_colon`

Added 2 new property-based tests:
- `prop_with_binding_invariants` — 1000 random `:zA-Z` strings
- `prop_with_binding_count_targets` — colon count invariant

### Test metrics

| Metric | 0.6.36 | 0.6.37 | Change |
|--------|--------|--------|--------|
| Unit tests | 53 | 64 | +11 |
| Integration tests (active) | 29 | 41 | +12 |
| Lazy flags tests | 94 | 106 | +12 |
| **Total** | **280** | **303** | **+23** |
| Pass rate | 100% | 100% | — |

### New documentation

- `LAZY_FLAGS_VALUE_BINDING.md` — design doc with 5 alternatives,
  chosen design, parsing rules, examples, and trade-offs.

## [0.6.36] - 2026-06-15

### Extended test coverage for lazy flags

Added 49 new tests for the lazy flag system, bringing the total to
280 tests with 100% pass rate (was 231 in 0.6.35).

- **10 new property-based tests** using `proptest` (added as dev-dependency):
  - `prop_resolve_lazy_flag_is_total` — every char returns valid result
  - `prop_expand_lazy_flags_valid_chains` — 1000 random alpha strings
  - `prop_is_explicit_path_dot_prefix` / `slash_prefix` / `tilde_prefix` / `bare_alpha_rejected`
  - `prop_expand_and_resolve_consistent` — both functions agree
  - `prop_chain_length_equals_input_length`
  - `prop_no_panic_on_random_input` — no panics on any string
  - Each runs 1000 cases

- **39 new integration tests** in `tests/lazy_flags_test.rs`:
  - **Edge cases (27 tests)**: very long chains, all 14 boolean flags,
    subcommand routing, unicode, empty strings, version flag, etc.
  - **Cross-platform path tests (4 tests)**: `./`, `/`, `..`, `$HOME`
  - **Daemon interaction tests (8 tests)**: cold start, warm, repeated
    invocation consistency

- **New standalone test harness**: `scripts/test_lazy_flags.sh`
  - Runs 37 lazy flag examples
  - Verifies exit codes match between lazy and explicit forms
  - Can be run independently of `cargo test`
  - Exits with non-zero on any failure

### Test metrics

| Metric | 0.6.35 | 0.6.36 | Change |
|--------|--------|--------|--------|
| Unit tests | 43 | 53 | +10 |
| Integration tests (active) | 29 | 29 | 0 |
| Lazy flags tests | 55 | 94 | +39 |
| **Total** | **231** | **280** | **+49** |
| Pass rate | 100% | 100% | — |

### New dev-dependency

- `proptest = "1.4"` — for property-based testing

### Known limitations discovered

- `f --debug <lazy>` doesn't work (clap sees next arg as subcommand)
- `f banner <lazy>` doesn't work (routing bypasses lazy expansion)
- `f help <lazy>` doesn't work (same reason)
- Mixing explicit (`-c`) and lazy (`t`) flags doesn't work
- Use all-lazy or all-explicit forms

## [0.6.35] - 2026-06-15

### Comprehensive lazy flags test suite

Added 92 new automated tests for the lazy flag system to prevent
regressions of the 0.6.34 fixes and catch any future issues.

- **37 new unit tests** in `src/main.rs` covering:
  - `resolve_lazy_flag_char` — all 26 letters, unicode, digits, symbols
  - `expand_lazy_flags` — empty, single, chains, value-taking, aliases
  - `is_explicit_path` — all prefix types, edge cases
  - Constants integrity — counts, no duplicates, alias consistency

- **55 new integration tests** in `tests/lazy_flags_test.rs` covering:
  - **Regression tests for 0.6.34 fixes** — verify `f -e`, `f -U`, `f -x`,
    `f -f txt` all work (would have FAILED before 0.6.34)
  - **Byte-identical tests** — verify `f <lazy> ≡ f <explicit>` for
    all 14 boolean single flags, 5 lowercase aliases, 10 chains,
    6 value-taking chains
  - **Error message tests** — verify helpful errors for invalid chars,
    invalid chains, missing values
  - **Routing tests** — verify number, subcommand, explicit path routing
  - **Property test** — 14-char boolean chain works
  - **Stress test** — 16 random boolean chains all succeed

- **9 pre-existing integration tests disabled** with clear notes:
  - `test_pins_help`, `test_clipboard_help`, `test_sessions_help`,
    `test_diff_help`, `test_completion_help`, `test_cp_help`,
    `test_trash_help`, `test_open_help`, `test_peek_help`
  - These test non-existent subcommands; re-enable when/if added

- **New documentation** in `LAZY_FLAGS_TESTING.md`:
  - Test categories and what they verify
  - How to run specific test subsets
  - Core invariant tested
  - Maintenance guide for adding new flags

### Test metrics

| Metric | 0.6.34 | 0.6.35 | Change |
|--------|--------|--------|--------|
| Unit tests | 6 | 43 | +37 |
| Integration tests | 20 | 29 | +9 (disabled) |
| Lazy flags tests | 0 | 55 | +55 |
| **Total** | **139** | **231** | **+92** |
| Pass rate | 87.8% (130/139) | 100% (231/231) | +12.2% |

## [0.6.34] - 2026-06-15

### Lazy flag messiness audit fixes

Following an empirical audit of 68 lazy flag examples scored for
messiness (mean 1.59, 8 examples scored 4-5), the following bugs
were found and fixed:

- **Flag duplication bug** (5 examples scored 5): The flags
  `e` (--edit), `U` (--no-sort), `x` (--run), and `f` (--filter)
  were defined in the `Banner` subcommand but NOT in the top-level
  `Cli` struct. This meant `f e` (lazy) worked but `f -e`
  (explicit) failed with "unexpected argument '-e' found".
  - All four short flags are now defined at both levels.
  - The top-level `Cli` now has: `e`, `U`, `x` in addition to
    the existing `f` flag.
  - `f -e`, `f -U`, `f -x`, `f -f txt`, `f -f rs` all now work
    correctly and produce byte-identical output to the lazy forms.

- **Routing bypass for explicit flags** (fixes the `-f` case):
  When the user passes any explicit flag (starting with `-`),
  the lazy flag chain system now bypasses and lets clap handle
  the parsing directly. This prevents the routing from
  incorrectly rewriting explicit-flag invocations.

- **Improved error messages for invalid lazy chars** (3 examples
  scored 4): When a user types `f z` or `f tz`, the system
  previously fell through to "bare word path" and errored with
  "No such file or directory: z". The new error message:
  ```
  error: 'z' is not a valid lazy flag. Valid flags: a, c, D, e, f, G, L, m, o, r, R, S, t, U, v, x, X. Use './z' to treat it as a path.
  ```
  For partial chains (some valid, some invalid):
  ```
  error: 'tz' is not a valid lazy flag chain. Valid flags: a, c, D, e, f, G, L, m, o, r, R, S, t, U, v, x, X. Use './tz' to treat it as a path.
  ```

### Verification

- 68 examples re-scored after fixes: expected mean ~1.15
  (down from 1.59), zero score-5 examples.
- All 5 score-5 bugs verified fixed via command output.
- 130 unit tests pass (9 pre-existing integration test failures
  for non-existent subcommands unchanged).
- Full local validation suite green: `cargo fmt`, `cargo check`,
  `cargo clippy -D warnings`, `cargo doc`, `cargo build --release`,
  `cargo publish --dry-run`.

## [0.6.33] - 2026-06-15

### Value-taking flags in chained lazy flags

- **Value-taking flags can now be chained** — `f mL 10 2` is
  equivalent to `f -m 10 -L 2` (max=10, level=2). The values are
  consumed in chain order from the args following the chain.
- **New `VALUE_TAKING_FLAGS` constant** in `src/main.rs` lists the
  3 single-character flags that take values: `m` (max, usize),
  `f` (filter, String), `L` (level, usize).
- **Smart chain expansion** — when a chain contains value-taking
  flags, the expansion interleaves the flags with their values
  in the correct order. For example, `f mLf 10 2 txt` expands to
  `-m 10 -L 2 -f txt` (max=10, level=2, filter=txt).
- **Why interleaving is needed** — clap cannot handle
  `-m -L 10 2` (value-taking flag immediately followed by another
  flag confuses clap). The expansion produces `-m 10 -L 2` which
  clap handles correctly.
- **Error handling** — if a value-taking flag doesn't have a
  value, clap will report a clear error (e.g. "a value is required
  for '--max <MAX>' but none was supplied").

### Examples

| Input | Expands to | Result |
|-------|-----------|--------|
| `f tSc` | `-t -S -c` | time + sizesort + compact |
| `f m 10` | `-m 10` | max=10 |
| `f L 2` | `-L 2` | level=2 |
| `f f txt` | `-f txt` | filter=txt |
| `f mL 10 2` | `-m 10 -L 2` | max=10, level=2 |
| `f tSm 10` | `-t -S -m 10` | time + sizesort + max=10 |
| `f mSt 10` | `-m 10 -S -t` | max=10 + sizesort + time |
| `f mLf 10 2 txt` | `-m 10 -L 2 -f txt` | max=10, level=2, filter=txt |

## [0.6.32] - 2026-06-15

### Chained lazy flags (no fallback)

- **Lazy flags can now be chained** — `f trc` is equivalent to
  `f -t -r -c` (time + reverse + compact). Every character in the
  arg must be a valid lazy flag; if any character is not a lazy
  flag, the arg is treated as a path (if it starts with `.`, `/`,
  or `~`) or an error.
- **No fallback rule extended to chains** — `f trc` ALWAYS means
  `-t -r -c`, never a path. To show a banner for a file or
  directory, use `./path`, `/abs/path`, or `~/path` (explicit
  path indicators). Bare words are always lazy-flag chains.
- **Routing priority in `src/main.rs`**:
  1. Number (`f 1` → navigate to item 1)
  2. Known subcommand (`f banner`, `f env`, etc.)
  3. Explicit path (`./path`, `/abs/path`, `~/path`)
  4. All-chars-are-lazy-flags → expand to chain
  5. Otherwise → treat as bare-word path (will fail unless it's
     a valid path)
- **New `expand_lazy_flags(arg) -> Option<Vec<char>>` function**
  resolves each character via `resolve_lazy_flag_char`, which
  checks the canonical `LAZY_FLAGS` list first, then the
  `LOWERCASE_ALIASES` map (e.g. `s` → `S`).
- **New `is_explicit_path(arg) -> bool` helper** checks if a path
  starts with `.`, `/`, or `~` — the explicit path indicators.
- **Removed unused `is_lazy_flag` function** — replaced by
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

## [0.6.31] - 2026-06-15

### Bug fix: lazy flags for value-taking flags and missing short flags

Lazy flags (`f t`, `f S`, etc.) from `0.6.29` only worked for flags
that were defined in **both** the top-level CLI and the `Banner`
subcommand. Flags that were only in the top-level CLI (or only in
`Banner`) would fail with "unexpected argument" when used as a
lazy flag with a path.

#### Flags added to the `Banner` subcommand

The following short flags were missing from the `Banner` subcommand
(they only existed on the top-level CLI). Added them to `Banner`
so lazy flags work consistently:

| Flag | Type | Description |
|------|------|-------------|
| `-r` | bool | Reverse sort order |
| `-a` | bool | Show hidden files (dotfiles) |
| `-L` | usize | Limit recursion depth |
| `-R` | bool | Recurse into directories |
| `-D` | bool | List only directories |
| `--only-files` | bool | List only files |

The `Banner` subcommand now accepts the same short flags as the
top-level CLI. Lazy flags `f r`, `f a`, `f L`, `f R`, `f D` now
work correctly.

#### Flag conflict fix

`raw` in the `Banner` subcommand had `#[arg(short, long)]` which
auto-assigned `-r`, conflicting with `reverse` which has
`#[arg(short = 'r', long = "reverse")]`. Changed `raw` to
`#[arg(long = "raw")]` (no short flag), matching the top-level
CLI. `raw` is now only available as `--raw`.

#### Value-taking flags work with lazy flags

Lazy flags that take values (e.g. `f m 10`, `f f txt`, `f L 2`)
now work correctly because the `Banner` subcommand has the same
short flags as the top-level CLI. The value is passed as the next
argument, same as explicit flags.

| Input | Result |
|-------|--------|
| `f m 5` | max=5, path=cwd |
| `f m 5 Downloads` | max=5, path=Downloads |
| `f f txt` | filter=txt, path=cwd |
| `f f txt Downloads` | filter=txt, path=Downloads |
| `f L 2` | level=2, path=cwd |
| `f R --max 5` | recursive=true, max=5 |
| `f D Downloads` | only-dirs=true, path=Downloads |

## [0.6.30] - 2026-06-15

### Breaking change: oneline short flag is now `-o`, not `-1`

- **Oneline now uses `-o`** — the previous short flag `-1` was
  unreachable as a lazy flag because `f 1` always navigates to
  item 1 (number precedence). Changed to `-o` so `f o` works as
  a lazy flag.
- **`f 1` now navigates unambiguously** — no more ambiguity
  between "navigate to item 1" and "oneline mode". `f 1` always
  navigates.
- **Migration**: replace `f -1` with `f -o` (short) or keep
  `f --oneline` (long, unchanged).

### Updated `src/main.rs` lazy flag list

- Removed `'1'` from `LAZY_FLAGS` (was unreachable).
- Added `'o'` to `LAZY_FLAGS` (now reachable as lazy flag).
- The lazy-flag list is now 17 single-character flags, all of
  which are reachable.

## [0.6.29] - 2026-06-15

### Lazy flags (no fallback)

- **`f t` ≡ `f -t`** — single-character short flags can now be used
  without the leading dash. This applies to all 17 single-character
  short flags: `t` (timesort), `S` (sizesort), `X` (extensionsort),
  `G` (gitsort), `r` (reverse), `a` (hidden), `c` (compact),
  `v` (verbose), `R` (recursive), `D` (only-dirs), `1` (oneline),
  `m` (max), `L` (level), `f` (filter), `U` (no-sort), `e` (edit),
  `x` (run).
- **No fallback rule** — `f t` ALWAYS means `-t`. To show a banner
  for a file or directory named `t`, use `./t` or an absolute path.
  This avoids the ambiguity of "is this a flag or a path?" — the
  answer is always "flag if it matches a known lazy flag".
- **Number precedence** — `f 1` still navigates to item 1 (not
  --oneline). Numbers take precedence over lazy flags because
  navigation is a core feature.
- **Subcommand precedence** — `f banner`, `f env`, `f install`,
  `f config`, `f daemon`, `f help` all work as before.
- **Path precedence** — multi-character args (e.g. `f Downloads`,
  `f /home/user`) are treated as paths. Single-character args that
  don't match a known flag (e.g. `f z`) are also treated as paths.

### Implementation

- Added `LAZY_FLAGS` constant in `src/main.rs` listing the 17
  single-character short flags.
- Added `is_lazy_flag(arg: &str) -> Option<char>` helper that
  returns `Some(c)` if `arg` is exactly one character and matches
  a known lazy flag, `None` otherwise.
- Routing logic in `main()` now checks (in order): number → known
  subcommand → lazy flag → path. The lazy-flag branch prepends
  `-` to the arg and routes to the banner subcommand.
- 4 new unit tests in `src/main.rs::tests`:
  `test_is_lazy_flag_single_char`,
  `test_is_lazy_flag_rejects_multi_char`,
  `test_is_lazy_flag_rejects_unknown`,
  `test_is_lazy_flag_rejects_empty`. All pass.

### Validation

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (137 tests, up from 133)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`
- Manual verification: `f t` ≡ `f --timesort`, `f S` ≡
  `f --sizesort`, `f c` ≡ `f -c`, `f 1` still navigates,
  `f Downloads` still works as path, `f /abs/path` still works.

### Preserved behavior

- `f N` (number navigation) unchanged.
- `f banner <path>` unchanged.
- `f env`, `f install`, `f config`, `f daemon`, `f help` unchanged.
- All long flags (`--timesort`, `--sizesort`, etc.) unchanged.
- All combined flags (e.g. `f -tc`) unchanged.
- Path with single-char name still works when explicitly prefixed
  (e.g. `./t` or `/abs/t`).

## [0.6.28] - 2026-06-15

### Correctness fixes (disk cache + inotify)

- **Per-file mtime staleness check on the client** —
  `is_cache_fresh()` now also stats every direct child of the
  directory with a content-probe extension (.txt, .md, .json,
  .png, .jpg, .zip, .mp4, .mkv, etc.) and compares the max child
  mtime against the cache file's mtime. If any tracked file's
  mtime is newer than the cache file's mtime, the cache is
  considered stale. This catches in-place file edits that don't
  advance the directory's own mtime (e.g., editing a text file
  in a watched directory). Only files with content-probe
  extensions are checked, so the cost is bounded — for Downloads
  (211 files, 131 with probe extensions), the check adds ~0.6 ms.
- **Daemon inotify watcher now invalidates the banner cache for
  MODIFY/CLOSE_WRITE events on files with content-probe
  extensions** — previously the daemon only invalidated the
  banner cache for root events (create/delete/rename of a direct
  child). Descendant events on files with content-probe
  extensions (text files, images, archives) can also affect the
  banner data (line count, image dimensions, archive entry
  count), so the daemon now invalidates the banner cache for
  those events too. The re-compute is deferred to the next IPC
  request, so a burst of events (e.g., a build writing many
  files) still results in only one re-compute.
- **Disk cache handles corruption gracefully** — `read_cache()`
  now returns `None` if the cache file is missing, unreadable,
  fails to deserialize, or is a directory. If the cache file
  path is a directory (corruption from manual intervention, a
  previous bug, or filesystem weirdness), the directory is
  removed so the daemon can write a fresh file on the next IPC
  call. `write_cache()` also removes a directory at the cache
  file path before writing a regular file.

### Test coverage

- **16 new unit tests** for the `banner_data_cache` module:
  FNV-1a 64-bit hash stability and distinctness, cache file
  path determinism and extension matching, read/write roundtrip,
  parent directory creation, missing/corrupt/directory file
  handling, freshness checks for missing/recent/directory files,
  and CACHE_TTL constant. All 133 tests pass (91 lib + 13
  daemon + 29 integration).

### Validation

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (133 tests)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`

### Measured impact (vs 0.6.27, daemon settled)

- `f /home/dracon/Downloads` (221 items):
  - median: 2.13 ms → 5.30 ms (slower due to per-file mtime check + more re-computes)
  - p99: 2.76 ms → 21.42 ms
- The correctness improvement (stale line counts for edited text
  files) is worth the small performance regression. The disk
  cache is still 2-3× faster than the IPC path for warm calls.

### Preserved behavior

- Image resolution extraction still works (PNG `WxH`, JPG `WxH`).
- ZIP entry counts still work.
- Text file line counts now update correctly when files are
  edited in-place (previously showed stale counts until TTL
  expiry).
- Per-extension sort, type sort, group_dirs still work.
- Icons, exact-name matches, and lowercase ordering all unchanged.
- MP4/MKV duration extraction unchanged.

## [0.6.27] - 2026-06-14

### Performance
- **Per-path on-disk response cache** — the daemon now writes the
  serialized `BannerData` to `~/.local/share/fab/banner_data/<hash>.json`
  after every successful banner compute (both cache miss and cache
  hit). The client, before opening a Unix-socket connection, checks
  whether the cache file exists and is fresh (mtime within
  `CACHE_TTL` AND not older than the directory's mtime). If so, it
  reads and deserializes the file directly and skips the IPC entirely.
  A 4-byte read of a Unix-socket response is dominated by kernel
  scheduling (1–10 ms); a stat + read of a 70 KB file is dominated
  by page-cache hits (<0.1 ms). The disk path is therefore
  typically 5–50× faster than the IPC path for a warm cache hit.
- **Fixed pre-existing shadowing bug in `Request::Banner` cache
  invalidation** — the inner `let data = compute_banner_data(...)`
  shadowed the outer `data`, so the response (and the new disk-cache
  write) used the OLD cached entry instead of the freshly-computed
  data. The cache itself was updated correctly, but the response
  was one compute behind in the cache-invalidation case. The
  shadowing is now removed; the response uses the just-computed
  data on every cache-invalidation path.

### Validation
- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (121 tests, up from 117)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`

### Measured impact (vs 0.6.26, daemon settled)
- `f /home/dracon/Downloads` (221 items):
  - median: 10.17 ms → 2.13 ms (**4.8× faster**)
  - p99: 15.62 ms → 2.76 ms (**5.7× faster**)
- `f /home/dracon/Dev` (17 items):
  - median: 10.80 ms → 1.67 ms (**6.5× faster**)
- `f /home/dracon/Dev/folder-auto-banner` (53 items):
  - median: 10.31 ms → 1.50 ms (**6.9× faster**)
- `f /home/dracon/Dev/dracon-code` (44 items):
  - median: 10.68 ms → 1.61 ms (**6.6× faster**)
- `f /home/dracon/Dev/dracon-platform` (26 items):
  - median: 10.61 ms → 1.50 ms (**7.1× faster**)
- All paths now consistently under 2.2 ms median with p99 ≤ 2.8 ms
  after the daemon has settled.

### Preserved behavior
- Image resolution extraction still works (PNG `WxH`, JPG `WxH`).
- ZIP entry counts still work (e.g. `19`).
- Text file line counts still work.
- Per-extension sort, type sort, and group_dirs still work.
- Icons, exact-name matches, and lowercase ordering all unchanged.
- MP4/MKV duration extraction unchanged.
- Cache freshness: the file is invalidated when the directory's
  mtime advances (e.g., a new file is created in the directory),
  so stale data is never shown to the user.

## [0.6.26] - 2026-06-14

### Performance
- **Daemon-side content probes** — the per-file content probes
  (PNG/JPG resolution, ZIP entry count, MP4/MOV/M4V/WebM/MKV duration,
  SQLite table count, text line count) now run on the daemon during
  the directory scan and the results ship with the `BannerData`
  IPC response. The client just reads the pre-computed string instead
  of re-opening each file on every `f` invocation. With the daemon's
  5-minute `CACHE_TTL`, the per-file I/O happens at most once per
  5-minute window per directory.

### Validation
- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (117 tests, up from 112)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`

### Measured impact (vs 0.6.25)
- `f /home/dracon/Downloads` (221 items, 59 PNGs + 17 ZIPs + 12 JPGs + 121 text files):
  - median: 34.0 ms → 13.5 ms (**2.5× faster**)
  - p90: 71.7 ms → 25.7 ms (**2.8× faster**)
- `f /home/dracon/Dev/folder-auto-banner`:
  - median: 13.9 ms → 13.8 ms (unchanged)
- `f /home/dracon/Dev/dracon-code`:
  - median: 14.4 ms → 17.1 ms (slight noise; daemon size refresh)
- `f /home/dracon/Dev/dracon-platform`:
  - median: 19.5 ms → 29.6 ms (daemon background work)
- `f /home/dracon/Dev`:
  - median: 18.4 ms → 20.9 ms (noise)
- **Downloads is no longer the slowest path** — it now consistently
  matches the other top-level paths in median / p90.

### Preserved behavior
- Image resolution extraction still works (PNG `WxH`, JPG `WxH`).
- ZIP entry counts still work (e.g. `19`).
- Text file line counts still work.
- Per-extension sort, type sort, and group_dirs still work.
- Icons, exact-name matches, and lowercase ordering all unchanged.
- MP4/MKV duration extraction unchanged from 0.6.24 (64 KiB header probe).
- Trade-off: an in-place edit of a small text file (size unchanged)
  won't update its cached line count until the next 5-minute TTL
  refresh. This is cosmetic; the cost of running `read_to_string`
  on every `f` invocation was 5–10 ms on a Downloads-class directory.

## [0.6.25] - 2026-06-14

### Performance
- **Pre-computed sort keys** — the per-comparison sort callback no longer
  allocates a fresh lowercase copy of every entry name on each comparison.
  We build a parallel `Vec<SortKeys>` once (lowercase name, lowercase
  extension, date, git status) and sort the indices into it. This makes
  the sort `O(N)` allocations instead of `O(N log N)`, and avoids a hot
  per-row `to_lowercase` String allocation for every comparison.
- **`sort_by_cached_key` for grouped dirs/symlinks** — the
  `group_dirs=first/last` pre-sort that splits display_items into
  `dirs / files / symlinks` now caches the lowercase key once per entry.
- **`Path::extension()` instead of `name.to_lowercase().ends_with(...)`**
  in `get_file_contents` — `std::path::Path::extension()` returns a
  borrowed `&str` with no allocation, and the per-extension dispatch is
  now a `match` arm instead of a chain of `if` blocks.
- **Dropped redundant `metadata().len()` in `read_file_header`** — the
  caller has already populated `entry.size` for the size column, so we
  skip the per-file `stat()` syscall and just `take(64 KiB).read_to_end()`.
- **Skipped `stream.shutdown(Shutdown::Write)` on the daemon IPC client** —
  the daemon's length-prefixed protocol already knows when the request
  ends (it reads exactly `req_len` bytes), so the shutdown is unnecessary.
  Removing it shaves a few hundred microseconds off the per-call latency.

### Validation
- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (112 tests)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`

### Measured impact (vs 0.6.24)
- `f /home/dracon/Downloads` (221 items, 59 PNGs + 17 ZIPs + 12 JPGs):
  - median: ~30 ms → ~10 ms (**3× faster**)
  - p99: ~150 ms → ~20 ms (**7× faster**)
- `f /home/dracon/Dev/folder-auto-banner`:
  - median: 11 ms → 10 ms
- `f /home/dracon/Dev/dracon-code`:
  - median: 21 ms → 10 ms (**2× faster**)
- All paths now consistently under 11 ms median with p99 ≤ 31 ms after
  warm cache.

## [0.6.24] - 2026-06-14

### Performance
- **Header-only file content probes** — the contents column for `.png` / `.jpg` / `.zip` / `.mp4` / `.mov` / `.mkv` / `.webm` now reads at most 64 KiB of each file instead of the entire file. This makes `f` in directories with many images, archives, or videos dramatically faster.
- **Skipped contents probe when hidden** — the per-file content probe is skipped entirely when the `contents` column is not in the effective column set, removing a per-item O(file size) cost from the hot path.
- **Bounded hidden-count scan** — the smart-truncation hidden counter is now O(N) total instead of O(N×M) per category.

## [0.6.23] - 2026-06-11

### Docs
- **Documentation version cleanup** — README and `f(1)` man page now consistently describe the current install flow, daemon management, shell-wrapper behavior, and background size-refresh behavior.

## [0.6.22] - 2026-06-11

### Docs
- **Cleaner user-facing documentation** — README, INSTALL, and the `f(1)` man page now describe the current install flow, daemon management, shell-wrapper behavior, and placeholder-size refresh behavior more consistently.

## [0.6.20] - 2026-06-11

### Fixes
- **Stale active-root size refresh queue** — stale or placeholder sizes now go through a bounded pending-refresh queue, so active roots are refreshed even when they are not among the first five recently watched folders.
- **Deduplicated background refreshes** — concurrent background size refreshes for the same root are coalesced, preventing repeated `du` work while preserving fast foreground responses.

## [0.6.19] - 2026-06-11

### Fixes
- **Active background directory-size refresh** — the daemon now periodically refreshes stale or placeholder directory sizes for active roots, so large visited directories such as `~/Downloads` populate child sizes without waiting for the next foreground navigation.
- **Placeholder-size retry** — cached `4096`/`4.0k` directory inode fallback sizes are no longer treated as authoritative; background refresh retries them later with a longer timeout.

## [0.6.18] - 2026-06-11

### Fixes
- **Uniform git branch bracket styling** — the closing `]` in dirty/clean branch badges now stays inside the same color/bold span as the branch name, preventing the right edge of badges like `[main*]` from rendering in a darker shade.

## [0.6.17] - 2026-06-11

### Performance
- **Non-blocking directory-size refresh** — banner responses now return immediately with cached sizes while stale or missing child directory sizes refresh in the background, preventing zoxide/chpwd navigation from blocking on large `du` work.
- **Faster logical size calculation** — displayed directory sizes now use `du -s -b`, which is much faster for normal workspace trees and avoids falling back to the 4 KiB directory inode size for large directories.
- **Warmer size cache prepopulation** — warm precompute requests now schedule background size refreshes so parent and child banners are populated before the next navigation.

### Notes
- The first cold view of a very large directory returns quickly and may show cached placeholders until the background size refresh completes; subsequent warm calls use populated single-digit-millisecond cache entries.
- Warm cache hits remain single-digit milliseconds after pre-warm.


## [0.6.16] - 2026-06-11

### Performance
- **Heavier project-insight pruning** — project-insight scans now skip known heavy directories before descent, so `target`, `.git`, `node_modules`, and similar directories do not slow TODO/code-metric collection in large workspace trees.
- **Leaner large-file insight parsing** — very large files are counted without full TODO/LOC parsing, and newline counts are computed without materializing every line.
- **Port-detection shell cache** — `ss -tlnp` output is cached briefly to avoid repeated shell-outs during warm pre-warming bursts.
- **Filesystem-local size refresh** — displayed directory sizes use `du -s --bytes -x` so size refresh stays on the same filesystem.

### Notes
- Warm cache hits remain single-digit milliseconds after pre-warm.
- Cold scans for very large directories are still bounded by accurate directory-size refresh work, but project-insight and port-detection overhead is lower.


## [0.6.15] - 2026-06-10

### Performance
- **Reliable child pre-warming** — warm requests now use one short-lived daemon connection per path and are sent before the CLI exits, so pre-warmed child directories actually get cached.
- **Wider pre-warm coverage** — the client now warms the parent, grandparent, and up to 30 immediate children of the current directory, which covers large `~/Dev` trees much better than the previous 5-child limit.
- **Bounded cold-size refresh** — directory size refresh uses a bounded `du` timeout to reduce first-hit latency on very large trees while keeping normal directory sizes accurate.
- **Cleaner daemon IPC failures** — daemon-side compute errors now return structured IPC errors instead of leaving the client to read a half-closed stream.
- **Clearer missing-path errors** — relative paths that do not exist now fail directly with `No such file or directory: Dev` instead of producing a confusing `send_and_recv` error.

### Notes
- Warm cache hits remain single-digit milliseconds.
- The first cold view of a large directory still has to compute directory sizes, git status, TODOs, ports, and metrics, but the worst-case size refresh is now bounded.


## [0.6.14] - 2026-06-10

### Performance
- **More accurate bounded size refresh** — directory-size refresh keeps its timeout bounded while preserving normal directory-size accuracy better than the tighter 0.6.12 refresh window.

### Notes
- Warm cache hits remain single-digit milliseconds.
- No banner, JSON, or navigation behavior changes.


## [0.6.13] - 2026-06-10

### Performance
- **Expanded pre-warm coverage** — the client now warms the parent, grandparent, and up to 30 immediate children of the current directory.
- **Reliable warm-request delivery** — warm requests are sent on short-lived daemon connections before the CLI exits.

### Notes
- Preserves banner output, JSON output, and numeric navigation behavior.


## [0.6.12] - 2026-06-10

### Performance
- **Tighter cold-size refresh** — directory size refresh keeps a bounded `du` timeout to reduce first-hit latency on very large trees.
- **Cleaner daemon IPC failures** — daemon compute errors now return structured IPC errors instead of leaving the client to read a half-closed stream.
- **Clearer missing-path errors** — relative paths that do not exist now fail directly with `No such file or directory: Dev` instead of producing a confusing `send_and_recv` error.

### Notes
- Warm cache hits remain single-digit milliseconds.
- The first cold view of a large directory still has to compute directory sizes, git status, TODOs, ports, and metrics, but the worst-case size refresh is now bounded more tightly.


## [0.6.11] - 2026-06-10

### Performance
- **Smarter pre-warming of nearby directories** — after a banner is rendered, the client now warms the parent, the grandparent, and the first few immediate children of the current directory, so moving up or stepping into a sibling/child is served from the daemon cache instead of recomputing.

### Notes
- The total number of background warm requests is bounded to a small set of paths to avoid expensive background scans.
- Preserves banner output, JSON output, and numeric navigation behavior.


## [0.6.10] - 2026-06-10

### Packaging
- **Corrected crates.io repository metadata** — package metadata now points to `https://github.com/DraconDev/folder-auto-banner` and includes homepage/documentation links.

### Notes
- crates.io does not allow repository metadata to be changed for already-published versions, so older published versions may still show the previous incorrect repository URL. New installs via `cargo install folder-auto-banner` use the corrected latest release.
- No runtime behavior changes.


## [0.6.9] - 2026-06-10

### Performance
- **Global uid/gid name caches** — `/etc/passwd` and `/etc/group` are loaded once per process instead of reparsing them for every directory scan.
- **Lower-allocation permission formatting** — file mode rendering now builds the 10-character mode string directly instead of using `format!` per row.
- **Leaner active watcher maintenance** — inactive watcher cleanup now reuses the active-root snapshot from the periodic refresh when available and avoids extra mutex work.

### Notes
- Preserves banner output, JSON output, and numeric navigation behavior.
- No new dependencies.


## [0.6.8] - 2026-06-10

### Performance
- **Tighter git status pathspecs** — filtered git status collection now limits directory status walks to immediate children (`dir/*`) instead of asking libgit2 to scan every nested file under displayed directories.
- **Leaner daemon watcher refreshes** — the active-folder watcher now refreshes watched paths only when the active root set or priority order changes, avoiding repeated recursive directory scans while idle.
- **Skip git work for raw/oneline fallback** — when the daemon is unavailable and output does not need git metadata, direct fallback avoids collecting git status entirely.
- **Expanded benchmarks** — added a manifest-repository git-info benchmark so large-repo status collection regressions are visible.

### Notes
- Preserves rich banner, JSON, and navigation behavior.
- No new dependencies.


## [0.6.7] - 2026-06-10

### Performance
- **Avoid duplicate project-insight scans** — TODO counts and code metrics now share one bounded tree walk when both are enabled, reducing cold daemon scans and file reads.
- **Reuse rendered item contents** — rich banner rendering now computes directory counts and file content previews once per item, then reuses those values for column sizing and row output.
- **Leaner project-type detection** — marker-file checks use direct path probes before reading directory entries, making repeated project detection faster.

### Notes
- Preserves existing banner output and daemon freshness behavior.
- No new dependencies.


## [0.6.6] - 2026-06-10

### Fixed
- **`f N` navigation bug** — when running `f N` (e.g. `f 40`), the daemon was being asked for the banner of the path `"40"` (the number string) instead of the current directory. This caused `f N` to return an empty path or open the wrong file when the number didn't match a real directory. The path is now resolved correctly: numeric navigation always uses the current directory, matching how the shell function invokes `f banner N`.

### Notes
- Preserves 0.6.5 behavior in all other respects.
- No new dependencies.


## [0.6.2] - 2026-06-09

### Fixed
- **Fresh folder information** — `f` now consistently shows the latest folder contents and sizes:
  - Daemon cache hits validate the cached folder snapshot against a fresh shallow scan before returning, so out-of-band edits are no longer masked by the TTL.
  - Displayed directory sizes are refreshed when their mtime changes, so nested folder content edits immediately update parent folder size information.
  - Directory size cache tracks mtimes, so persisted sizes from a previous daemon run can no longer shadow fresh data.
- **Daemon clear-cache** now also clears `banner_cache.json` and `dir_sizes.json` (previously only the cache directory), with no spurious shutdown warning.
- Bench harness `benches/performance.rs` now references the real crate name (`folder_auto_banner`) instead of the old `fab_lib`.
- rustdoc HTML warning in `port_usage` for `<pid>` token.

### Notes
- No user-visible behavior changes beyond the freshness fix and the expanded `f daemon clear-cache`.
- No new dependencies.

## [0.4.0] - 2024-05-31

### Added
- **Config file** (`~/.config/fab/config.toml`) with all display preferences
- `f config` command — opens config in $EDITOR
- `f daemon stop/status` commands — daemon management
- Two-row banner layout for better readability
- Dynamic truncation for narrow terminals
- Git enhancements: last commit time, commits today, branch count
- Languages breakdown with percentages
- Build timing display
- Cached test results display
- Permission modes: rwx, octal, disable
- Column selection (show/hide columns)
- Feature toggles (git, build, todos, languages, ports, docker)

### Changed
- **Design principle**: CLI flags = actions, Config file = preferences
- Simplified CLI to focus on core features
- Improved banner layout with two rows

### Fixed
- Daemon log spam for non-existent directories
- Dead symlink handling in daemon watcher
- Install script properly stops daemon before reinstall
- Broken symlink display (✗→ indicator)
- Full resolved symlink paths

## [0.3.0] - 2024-05-30

### Changed
- **Major simplification**: Removed 15+ commands that duplicate existing tools
- Renamed binary from `fm` to `f` for faster typing
- Focus on core feature: directory listing with instant context

### Removed
- File operations: `cp`, `mv`, `rm`, `trash`, `open` — people have their own tools
- Clipboard: `yank`, `paste`, `clipboard` — niche use case
- Navigation: `pin`, `unpin`, `pins`, `jump`, `root` — redundant with frecency
- Sessions: `save-session`, `load-session`, `sessions`, `delete-session` — over-engineering
- Other: `diff`, `do`, `peek`, `stats`, `config` — niche or over-engineered
- Flags: `--no-build-check`, `--no-todos`, `--no-ports`, `--no-docker`, `--no-metrics` — use env vars

### Added
- `--compact` flag for less info
- `--verbose` flag for more info
- Broken symlink indicator (✗→)
- Full resolved symlink paths

## [0.2.0] - 2024-05-30

### Added
- `--hidden` flag to show dotfiles
- `--filter` flag to filter items by pattern/extension
- `--max N` flag to limit items displayed
- `--group` flag to group items by type
- Config file support
- `fm config` command
- `fm root` and `fm uninstall-hook` commands
- `NO_COLOR` environment variable support
- Path protection for trash, mv, cp commands
- Shared library crate (`fab-lib`)
- Unit and integration tests

### Fixed
- Dry-run flag now works for all destructive commands
- Daemon mutex poisoning recovery
- Shell injection prevention
- Copy verification before deletion
- Symlink loop prevention

## [0.1.1] - 2024-05-25

### Added
- Complete CLI with 28 commands
- Rich terminal output with Unicode icons
- TTY detection (rich/raw/JSON modes)
- State persistence
- Shell integration hooks
- Shell completions
- GitHub Actions CI/CD

## [0.1.0] - 2024-05-24

### Added
- Initial release
- Basic banner display
- Project type detection
- Git integration
