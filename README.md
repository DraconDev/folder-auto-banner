# cfm — Contextual File Manager

> **Type `fm`. Get context.**

An ephemeral, zero-hostage intelligence layer for the shell. Print once, exit immediately. No TUI. No daemon. <5ms latency.

## Quick Start

```bash
# Build
cargo build --release
./target/release/fm

# Show in JSON (for scripting)
./target/release/fm banner --json

# Show raw paths (for piping)
./target/release/fm banner --raw

# Auto-detect project type and show aliases
./target/release/fm env
```

## The Banner

When run in a terminal, `fm` prints a rich context dashboard:

```text
┌──┐
│ 📂 ~/Dev/cfm [main ✚1 ?1]
│ 🦀 Rust │ 106 KB │ chore: update banner
│ ────────────────────────────────────────────────────
+----+------------------+-----------------+-------------------+
|    | Name             | Type            | Size              |
+=============================================================+
| 📂 | src              | 8 item(s)       | -                 |
+----+------------------+-----------------+-------------------+
│ ... and 5 more items.
└────────────────────────────────────────────────────────────────
```

## Commands

| Command | Description |
|---------|-------------|
| `fm` | Show directory banner |
| `fm banner --json` | JSON output |
| `fm banner --raw` | Raw paths for piping |
| `fm env` | Output project aliases |
| `fm mv/cp/rm/trash` | File operations |
| `fm yank/paste` | Clipboard operations |
| `fm pin/jump/pins` | Directory bookmarks |
| `fm stats` | Directory statistics |
| `fm diff` | Compare directories |
| `fm do` | Act on piped paths |
| `fm save-session/load-session` | Workspace management |
| `fm completion <shell>` | Generate completions |

## Shell Integration

Add to `~/.zshrc`:

```bash
_cfm_hook() {
    command fm banner "$PWD"
    eval "$(command fm env "$PWD")"
}
autoload -U add-zsh-hook
add-zsh-hook chpwd _cfm_hook
```

## Philosophy

- **Ephemeral**: Wake up, print, exit. No background processes.
- **Composable**: Works with pipes
- **Fast**: <5ms startup, exits immediately

## Building

```bash
cargo build --release
cargo install --path .
```

## Testing

```bash
cargo test    # 10 tests pass
cargo clippy  # 0 warnings
```

## License

MIT