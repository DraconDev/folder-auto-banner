# folder-auto-banner 0.6.31

## Summary
Bug fix for lazy flags. In `0.6.29`/`0.6.30`, lazy flags (`f t`,
`f S`, etc.) only worked for flags that were defined in **both** the
top-level CLI and the `Banner` subcommand. Flags that were only in
the top-level CLI (like `-a`, `-r`, `-L`, `-R`, `-D`) would fail
with "unexpected argument" when used as a lazy flag with a path.
This release adds those missing short flags to the `Banner`
subcommand so lazy flags work consistently.

## What changed

### Flags added to the `Banner` subcommand

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

### Flag conflict fix

`raw` in the `Banner` subcommand had `#[arg(short, long)]` which
auto-assigned `-r`, conflicting with `reverse` which has
`#[arg(short = 'r', long = "reverse")]`. Changed `raw` to
`#[arg(long = "raw")]` (no short flag), matching the top-level
CLI. `raw` is now only available as `--raw`.

### Value-taking flags work with lazy flags

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

## Why this fix

In `0.6.29` we added lazy flags (single-character args without the
leading dash, e.g. `f t` ≡ `f -t`). The routing logic in
`src/main.rs` expands the lazy flag and routes to the `Banner`
subcommand. But some short flags were only defined on the top-level
CLI, not on the `Banner` subcommand. When the user typed `f a
Downloads` (lazy hidden + path), it would route to `f banner -a
Downloads`, and the `Banner` subcommand would reject `-a` because
it didn't have that flag.

This release adds the missing short flags to the `Banner` subcommand
so lazy flags work for all short flags, not just the ones that
happened to be defined in both places.

## Validation

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (140 tests, unchanged count)
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`
- Manual verification: `f a` ≡ `f -a`, `f r` ≡ `f -r`, `f L 2` ≡
  `f -L 2`, `f R --max 5` ≡ `f -R --max 5`, `f D` ≡ `f -D`.

## Preserved behavior

- All existing explicit flags work unchanged.
- All existing lazy flags work unchanged.
- `f 1` still navigates to item 1 (number precedence).
- `f t`, `f S`, `f X`, `f G`, `f c`, `f v`, `f o` (oneline) all
  work unchanged.
- `--raw` still works (now the only way to use raw output from
  the banner subcommand).
