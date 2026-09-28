# Release notes — v0.7.15 (2026-09-28)

Patch release. Theme: **`cd` into a large directory no longer takes
seconds.** The prompt hook was spending 1.5–8 seconds per navigation on
directories this machine actually has — `~/Dev` is 330 GB across 1.19M
files, and `~/Dev/dracon-platform` is a 26,633-commit repo with a 50 GB
`.git`.

## Results

Fully cold (all caches wiped, fresh daemon), then warm:

| Path | cold before | cold after | warm after |
|---|---:|---:|---:|
| `~/Dev` | 1655–2966 ms | **61–292 ms** | 12–28 ms |
| `~/Dev/dracon-platform` | **8067 ms** | **643 ms** | 16–31 ms |
| `~/Dev/folder-auto-banner` | 1677 ms | **227 ms** | 10–26 ms |
| `~` | 1655–2966 ms | **227 ms** | 15–46 ms |
| `~/Downloads` | 556 ms | **61 ms** | 12–26 ms |

Background cost fell too: 5 concurrent `du` processes running at all times
→ **0**, and daemon CPU 5620 ms → 500 ms per 15 s wall (**11×**). Sustained
I/O pressure dropped from ~17–23% to ~12%.

## What's new

**The listing answers immediately; enrichment lands behind it.**
`compute_banner_data` gained a `ComputeMode`. A cache miss now runs
`ComputeMode::Fast` — the file listing plus a git status bounded to 400 ms —
and responds. The expensive parts (recursive TODO/code-metrics walk, port
detection, and the nine decorative git collectors) run in a background
`ComputeMode::Full` pass that replaces the cache entry and rewrites the
on-disk cache. So the **first** visit shows you the listing, and the **next**
one shows everything. `CacheEntry::enriched` tracks which pass produced an
entry so a banner is never left un-enriched.

**Directory sizes are measured, not guessed.** `du -s -b ~/Dev` takes
**106 s**; `~/Dev/dracon-platform` takes **56 s**. Every call hit the timeout,
returned the 4 KiB directory-inode placeholder, and — since a placeholder is
not authoritative — was never cached, so the daemon re-ran it forever and
every directory in the `~/Dev` banner rendered as `4.0k`. Sizes now use a
bounded breadth-first walk capped by file count, directory count, and wall
clock. Small trees stay exact; large ones return a true lower bound shown
with a `≥` prefix (`DirEntry::size_is_estimate`).

**`f daemon warm [PATH...]` is now reachable.** The daemon already handled a
`Warm` request and the client already had `warm_paths()` — no CLI subcommand
reached them, so "the daemon already knows this" was dead code. Use it to
pre-compute your usual directories.

**Daemon-side profiling.** `FAB_PROFILE=1` now reports from the daemon too:
per-phase scan timing, git timing per compute, total compute time tagged with
its `ComputeMode`, and the request trace. Previously only the client was
instrumented, which is why a 1.5 s insight walk and an 11-subprocess git
fan-out were both invisible until the banner stopped feeling instant.

## Fixes

- **`git status` could block the prompt for up to 10 s** on a large
  repository (`GIT_COMMAND_TIMEOUT`). The fast pass now caps it at 400 ms; a
  huge repository shows its listing without git badges, and the background
  pass fills the real statuses in. Nine of the eleven `git` subprocesses were
  header decoration and are now enrichment-only.
- **Warming was making the real request slower.** One `f` invocation in a
  container directory sent **32** warm requests (parent, grandparent, up to
  30 children); visiting `~/Dev` launched 13 concurrent ~1 s tree walks racing
  the single banner you asked for, turning a 50 ms request into 1.4 s. Now
  capped at 4 client-side and 2 concurrent daemon-side. A dropped warm only
  defers work to the next visit, so it is always safe.
- **The client "fast path" was slower than the path it avoided.** Validating
  the disk cache walked 8192 entries at depth 8 — **11,430 syscalls and
  22–76 ms** — before a Unix-socket round trip costing 1–10 ms. On a
  container like `~/Dev` some project is always being written, so the cache
  was permanently stale and the fast path never engaged. The walk is now
  depth-1; deeper changes remain covered by the daemon's inotify watcher.
  Syscalls for `f banner ~`: **11,430 → 1,941**.

## Notes for users

- **Behaviour change worth knowing:** the *first* visit to a directory now
  shows the listing with sizes, without TODO counts / language mix / ports
  until the background pass finishes a moment later. The next visit is
  complete and fast. This is the trade that removes the multi-second stall;
  every field still appears.
- Directory sizes on very large trees are now `≥` lower bounds instead of
  `4.0k` placeholders. On `~/Dev/dracon-platform` that means `≥1.4G` against
  a real 222 GB — loose, but an honest observation rather than a fabricated
  total. Small trees remain exact and unmarked.
- `DirEntry` gained a serde-defaulted field. Existing on-disk caches load
  fine; run `f daemon clear-cache` if you want a clean slate.
- The `≥` marker is not persisted: after a daemon restart, a previously
  sampled size shows without its marker until the next refresh relabels it.
  The number is still a correct lower bound.
- Unrelated and untouched: if your `~/.zshrc` still carries the stale
  comment saying the auto-banner hook is "Disabled because `f banner` scans
  the current directory synchronously", that hook is actually *enabled* at
  the bottom of the file, and the comment now describes a problem that no
  longer exists. Worth deleting.

## Tests

253 passing, 4 failing (baseline at v0.7.14: 244 passing, 5 failing). The 4
failures are the same pre-existing alias-routing cases, unchanged by this
work and verified against a clean checkout of the previous tag; the fifth
baseline failure, `test_daemon_new`, only fails when a live `fabd` holds the
socket. +9 new tests, including `test_max_descendant_mtime_stops_at_one_level`,
which pins the depth-1 freshness boundary so re-deepening the walk fails
loudly, and `test_sampled_sizes_are_cached_and_marked`, which guards the
uncacheable-mtime regression that caused the `du` storm.

`cargo clippy --all-targets`: no issues. No lint suppressions were added.



### macOS support (newly buildable)

The daemon and `f` now compile cleanly for `aarch64-apple-darwin` as well as
both Linux targets. There is one behavioral caveat: `inotify` is a Linux-only
crate, so the filesystem watcher that backs the daemon's cache-invalidation
fast path does not start on macOS. The cache still invalidates on the mtime
checks in `cache_entry_is_fresh` and `cached_dir_size_is_fresh`, so staleness
is bounded by the 5-minute banner TTL rather than by filesystem events.
This is gated at the dep and call-site level (`#[cfg(target_os = "linux")]`
on the `inotify` dependency, the watcher constants, the `watch_loop` thread,
and its helpers). On Linux there is no change — the watcher still fires
exactly as before.

`libc` is promoted to a direct `unix`-gated dependency (it was transitive
via `inotify`); the call sites that use it (`ioctl(TIOCGWINSZ)` for terminal
size) are already `#[cfg(unix)]`.

The release matrix builds the two Linux targets:
`x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`.

The daemon also compiles cleanly for `x86_64-apple-darwin` and
`aarch64-apple-darwin` (verified locally via `cargo build --target …`); it is
not in the release matrix because this repo's CI is currently routing the
apple matrix entry to `ubuntu` rather than `macos-latest`, so the link step
fails with gcc unable to parse Apple's `-framework` / `-arch` flags. The
matrix entry is removed for this release; restoring it requires a working
macOS runner (or an osxcross setup step) and is a CI-inffrastructure fix,
not a code one. On Linux the inotify watcher still fires exactly as before,
and on macOS it is the same code path minus the watcher thread.
