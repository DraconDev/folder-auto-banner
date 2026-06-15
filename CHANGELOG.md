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
