# Release Notes — f 0.6.37

**Date**: 2026-06-15

## Lazy flag value-binding with `:` separator

This release answers the question: "what if we want to give the
argument to the last one?" when using lazy flag chains with
multiple value-taking flags.

### The problem

`f mLf 10` was ambiguous. With 1 value and 3 value-taking flags
(`m`, `L`, `f`), the value would bind to `-m` (the first flag,
chain order). But the user might have intended `-f` (the last
flag, hence the question).

### The solution

A `:` immediately after a value-taking flag in the chain marks
it as an **explicit value-binding target**:

```text
f mLf: 10         →  -f 10
f m:L:f: 10 2 txt →  -m 10 -L 2 -f txt
f m:L: 10 2       →  -m 10 -L 2 -f
f trcL: 5         →  -t -r -c -L 5
f ml: 10          →  -m -L 10
```

Non-target value-taking flags that come before the last target
in the chain are omitted from the output (clap requires a value
for value-taking flags, so they can't be pushed without one).

### Why this design

- **Solves the user's question** directly: `f mLf: 10` binds `10`
  to `-f`, the last value-taking flag.
- **Backward compatible**: existing chains without `:` work
  exactly as before. The 0.6.36 test suite passes unchanged.
- **Explicit binding**: the `:` is a clear visual marker — no
  ambiguity about which flag gets the value.
- **Composable**: any subset of flags can be marked, the rest
  use defaults.
- **No new chars added to `LAZY_FLAGS`**: `:` is parsed in the
  expansion function as a separator, not a flag.
- **Preserves the no-fallback rule**: bare words are still
  lazy-flag chains, never paths.
- **Preserves case-insensitive aliases**: `l`→`L`, etc.

### Improved error messages

`f m` (value-taking flag with no value, no `:` marker) now
produces a clearer error:

```text
error: flag '-m' in chain 'm' requires a value, but no more
arguments were provided. Use 'm:' to mark which flag should
receive the value, or supply a value after the chain.
```

### Backward compatibility

- `f m 10` → `-m 10` (unchanged)
- `f t` → `-t` (unchanged)
- `f trc` → `-t -r -c` (unchanged)
- `f mL 10 2` → `-m 10 -L 2` (unchanged)
- `f mLf 10 2 txt` → `-m 10 -L 2 -f txt` (unchanged)
- `f m 10 20` → `-m 10`, `20` is a path (unchanged)
- `f -t`, `f --filter txt`, etc. (unchanged)

### Test coverage

- 12 new integration tests in `tests/lazy_flags_test.rs`
- 9 new unit tests in `src/main.rs`
- 2 new property-based tests (1000 cases each)
- Total: 303 tests, 100% pass rate (was 280 in 0.6.36)

### New documentation

- `LAZY_FLAGS_VALUE_BINDING.md` — design doc with 5 alternatives,
  chosen design, parsing rules, examples, and trade-offs.

### Files changed

- `src/main.rs` — added `expand_lazy_flags_with_binding`,
  updated main loop to use binding rules, added unit tests.
- `tests/lazy_flags_test.rs` — added 12 new tests, updated
  2 error-message tests.
- `Cargo.toml` — version bump to 0.6.37.
- `f.1` — version bump to 0.6.37.
- `CHANGELOG.md` — entry for 0.6.37.
- `LAZY_FLAGS_VALUE_BINDING.md` — new design document.

### Installation

```bash
cargo install folder-auto-banner
```

Or update an existing install:

```bash
cargo install folder-auto-banner --force
```

The daemon (`fabd`) does not need to be restarted for this change
(the binding logic is entirely client-side).
