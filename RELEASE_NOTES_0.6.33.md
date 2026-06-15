# folder-auto-banner 0.6.33

## Summary
**Value-taking flags in chained lazy flags** — `f mL 10 2` is now
equivalent to `f -m 10 -L 2` (max=10, level=2). The values are
consumed in chain order from the args following the chain. This
fixes the limitation in `0.6.32` where only boolean flags could be
chained.

## What changed

### New `VALUE_TAKING_FLAGS` constant

In `src/main.rs`, a new constant lists the 3 single-character flags
that take values:

```rust
const VALUE_TAKING_FLAGS: &[char] = &[
    'm', // --max <MAX> (usize)
    'f', // --filter <PATTERN> (String)
    'L', // --level <LEVEL> (usize)
];
```

### Smart chain expansion

When a chain contains value-taking flags, the expansion interleaves
the flags with their values in the correct order. For example:

- `f tSc` → `-t -S -c` (all boolean, no values to consume)
- `f mL 10 2` → `-m 10 -L 2` (values consumed in chain order)
- `f mLf 10 2 txt` → `-m 10 -L 2 -f txt` (three value-taking flags)

### Why interleaving is needed

clap cannot handle `-m -L 10 2` (value-taking flag immediately
followed by another flag confuses clap). The expansion produces
`-m 10 -L 2` which clap handles correctly.

### Error handling

If a value-taking flag doesn't have a value, clap reports a clear
error. For example, `f mL 10` (missing value for L) will fail with:
```
error: invalid value '/home/dracon/Downloads' for '--level <LEVEL>': invalid digit found in string
```

## Examples

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

## Validation

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (139 tests)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`
- Manual verification: `f mL 10 2` ≡ `f -m 10 -L 2`,
  `f tSm 10` ≡ `f -t -S -m 10`, `f mLf 10 2 txt` works correctly.

## Preserved behavior

- Single-char lazy flags still work: `f t`, `f S`, `f a`, etc.
- Boolean chains still work: `f trc`, `f tSc`, etc.
- Numbers still navigate: `f 1`, `f 2`, etc.
- Subcommands still work: `f banner`, `f env`, etc.
- Explicit paths still work: `f ./path`, `f /abs/path`, `f ~/path`.
- All explicit flags still work: `f -t`, `f -S`, `f -m 10`, etc.
