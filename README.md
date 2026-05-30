# cfm — Contextual File Manager

An ephemeral, zero-hostage intelligence layer for the shell.

## Features

- **Auto banner** on every `cd` (zsh / bash)
- **Rich output** with Unicode icons, Git status, build status, TODO counts
- **30 commands** for file operations, sessions, pins, and more
- **Daemon** (`cfmd`) for fast cached banner data
- **Configurable** via `~/.config/cfm/config.toml`
- **Safe** with symlink guards, path protection, and dry-run support

## Quick Start

```bash
./install.sh
exec zsh   # or: source ~/.bashrc
```

## Commands

| Category | Commands |
|----------|----------|
| Banner | `f`, `f banner` |
| File Ops | `f mv`, `f cp`, `f rm`, `f trash`, `f open` |
| Clipboard | `f yank`, `f paste`, `f clipboard` |
| Piping | `f do`, `f peek` |
| Stats | `f stats` |
| Spatial | `f pin`, `f jump`, `f root`, `f pins`, `f unpin` |
| Sessions | `f save-session`, `f load-session`, `f sessions`, `f delete-session` |
| Diff | `f diff` |
| Shell | `f install-hook`, `f uninstall-hook`, `f completion` |
| Config | `f config`, `f config --edit`, `f config --get`, `f config --set` |
| Daemon | `f daemon start/stop/status/restart/clear-cache` |

## Testing

```bash
cargo test    # 76 tests pass
cargo clippy  # 0 warnings
```

## Architecture

- **`f`** — CLI binary (ephemeral: wake up, read state, print output, exit)
- **`cfmd`** — Background daemon (Unix socket IPC, inotify watching, proactive scanning)
- **`cfm-lib`** — Shared library (modules used by both binaries)

## License

MIT
