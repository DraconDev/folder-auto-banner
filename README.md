# cfm — Contextual File Manager

> **Type `fm`. Get context.**

An ephemeral, zero-hostage intelligence layer for the shell. Print once, exit immediately. No TUI. No daemon. <5ms latency.

## Quick Start

```bash
# Build
cargo build --release

# Show directory context (rich output in terminal)
./target/release/fm

# Show in JSON (for scripting)
./target/release/fm --json

# Show raw paths (for piping)
./target/release/fm --raw

# Auto-detect project type and show aliases
./target/release/fm env
```

## The Banner

When run in a terminal, `fm` prints a rich context dashboard:

```text
┌──┐
│ 📂 ~/Dev/cli-file-manager [main ✚1 ?1]
│ 🦀 Rust │ 103 KB │ chore: update banner
│ ----------------------------------------------------------------------------
+----+------------------+-----------------+-------------------+
|    | Name             | Type            | Size              |
+=============================================================+
| 📂 | src              | 8 item(s)       | -                 |
|----+------------------+-----------------+-------------------|
| 📄 | Cargo.toml       | Config          | 1.2 KiB           |
+----+------------------+-----------------+-------------------+
│ ... and 5 more items. (Use 'ls' to see all)
└────────────────────────────────────────────────────────────────
```

Shows:
- **Project type** (🦀 Rust, 📦 Node, 🐍 Python, etc.)
- **Git status** (branch, dirty state, untracked files)
- **Directory size** and item count
- **Top items** with types and sizes

## Commands

| Command | Description |
|---------|-------------|
| `fm` | Show directory banner |
| `fm banner` | Show banner (same as `fm`) |
| `fm banner --json` | JSON output for scripting |
| `fm banner --raw` | Raw paths for piping |
| `fm env` | Output project aliases |
| `fm install-hook` | Install shell integration |
| `fm pin <name>` | Pin current directory |
| `fm pins` | List all pins |
| `fm stats` | Deep directory stats |
| `fm yank <paths>` | Copy to clipboard |
| `fm paste` | Paste from clipboard |

## Shell Integration

Add to your `~/.zshrc` or `~/.bashrc`:

```bash
# cfm shell integration
_cfm_hook() {
    command fm banner "$PWD"
    eval "$(command fm env "$PWD")"
}
autoload -U add-zsh-hook
add-zsh-hook chpwd _cfm_hook
```

This shows the banner on every `cd` and auto-generates context aliases like `run=cargo run` in Rust projects.

## Philosophy

- **Ephemeral**: Wake up, print, exit. No background processes.
- **Composable**: Works with pipes (`fm | grep ...`)
- **Visible**: Rich output when in terminal, clean text when piped
- **Fast**: <5ms startup, exits immediately

## Building

```bash
cargo build --release
sudo cp target/release/fm ~/.cargo/bin/  # or install via cargo
```

## Requirements

- Rust 1.70+
- Terminal with Unicode support (for icons)

## License

MIT