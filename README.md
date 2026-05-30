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
| Banner | `fm`, `fm banner` |
| File Ops | `fm mv`, `fm cp`, `fm rm`, `fm trash`, `fm open` |
| Clipboard | `fm yank`, `fm paste`, `fm clipboard` |
| Piping | `fm do`, `fm peek` |
| Stats | `fm stats` |
| Spatial | `fm pin`, `fm jump`, `fm root`, `fm pins`, `fm unpin` |
| Sessions | `fm save-session`, `fm load-session`, `fm sessions`, `fm delete-session` |
| Diff | `fm diff` |
| Shell | `fm install-hook`, `fm uninstall-hook`, `fm completion` |
| Config | `fm config`, `fm config --edit`, `fm config --get`, `fm config --set` |
| Daemon | `fm daemon start/stop/status/restart/clear-cache` |

## Testing

```bash
cargo test    # 76 tests pass
cargo clippy  # 0 warnings
```

## Architecture

- **`fm`** — CLI binary (ephemeral: wake up, read state, print output, exit)
- **`cfmd`** — Background daemon (Unix socket IPC, inotify watching, proactive scanning)
- **`cfm-lib`** — Shared library (modules used by both binaries)

## License

MIT
