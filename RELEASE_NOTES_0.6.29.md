# folder-auto-banner 0.6.29

## Summary
Adds **lazy flags** — `f t` is now equivalent to `f -t`, `f S` to
`f -S`, etc. Single-character short flags can be used without the
leading dash. **No fallback**: `f t` ALWAYS means `-t` (sort by
time). To show a banner for a file or directory named `t`, use
`./t` or an absolute path.

## What changed

### Routing logic in `src/main.rs`

Before:
```rust
if number → navigate
else if known subcommand → use it
else → treat as path
```

After:
```rust
if number → navigate              // f 1, f 2, f 3 ...
else if known subcommand → use it // f banner, f env, f install ...
else if single-char lazy flag     // f t → f -t, f S → f -S ...
else → treat as path              // f Downloads, f /abs/path
```

### The 17 lazy flags

| Lazy | Equivalent | Description |
|------|------------|-------------|
| `f t` | `f -t` | Sort by time modified |
| `f S` | `f -S` | Sort by size |
| `f X` | `f -X` | Sort by extension |
| `f G` | `f -G` | Sort by git status |
| `f r` | `f -r` | Reverse sort |
| `f a` | `f -a` | Show hidden files |
| `f c` | `f -c` | Compact output |
| `f v` | `f -v` | Verbose output |
| `f R` | `f -R` | Recurse into subdirectories |
| `f D` | `f -D` | List only directories |
| `f 1` | `f -1` | One file per line |
| `f m` | `f -m` | Maximum items |
| `f L` | `f -L` | Limit recursion depth |
| `f f` | `f -f` | Filter by pattern |
| `f U` | `f -U` | No sort |
| `f e` | `f -e` | Force open in editor |
| `f x` | `f -x` | Force run file |

## Why no fallback

The user requested "no fallback at all, just lazy flags". This means
`f t` ALWAYS means sort by time, never a path. Rationale:
- **No ambiguity** — the user never has to think "is this a flag or
  a path?"
- **Predictable** — same input always produces same output
- **Paths still work** — use `./t` or `/full/path/to/t` for files
  named `t`

## Precedence rules

1. **Numbers** (`f 1`, `f 2`, etc.) navigate to item N. Number
   navigation is a core feature, so numbers take precedence over
   lazy flags. `f 1` navigates, not enables oneline.
2. **Known subcommands** (`f banner`, `f env`, `f install`,
   `f config`, `f daemon`, `f help`) work as before.
3. **Lazy flags** — single-char args matching a known flag are
   expanded to `-X`.
4. **Paths** — everything else (multi-char, or single-char not in
   the lazy-flag list) is treated as a path.

## Validation

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

## Preserved behavior

- `f N` (number navigation) unchanged.
- `f banner <path>` unchanged.
- `f env`, `f install`, `f config`, `f daemon`, `f help` unchanged.
- All long flags (`--timesort`, `--sizesort`, etc.) unchanged.
- All combined flags (e.g. `f -tc`) unchanged.
- Path with single-char name still works when explicitly prefixed
  (e.g. `./t` or `/abs/t`).
