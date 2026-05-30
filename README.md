# cfm — Contextual File Manager

A directory listing with instant context.

## What It Does

When you run `f`, you see:
- File listing (like `ls`/`exa`/`lsd`)
- Git status
- Build status
- TODO count
- Project type
- Ports in use
- Docker status

**All instantly, no extra commands needed.**

## Quick Start

```bash
./install.sh
exec zsh   # or: source ~/.bashrc
```

## Usage

```bash
f                    # Directory listing + context
f <dir>              # Listing for specific dir
f --sort size        # Sort by size
f --hidden           # Show dotfiles
f --tree             # Tree view
f --json             # JSON output
f --filter rs        # Filter by pattern
```

## Flags

| Flag | Description |
|------|-------------|
| `--sort name\|size\|date\|type` | Sort order |
| `--reverse` | Reverse sort |
| `--hidden` | Show dotfiles |
| `--tree [depth]` | Tree view (0 = unlimited) |
| `--json` | JSON output |
| `--raw` | Plain text output |
| `--compact` | Less info |
| `--verbose` | More info |
| `--filter <pattern>` | Filter by name |
| `--max <N>` | Limit items |
| `--group` | Group by type |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CFM_NO_TODOS` | Set to `1` to disable TODO scanning |
| `CFM_NO_PORTS` | Set to `1` to disable port detection |
| `CFM_NO_DOCKER` | Set to `1` to disable Docker detection |
| `CFM_NO_METRICS` | Set to `1` to disable code metrics |
| `NO_COLOR` | Disable colors (per spec) |

## Testing

```bash
cargo test    # 77 tests pass
cargo clippy  # 0 warnings
```

## License

MIT
