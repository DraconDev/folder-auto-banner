# Release Notes — f 0.6.36

**Date**: 2026-06-15
**Type**: Extended test coverage

## Summary

Added 49 new tests for the lazy flag system, bringing the total to 280 tests with 100% pass rate. Added a standalone test harness script and a new `proptest` dev-dependency for property-based testing.

## What's New

### 10 new property-based tests

Using `proptest` (added as dev-dependency), each running 1000 random cases:
- `prop_resolve_lazy_flag_is_total` — every char returns valid result
- `prop_expand_lazy_flags_valid_chains` — 1000 random alpha strings
- `prop_is_explicit_path_dot_prefix` — any `.<x>` is explicit
- `prop_is_explicit_path_slash_prefix` — any `/<x>` is explicit
- `prop_is_explicit_path_tilde_prefix` — any `~<x>` is explicit
- `prop_is_explicit_path_bare_alpha_rejected` — bare words aren't explicit
- `prop_expand_and_resolve_consistent` — both functions agree
- `prop_chain_length_equals_input_length` — expansion preserves length
- `prop_no_panic_on_random_input` — no panics on any string
- `prop_expand_empty_string_returns_none` — empty input rejected

### 39 new integration tests

- **27 edge case tests**: very long chains, all 14 boolean flags, subcommand routing, unicode, empty strings, version flag, etc.
- **4 cross-platform path tests**: `./`, `/`, `..`, `$HOME`
- **8 daemon interaction tests**: cold start, warm, repeated invocation consistency

### New standalone test harness: `scripts/test_lazy_flags.sh`

- Runs 37 lazy flag examples
- Verifies exit codes match between lazy and explicit forms
- Can be run independently of `cargo test`
- Exits with non-zero on any failure
- Handles value-taking flags correctly (consumes next arg as value)

## Test Metrics

| Metric | 0.6.35 | 0.6.36 | Change |
|--------|--------|--------|--------|
| Unit tests | 43 | 53 | +10 |
| Integration tests (active) | 29 | 29 | 0 |
| Lazy flags tests | 55 | 94 | +39 |
| **Total** | **231** | **280** | **+49** |
| Pass rate | 100% | 100% | — |

## Known Limitations Discovered

The extended testing revealed several limitations of the lazy flag system:
- `f --debug <lazy>` doesn't work (clap sees next arg as subcommand)
- `f banner <lazy>` doesn't work (routing bypasses lazy expansion)
- `f help <lazy>` doesn't work (same reason)
- Mixing explicit (`-c`) and lazy (`t`) flags doesn't work

These are documented in the test file with notes explaining the workarounds.

## New Dev-Dependency

- `proptest = "1.4"` — for property-based testing

## Validation

- ✅ `cargo fmt --all -- --check`
- ✅ `cargo check --all-targets`
- ✅ `cargo clippy --all-targets --all-features -- -D warnings`
- ✅ `cargo test --all-features --no-fail-fast -- --test-threads=1` — 280 pass
- ✅ `cargo doc --no-deps`
- ✅ `cargo build --release --locked`
- ✅ `cargo publish --dry-run --locked`
- ✅ `scripts/test_lazy_flags.sh` — 37/37 pass

## Files Changed

- `src/main.rs` — added 10 property-based tests
- `tests/lazy_flags_test.rs` — added 39 edge case + cross-platform + daemon tests
- `scripts/test_lazy_flags.sh` — NEW standalone test harness
- `Cargo.toml` — version bump to 0.6.36, added `proptest` dev-dependency
- `f.1` — version bump to 0.6.36
- `CHANGELOG.md` — 0.6.36 entry
- `LAZY_FLAGS_TESTING.md` — updated test count
- `RELEASE_NOTES_0.6.36.md` — this file
