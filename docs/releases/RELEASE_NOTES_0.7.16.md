# 0.7.16 — the banner no longer hangs on a directory containing a FIFO

`f <dir>` could hang forever, with no output and no error, on any directory
holding a FIFO. The directory listing still appeared only after minutes; more
often the process just sat there. The prompt never came back.

## What was happening

Rendering probes each listed item for the contents column (line counts, image
dimensions, archive entries). That probe opens the file. For a FIFO, opening
read-only waits for a writer that may never exist, so the process blocked
inside `openat` — kernel wait channel `wait_for_partner` — and the prompt was
gone.

Two things made it worse than an exotic edge case:

- The probe was reached for every entry that was not a directory, so a FIFO
  qualified.
- Nothing in the path checked that the entry was a regular file, and the open
  itself had no `O_NONBLOCK`.

## Why it surfaced now

It was always there, but this release cycle is when the banner started
*displaying* those entries and when CI began listing directories that contain
one. A GitHub runner keeps `clr-debug-pipe-*` FIFOs in `/tmp` from its own .NET
diagnostics tooling. The test suite scans `/tmp` and `$HOME`, so the suite
stalled in `tests/alias_test.rs` for hours with no output — which is also why
the v0.7.15 release never published: its `Run tests` step hung the same way
and `Create Release` waited on it forever.

## The fix

- `get_file_contents` refuses anything that is not a regular file, so the probe
  is never reached for a FIFO, socket, or device node.
- The text probe opens with `O_NONBLOCK` and re-checks the descriptor, so a
  path that turns into a FIFO between the scan's `stat` and the open returns
  an error instead of blocking.
- `read_cache` got the same regular-file guard. A cache path left as a FIFO by
  something else would otherwise hang every later invocation for that path.

Regular files are unaffected: `O_NONBLOCK` is a no-op for them, and line counts
are unchanged.

## Guard rails

- CI jobs and the release jobs have `timeout-minutes`, so a hang fails with
  logs instead of stalling for six hours.
- The alias tests bound every `f` invocation and, on a timeout, dump the hung
  process tree with each thread's wait channel, syscall, and the daemon log.
  That is what identified this bug; the same harness now reports the next one
  in one run instead of five hours.
- With `FAB_PROFILE=1` the daemon logs to `fabd.log` beside its socket instead
  of `/dev/null`, so a wedged daemon is no longer invisible.

## Release packaging

The publish step uploaded every target's binary under the same asset name
(`f`, `fabd`), so architectures overwrote each other: v0.7.16 shipped an
x86_64 `f` next to an aarch64 `fabd`, which cannot work together. Assets are now
named `f-<arch>-linux` / `fabd-<arch>-linux`, a missing target now fails the run
instead of clobbering an existing asset, and the release body comes from the
version's notes file. v0.7.16's assets were repaired in place.

## Tests

260 passing, 0 failing. Three new tests: the FIFO cache read, the FIFO content
probe, and an end-to-end render of a directory containing FIFOs. Each fails
fast instead of hanging when its guard is removed.

Upstream: 0.7.15's `cd`-latency work (immediate listing, enrichment behind it)
is included here; 0.7.15 itself was tagged but never published.
