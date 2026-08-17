# folder-auto-banner 0.6.30

## Summary
**Breaking change**: the oneline short flag is now `-o` instead of
`-1`. The previous `-1` was unreachable as a lazy flag because
`f 1` always navigates to item 1 (number precedence over lazy
flags). Changed to `-o` so `f o` works as a lazy flag.

## What changed

### Oneline short flag: `-1` → `-o`

Before (0.6.29):
- `f -1` → oneline mode
- `f 1` → navigate to item 1 (number precedence wins)
- `f 1` as lazy flag for oneline → **unreachable**

After (0.6.30):
- `f -o` → oneline mode
- `f 1` → navigate to item 1 (unchanged)
- `f o` → oneline mode (lazy flag, now reachable)

### Migration

| Old | New |
|-----|-----|
| `f -1` | `f -o` |
| `f --oneline` | `f --oneline` (unchanged) |
| `f o` | `f o` (now works as lazy oneline) |

## Why this change

In `0.6.29` we added lazy flags (e.g. `f t` ≡ `f -t`). The oneline
flag had short = `'1'`, but the routing logic checks numbers first
(for navigation), so `f 1` always navigates to item 1, never
enables oneline. The `'1'` entry in `LAZY_FLAGS` was dead code.

Changing the short flag to `'o'` makes oneline reachable as a
lazy flag: `f o` ≡ `f -o` ≡ `f --oneline`.

## Validation

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (140 tests, unchanged count — just
  updated test references)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`
- Manual verification: `f o` ≡ `f -o` ≡ `f --oneline`, `f 1`
  still navigates, `f -1` now errors with "unexpected argument".

## Preserved behavior

- `f 1`, `f 2`, ..., `f N` still navigate to item N.
- `f --oneline` still works (long form unchanged).
- All other short and long flags unchanged.
- All lazy flags (`f t`, `f S`, `f c`, `f a`, etc.) unchanged.
